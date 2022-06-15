use std::{future::Future, marker::PhantomData, sync::Arc};

use ethereum_types::H256;
use futures::StreamExt;
use jsonrpsee::core::RpcResult;
use tokio::sync::{oneshot, Semaphore};

use sc_client_api::backend::{Backend, StateBackend, StorageProvider};
use sc_utils::mpsc::TracingUnboundedSender;
use sp_api::{ApiExt, BlockId, Core, HeaderT, ProvideRuntimeApi};
use sp_block_builder::BlockBuilder;
use sp_blockchain::{
    Backend as BlockchainBackend, Error as BlockChainError, HeaderBackend, HeaderMetadata,
};
use sp_runtime::traits::{BlakeTwo256, Block as BlockT, UniqueSaturatedInto};

use fc_rpc::{frontier_backend_client, internal_err, OverrideHandle};
use fp_rpc::EthereumRuntimeRPCApi;

use amax_eva_client_evm_tracing::{
    formatters::{trace_filter::Formatter, ResponseFormatter},
    types::{self, TransactionTrace},
};
use amax_eva_rpc_core_types::{RequestBlockId, RequestBlockTag};
use primitives_rpc::debug::DebugRuntimeApi;

pub enum Request {
    Transaction(H256),
    Block(RequestBlockId),
}

pub enum Response {
    Traces(Vec<TransactionTrace>),
}

pub type Responder = oneshot::Sender<RpcResult<Response>>;
pub type Requester = TracingUnboundedSender<(Request, Responder)>;

pub struct TraceTask<B: BlockT, C, BE>(PhantomData<(B, C, BE)>);

