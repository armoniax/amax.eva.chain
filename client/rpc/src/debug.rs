use std::{future::Future, marker::PhantomData, sync::Arc};

use ethereum_types::H256;
use futures::{SinkExt, StreamExt};
use jsonrpsee::core::{async_trait, RpcResult};
use tokio::sync::{oneshot, Semaphore};

// Substrate
use sc_client_api::backend::{Backend, StateBackend, StorageProvider};
use sc_utils::mpsc::TracingUnboundedSender;
use sp_api::{BlockId, Core, HeaderT, ProvideRuntimeApi};
use sp_blockchain::{
    Backend as BlockchainBackend, Error as BlockChainError, HeaderBackend, HeaderMetadata,
};
use sp_runtime::traits::{BlakeTwo256, Block as BlockT, UniqueSaturatedInto};

// Frontier
use fc_rpc::{frontier_backend_client, internal_err, OverrideHandle};

// Local
use amax_eva_client_evm_tracing::formatters::ResponseFormatter;
pub use amax_eva_rpc_core::{single, DebugApiServer, RequestBlockId, RequestBlockTag, TraceParams};
pub use primitives_rpc::debug::{DebugRuntimeApi, TracerInput};

pub enum RequesterInput {
    Transaction(H256),
    Block(RequestBlockId),
}

pub enum Response {
    Single(single::TransactionTrace),
    Block(Vec<single::TransactionTrace>),
}

pub type Responder = oneshot::Sender<RpcResult<Response>>;
pub type DebugRequester =
    TracingUnboundedSender<((RequesterInput, Option<TraceParams>), Responder)>;

pub struct Debug {
    pub requester: DebugRequester,
}

impl Debug {
    pub fn new(requester: DebugRequester) -> Self {
        Self { requester }
    }
}

#[async_trait]
impl DebugApiServer for Debug {
    /// Handler for `debug_traceTransaction` request. Communicates with the service-defined task
    /// using channels.
    async fn trace_transaction(
        &self,
        transaction_hash: H256,
        params: Option<TraceParams>,
    ) -> RpcResult<single::TransactionTrace> {
        let mut requester = self.requester.clone();

        let (tx, rx) = oneshot::channel();
        // Send a message from the rpc handler to the service level task.
        requester
            .send(((RequesterInput::Transaction(transaction_hash), params), tx))
            .await
            .map_err(|err| {
                internal_err(format!("failed to send request to debug service : {:?}", err))
            })?;

        // Receive a message from the service level task and send the rpc response.
        rx.await
            .map_err(|err| internal_err(format!("debug service dropped the channel : {:?}", err)))?
            .map(|res| match res {
                Response::Single(res) => res,
                _ => unreachable!(),
            })
    }

    async fn trace_block(
        &self,
        id: RequestBlockId,
        params: Option<TraceParams>,
    ) -> RpcResult<Vec<single::TransactionTrace>> {
        let mut requester = self.requester.clone();

        let (tx, rx) = oneshot::channel();
        // Send a message from the rpc handler to the service level task.
        requester.send(((RequesterInput::Block(id), params), tx)).await.map_err(|err| {
            internal_err(format!("failed to send request to debug service : {:?}", err))
        })?;

        // Receive a message from the service level task and send the rpc response.
        rx.await
            .map_err(|err| internal_err(format!("debug service dropped the channel : {:?}", err)))?
            .map(|res| match res {
                Response::Block(res) => res,
                _ => unreachable!(),
            })
    }
}

pub struct DebugHandler<B: BlockT, C, BE>(PhantomData<(B, C, BE)>);