impl<B, C, BE> TraceTask<B, C, BE>
where
    BE: Backend<B> + 'static,
    BE::State: StateBackend<BlakeTwo256>,
    C: ProvideRuntimeApi<B>,
    C: StorageProvider<B, BE>,
    C: HeaderMetadata<B, Error = BlockChainError> + HeaderBackend<B>,
    C: Send + Sync + 'static,
    B: BlockT<Hash = H256> + Send + Sync + 'static,
    B::Header: HeaderT<Number = u32>,
    C::Api: BlockBuilder<B>,
    C::Api: EthereumRuntimeRPCApi<B>,
    C::Api: DebugRuntimeApi<B>,
    C::Api: ApiExt<B>,
{
    /// Return the trace of the transaction.
    fn handle_trace_transaction_req(
        client: Arc<C>,
        backend: Arc<BE>,
        frontier_backend: Arc<fc_db::Backend<B>>,
        transaction_hash: H256,
        overrides: Arc<OverrideHandle<B>>,
    ) -> RpcResult<Response> {
        // load the transaction's ethereum_block_hash hash and index.
        let (hash, index) = match frontier_backend_client::load_transactions::<B, C>(
            client.as_ref(),
            frontier_backend.as_ref(),
            transaction_hash,
            false,
        ) {
            Ok(Some((hash, index))) => (hash, index as usize),
            Ok(None) => return Err(internal_err("Transaction hash not found")),
            Err(e) => return Err(e),
        };

        // found the block hash by ethereum_block_hash
        let reference_id =
            match frontier_backend_client::load_hash::<B>(frontier_backend.as_ref(), hash) {
                Ok(Some(hash)) => hash,
                Ok(_) => return Err(internal_err("Block hash not found")),
                Err(e) => return Err(e),
            };

        // Get ApiRef. This handle allow to keep changes between txs in an internal buffer.
        let api = client.runtime_api();

        // Get Blockchain backend
        let blockchain = backend.blockchain();

        // Get the header I want to work with.
        let header = match client.header(reference_id) {
            Ok(Some(h)) => h,
            _ => return Err(internal_err("Block header not found")),
        };

        let height = *header.number();

        // Get parent blockid.
        let parent_block_id = BlockId::Hash(*header.parent_hash());

        // Get the extrinsics.
        let ext = blockchain.body(reference_id).unwrap().unwrap();

        let schema = frontier_backend_client::onchain_storage_schema::<B, C, BE>(
            client.as_ref(),
            reference_id,
        );

        // Get the block that contains the requested transaction. Using storage overrides we align
        // with `:ethereum_schema` which will result in proper SCALE decoding in case of migration.
        let (eth_block, eth_transactions) = match overrides.schemas.get(&schema) {
            Some(schema) => (
                schema.current_block(&reference_id),
                schema.current_transaction_statuses(&reference_id),
            ),
            _ => return Err(internal_err(format!("No storage override at {:?}", reference_id))),
        };

        // Get the actual ethereum transaction.
        if let (Some(eth_block), Some(eth_transactions)) = (eth_block, eth_transactions) {
            let transactions = eth_block.transactions;
            let eth_block_hash = eth_block.header.hash();

            if let Some(transaction) = transactions.get(index) {
                let f = || -> RpcResult<_> {
                    api.initialize_block(&parent_block_id, &header)
                        .map_err(|e| internal_err(format!("Runtime api access error: {:?}", e)))?;

                    let _result = api
                        .trace_transaction(&parent_block_id, ext, transaction)
                        .map_err(|e| internal_err(format!("Runtime api access error : {:?}", e)))?
                        .map_err(|e| internal_err(format!("DispatchError: {:?}", e)))?;

                    Ok(primitives_rpc::debug::Response::Single)
                };

                let mut proxy = amax_eva_client_evm_tracing::listeners::CallList::default();
                proxy.using(f)?;
                proxy.finish_transaction();

                let mut traces = match Formatter::format(proxy) {
                    Some(t) => t,
                    None => vec![],
                };

                for trace in traces.iter_mut() {
                    trace.block_hash = eth_block_hash;
                    trace.block_number = height;
                    trace.transaction_hash = eth_transactions
                        .get(trace.transaction_position as usize)
                        .ok_or_else(|| {
                            tracing::warn!(
                                "Bug: A transaction has been replayed while it shouldn't (in block {}).",
                                height
                            );

                            internal_err(format!(
                                "Bug: A transaction has been replayed while it shouldn't (in block {}).",
                                height
                            ))
                        })?
                        .transaction_hash;

                    // Reformat error messages.
                    if let types::block::TransactionTraceOutput::Error(ref mut error) = trace.output
                    {
                        if error.as_slice() == b"execution reverted" {
                            *error = b"Reverted".to_vec();
                        }
                    }
                }

                return Ok(Response::Traces(traces))
            }
        }

        Err(internal_err("Runtime block call failed"))
    }

    /// Return the trace of the transactions in a block.
    fn handle_trace_block_req(
        client: Arc<C>,
        backend: Arc<BE>,
        frontier_backend: Arc<fc_db::Backend<B>>,
        request_block_id: RequestBlockId,
        overrides: Arc<OverrideHandle<B>>,
    ) -> RpcResult<Response> {
        let reference_id: BlockId<B> = match request_block_id {
            RequestBlockId::Number(n) => Ok(BlockId::Number(n.unique_saturated_into())),
            RequestBlockId::Tag(RequestBlockTag::Latest) => {
                Ok(BlockId::Number(client.info().best_number))
            },
            RequestBlockId::Tag(RequestBlockTag::Earliest) => {
                Ok(BlockId::Number(0u32.unique_saturated_into()))
            },
            RequestBlockId::Tag(RequestBlockTag::Pending) => {
                Err(internal_err("'pending' blocks are not supported"))
            },
            RequestBlockId::Hash(eth_hash) => {
                match frontier_backend_client::load_hash::<B>(frontier_backend.as_ref(), eth_hash) {
                    Ok(Some(id)) => Ok(id),
                    Ok(_) => Err(internal_err("Block hash not found")),
                    Err(e) => Err(e),
                }
            },
        }?;

        // Get ApiRef. This handle allow to keep changes between txs in an internal buffer.
        let api = client.runtime_api();
        // Get Blockchain backend
        let blockchain = backend.blockchain();
        // Get the header I want to work with.
        let header = match client.header(reference_id) {
            Ok(Some(h)) => h,
            _ => return Err(internal_err("Block header not found")),
        };

        // Get parent blockid.
        let parent_block_id = BlockId::Hash(*header.parent_hash());

        let schema = frontier_backend_client::onchain_storage_schema::<B, C, BE>(
            client.as_ref(),
            reference_id,
        );

        // Get the block that contains the requested transaction. Using storage overrides we align
        // with `:ethereum_schema` which will result in proper SCALE decoding in case of migration.
        let (eth_block, eth_transactions) = match overrides.schemas.get(&schema) {
            Some(schema) => (
                schema.current_block(&reference_id),
                schema.current_transaction_statuses(&reference_id),
            ),
            _ => return Err(internal_err(format!("No storage override at {:?}", reference_id))),
        };

        // Known ethereum transaction hashes.
        let eth_tx_hashes: Vec<_> =
            eth_transactions.clone().unwrap().iter().map(|t| t.transaction_hash).collect();

        // If there are no ethereum transactions in the block return empty trace right away.
        if eth_tx_hashes.is_empty() {
            return Ok(Response::Traces(vec![]))
        }

        // Get the extrinsics.
        let ext = blockchain.body(reference_id).unwrap().unwrap();

        // Trace the block.
        let f = || -> RpcResult<_> {
            api.initialize_block(&parent_block_id, &header)
                .map_err(|e| internal_err(format!("Runtime api access error: {:?}", e)))?;

            let _result = api
                .trace_block(&parent_block_id, ext, eth_tx_hashes)
                .map_err(|e| {
                    internal_err(format!(
                        "Blockchain error when replaying block {} : {:?}",
                        reference_id, e
                    ))
                })?
                .map_err(|e| {
                    internal_err(format!(
                        "Internal runtime error when replaying block {} : {:?}",
                        reference_id, e
                    ))
                })?;
            Ok(primitives_rpc::debug::Response::Block)
        };

        let mut proxy = amax_eva_client_evm_tracing::listeners::CallList::default();
        proxy.using(f)?;
        proxy.finish_transaction();

        let mut traces = match Formatter::format(proxy) {
            Some(t) => t,
            None => vec![],
        };

        let height = *header.number();

        if let (Some(eth_block), Some(eth_transactions)) = (eth_block, eth_transactions) {
            let eth_block_hash = eth_block.header.hash();

            for trace in traces.iter_mut() {
                trace.block_hash = eth_block_hash;
                trace.block_number = height;
                trace.transaction_hash = eth_transactions
                    .get(trace.transaction_position as usize)
                    .ok_or_else(|| {
                        tracing::warn!(
                            "Bug: A transaction has been replayed while it shouldn't (in block {}).",
                            height
                        );

                        internal_err(format!(
                            "Bug: A transaction has been replayed while it shouldn't (in block {}).",
                            height
                        ))
                    })?
                    .transaction_hash;

                // Reformat error messages.
                if let types::block::TransactionTraceOutput::Error(ref mut error) = trace.output {
                    if error.as_slice() == b"execution reverted" {
                        *error = b"Reverted".to_vec();
                    }
                }
            }
        }

        Ok(Response::Traces(traces))
    }
}