impl<B, C, BE> DebugHandler<B, C, BE>
where
    B: BlockT<Hash = H256> + Send + Sync + 'static,
    C: ProvideRuntimeApi<B> + StorageProvider<B, BE>,
    C: HeaderMetadata<B, Error = BlockChainError> + HeaderBackend<B> + 'static,
    C::Api: sp_api::ApiExt<B>,
    C::Api: sp_block_builder::BlockBuilder<B>,
    C::Api: fp_rpc::EthereumRuntimeRPCApi<B>,
    C::Api: primitives_rpc::debug::DebugRuntimeApi<B>,
    BE: Backend<B> + 'static,
    BE::State: StateBackend<BlakeTwo256>,
{
    /// Task spawned at service level that listens for messages on the rpc channel and spawns
    /// blocking tasks using a permit pool.
    pub fn task(
        client: Arc<C>,
        backend: Arc<BE>,
        frontier_backend: Arc<fc_db::Backend<B>>,
        permit_pool: Arc<Semaphore>,
        overrides: Arc<OverrideHandle<B>>,
    ) -> (impl Future<Output = ()>, DebugRequester) {
        let (tx, mut rx): (DebugRequester, _) =
            sc_utils::mpsc::tracing_unbounded("debug-requester");

        let fut = async move {
            loop {
                match rx.next().await {
                    Some((
                        (RequesterInput::Transaction(transaction_hash), params),
                        response_tx,
                    )) => {
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
                                        Self::handle_transaction_request(
                                            client.clone(),
                                            backend.clone(),
                                            frontier_backend.clone(),
                                            transaction_hash,
                                            params,
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
                    Some(((RequesterInput::Block(request_block_id), params), response_tx)) => {
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
                                        Self::handle_block_request(
                                            client.clone(),
                                            backend.clone(),
                                            frontier_backend.clone(),
                                            request_block_id,
                                            params,
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
        (fut, tx)
    }

    fn handle_params(params: Option<TraceParams>) -> RpcResult<(TracerInput, single::TraceType)> {
        // Set trace input and type
        match params {
            Some(TraceParams { tracer: Some(tracer), .. }) => {
                const BLOCKSCOUT_JS_CODE_HASH: [u8; 16] =
                    hex_literal::hex!("94d9f08796f91eb13a2e82a6066882f7");
                const BLOCKSCOUT_JS_CODE_HASH_V2: [u8; 16] =
                    hex_literal::hex!("89db13694675692951673a1e6e18ff02");
                let hash = sp_io::hashing::twox_128(tracer.as_bytes());
                let tracer =
                    if hash == BLOCKSCOUT_JS_CODE_HASH || hash == BLOCKSCOUT_JS_CODE_HASH_V2 {
                        Some(TracerInput::Blockscout)
                    } else if tracer == "callTracer" {
                        Some(TracerInput::CallTracer)
                    } else {
                        None
                    };
                if let Some(tracer) = tracer {
                    Ok((tracer, single::TraceType::CallList))
                } else {
                    return Err(internal_err(format!(
                        "javascript based tracing is not available (hash :{:?})",
                        hash
                    )))
                }
            },
            Some(params) => Ok((
                TracerInput::None,
                single::TraceType::Raw {
                    disable_storage: params.disable_storage.unwrap_or(false),
                    disable_memory: params.disable_memory.unwrap_or(false),
                    disable_stack: params.disable_stack.unwrap_or(false),
                },
            )),
            _ => Ok((
                TracerInput::None,
                single::TraceType::Raw {
                    disable_storage: false,
                    disable_memory: false,
                    disable_stack: false,
                },
            )),
        }
    }

    fn handle_block_request(
        client: Arc<C>,
        backend: Arc<BE>,
        frontier_backend: Arc<fc_db::Backend<B>>,
        request_block_id: RequestBlockId,
        params: Option<TraceParams>,
        overrides: Arc<OverrideHandle<B>>,
    ) -> RpcResult<Response> {
        let (tracer_input, trace_type) = Self::handle_params(params)?;

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
                    Ok(_) => Err(internal_err("Block hash not found".to_string())),
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

        // Using storage overrides we align with `:ethereum_schema` which will result in proper
        // SCALE decoding in case of migration.
        let statuses = match overrides.schemas.get(&schema) {
            Some(schema) => schema.current_transaction_statuses(&reference_id),
            _ => return Err(internal_err(format!("No storage override at {:?}", reference_id))),
        };

        // Known ethereum transaction hashes.
        let eth_tx_hashes: Vec<_> = statuses.unwrap().iter().map(|t| t.transaction_hash).collect();

        // If there are no ethereum transactions in the block return empty trace right away.
        if eth_tx_hashes.is_empty() {
            return Ok(Response::Block(vec![]))
        }

        // Get the extrinsics.
        let ext = blockchain.body(reference_id).unwrap().unwrap();

        // Trace the block.
        let f = || -> RpcResult<_> {
            api.initialize_block(&parent_block_id, &header)
                .map_err(|e| internal_err(format!("Runtime api access error: {:?}", e)))?;

            api.trace_block(&parent_block_id, ext, eth_tx_hashes)
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

        return match trace_type {
            single::TraceType::CallList => {
                let mut proxy = amax_eva_client_evm_tracing::listeners::CallList::default();
                proxy.using(f)?;
                proxy.finish_transaction();
                let response = match tracer_input {
                    TracerInput::CallTracer => {
                        amax_eva_client_evm_tracing::formatters::CallTracer::format(proxy)
                            .ok_or("Trace result is empty.")
                            .map_err(|e| internal_err(format!("{:?}", e)))
                    },
                    _ => Err(internal_err("Bug: failed to resolve the tracer format.".to_string())),
                }?;

                Ok(Response::Block(response))
            },
            not_supported => Err(internal_err(format!(
                "Bug: `handle_block_request` does not support {:?}.",
                not_supported
            ))),
        }
    }

    /// Replays a transaction in the Runtime at a given block height.
    ///
    /// In order to successfully reproduce the result of the original transaction we need a correct
    /// state to replay over.
    ///
    /// Substrate allows to apply extrinsics in the Runtime and thus creating an overlayed state.
    /// This overlayed changes will live in-memory for the lifetime of the ApiRef.
    fn handle_transaction_request(
        client: Arc<C>,
        backend: Arc<BE>,
        frontier_backend: Arc<fc_db::Backend<B>>,
        transaction_hash: H256,
        params: Option<TraceParams>,
        overrides: Arc<OverrideHandle<B>>,
    ) -> RpcResult<Response> {
        let (tracer_input, trace_type) = Self::handle_params(params)?;

        let (hash, index) = match frontier_backend_client::load_transactions::<B, C>(
            client.as_ref(),
            frontier_backend.as_ref(),
            transaction_hash,
            false,
        ) {
            Ok(Some((hash, index))) => (hash, index as usize),
            Ok(None) => return Err(internal_err("Transaction hash not found".to_string())),
            Err(e) => return Err(e),
        };

        let reference_id =
            match frontier_backend_client::load_hash::<B>(frontier_backend.as_ref(), hash) {
                Ok(Some(hash)) => hash,
                Ok(_) => return Err(internal_err("Block hash not found".to_string())),
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
        let reference_block = match overrides.schemas.get(&schema) {
            Some(schema) => schema.current_block(&reference_id),
            _ => return Err(internal_err(format!("No storage override at {:?}", reference_id))),
        };

        // Get the actual ethereum transaction.
        if let Some(block) = reference_block {
            let transactions = block.transactions;
            if let Some(transaction) = transactions.get(index) {
                let f = || -> RpcResult<_> {
                    api.initialize_block(&parent_block_id, &header)
                        .map_err(|e| internal_err(format!("Runtime api access error: {:?}", e)))?;

                    api.trace_transaction(&parent_block_id, ext, transaction)
                        .map_err(|e| internal_err(format!("Runtime api access error : {:?}", e)))?
                        .map_err(|e| internal_err(format!("DispatchError: {:?}", e)))?;

                    Ok(primitives_rpc::debug::Response::Single)
                };

                return match trace_type {
                    single::TraceType::Raw { disable_storage, disable_memory, disable_stack } => {
                        let mut proxy = amax_eva_client_evm_tracing::listeners::Raw::new(
                            disable_storage,
                            disable_memory,
                            disable_stack,
                        );
                        proxy.using(f)?;
                        Ok(Response::Single(
                            amax_eva_client_evm_tracing::formatters::Raw::format(proxy).unwrap(),
                        ))
                    },
                    single::TraceType::CallList => {
                        let mut proxy = amax_eva_client_evm_tracing::listeners::CallList::default();
                        proxy.using(f)?;
                        proxy.finish_transaction();
                        let response = match tracer_input {
                            TracerInput::Blockscout => {
                                amax_eva_client_evm_tracing::formatters::Blockscout::format(proxy)
                                    .ok_or("Trace result is empty.")
                                    .map_err(|e| internal_err(format!("{:?}", e)))
                            },
                            TracerInput::CallTracer => {
                                let mut res =
                                    amax_eva_client_evm_tracing::formatters::CallTracer::format(
                                        proxy,
                                    )
                                    .ok_or("Trace result is empty.")
                                    .map_err(|e| internal_err(format!("{:?}", e)))?;
                                Ok(res.pop().unwrap())
                            },
                            _ => Err(internal_err(
                                "Bug: failed to resolve the tracer format.".to_string(),
                            )),
                        }?;
                        Ok(Response::Single(response))
                    },
                    not_supported => Err(internal_err(format!(
                        "Bug: `handle_transaction_request` does not support {:?}.",
                        not_supported
                    ))),
                }
            }
        }
        Err(internal_err("Runtime block call failed".to_string()))
    }
}