impl<B, C, BE> TraceTask<B, C, BE>
where
    BE: Backend<B> + 'static,
    BE::State: StateBackend<BlakeTwo256>,
    C: ProvideRuntimeApi<B>,
    C: StorageProvider<B, BE>,
    C: HeaderMetadata<B, Error = BlockChainError> + HeaderBackend<B>,
    C: Send + Sync + 'static,
    B: BlockT<Hash = H256> + Send + Sync + 'static,
    B::Header: HeaderT<Number = u32>,
    C::Api: BlockBuilder<B>,
    C::Api: EthereumRuntimeRPCApi<B>,
    C::Api: DebugRuntimeApi<B>,
    C::Api: ApiExt<B>,
{
    /// Task spawned at service level that listens for messages on the rpc channel and spawns
    /// blocking tasks using a permit pool.
    pub fn create(
        client: Arc<C>,
        backend: Arc<BE>,
        frontier_backend: Arc<fc_db::Backend<B>>,
        permit_pool: Arc<Semaphore>,
        overrides: Arc<OverrideHandle<B>>,
    ) -> (impl Future<Output = ()>, Requester) {
        // Communication with the outside world :
        let (tx, mut rx): (Requester, _) = sc_utils::mpsc::tracing_unbounded("trace-requester");

        // Task running in the service.
        let task = async move {
            loop {
                match rx.next().await {
                    Some((Request::Transaction(transaction_hash), response_tx)) => {
                        let client = client.clone();
                        let backend = backend.clone();
                        let frontier_backend = frontier_backend.clone();
                        let permit_pool = permit_pool.clone();
                        let overrides = overrides.clone();

                        tokio::task::spawn(async move {
                            let _ = response_tx.send(
                                async {
                                    let _permit = permit_pool.acquire().await;
                                    tokio::task::spawn_blocking(move || {
                                        Self::handle_trace_transaction_req(
                                            client.clone(),
                                            backend.clone(),
                                            frontier_backend.clone(),
                                            transaction_hash,
                                            overrides,
                                        )
                                    })
                                    .await
                                    .map_err(|e| {
                                        internal_err(format!(
                                            "Internal error on spawned task : {:?}",
                                            e
                                        ))
                                    })?
                                }
                                .await,
                            );
                        });
                    },
                    Some((Request::Block(request_block_id), response_tx)) => {
                        let client = client.clone();
                        let backend = backend.clone();
                        let frontier_backend = frontier_backend.clone();
                        let permit_pool = permit_pool.clone();
                        let overrides = overrides.clone();

                        tokio::task::spawn(async move {
                            let _ = response_tx.send(
                                async {
                                    let _permit = permit_pool.acquire().await;

                                    tokio::task::spawn_blocking(move || {
                                        Self::handle_trace_block_req(
                                            client.clone(),
                                            backend.clone(),
                                            frontier_backend.clone(),
                                            request_block_id,
                                            overrides,
                                        )
                                    })
                                    .await
                                    .map_err(|e| {
                                        internal_err(format!(
                                            "Internal error on spawned task : {:?}",
                                            e
                                        ))
                                    })?
                                }
                                .await,
                            );
                        });
                    },
                    _ => {},
                }
            }
        };
        (task, tx)
    }
}
