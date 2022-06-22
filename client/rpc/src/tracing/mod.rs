//! `trace_filter` RPC handler and its associated service task.
//! The RPC handler rely on `CacheTask` which provides a future that must be run inside a tokio
//! executor.
//!
//! The implementation is composed of multiple tasks :
//! - Many calls the the RPC handler `Trace::filter`, communicating with the main task.
//! - A main `CacheTask` managing the cache and the communication between tasks.
//! - For each traced block an async task responsible to wait for a permit, spawn a blocking task
//!   and waiting for the result, then send it to the main `CacheTask`.

use std::{marker::PhantomData, sync::Arc};

use ethereum_types::H256;
use futures::SinkExt;
use jsonrpsee::core::RpcResult;
use tokio::sync::oneshot;

use sp_api::{BlockId, HeaderT};
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_runtime::traits::Block as BlockT;

use fc_rpc::internal_err;

use amax_eva_client_evm_tracing::types::{
    block::{self, TransactionTrace},
    replay::{TraceResults, TraceResultsWithTransactionHash},
};
pub use amax_eva_rpc_core::{FilterRequest, RequestBlockId, RequestBlockTag, TraceServer};

mod cache;
pub use cache::{CacheRequester, CacheTask};

mod trace;
pub use trace::{Requester as TraceRequester, TraceTask};

type TxsTraceRes = Result<Vec<TransactionTrace>, String>;

/// RPC handler. Will communicate with a `CacheTask` through a `CacheRequester`.
pub struct Trace<B, C> {
    _phantom: PhantomData<B>,
    client: Arc<C>,
    trace_filter_requester: CacheRequester,
    trace_requester: TraceRequester,
    max_count: u32,
}

impl<B, C> Clone for Trace<B, C> {
    fn clone(&self) -> Self {
        Self {
            _phantom: PhantomData::default(),
            client: Arc::clone(&self.client),
            trace_filter_requester: self.trace_filter_requester.clone(),
            trace_requester: self.trace_requester.clone(),
            max_count: self.max_count,
        }
    }
}

impl<B, C> Trace<B, C>
where
    B: BlockT<Hash = H256> + Send + Sync + 'static,
    B::Header: HeaderT<Number = u32>,
    C: HeaderMetadata<B, Error = BlockChainError> + HeaderBackend<B>,
    C: Send + Sync + 'static,
{
    /// Create a new RPC handler.
    pub fn new(
        client: Arc<C>,
        trace_filter_requester: CacheRequester,
        trace_requester: TraceRequester,
        max_count: u32,
    ) -> Self {
        Self {
            client,
            trace_filter_requester,
            trace_requester,
            max_count,
            _phantom: PhantomData::default(),
        }
    }

    /// Convert an optional block ID (number or tag) to a block height.
    fn block_id(&self, id: Option<RequestBlockId>) -> RpcResult<u32> {
        match id {
            Some(RequestBlockId::Number(n)) => Ok(n),
            None | Some(RequestBlockId::Tag(RequestBlockTag::Latest)) => {
                Ok(self.client.info().best_number)
            },
            Some(RequestBlockId::Tag(RequestBlockTag::Earliest)) => Ok(0),
            Some(RequestBlockId::Tag(RequestBlockTag::Pending)) => {
                Err(internal_err("'pending' is not supported"))
            },
            Some(RequestBlockId::Hash(_)) => Err(internal_err("Block hash not supported")),
        }
    }

    /// Returns all traces of given transaction.
    async fn transaction_traces(
        self,
        transaction_hash: H256,
    ) -> RpcResult<Option<Vec<TransactionTrace>>> {
        let mut trace_requester = self.trace_requester.clone();
        let (tx, rx) = oneshot::channel();

        // Send a message from the rpc handler to the service level task.
        trace_requester
            .send((trace::Request::Transaction(transaction_hash), tx))
            .await?;

        // Receive a message from the service level task and send the rpc response.
        rx.await
            .map_err(|err| internal_err(format!("trace service dropped the channel : {:?}", err)))?
            .map(|res| match res {
                trace::Response::Traces(res) => Some(res),
            })
    }

    /// Returns all traces produced at given block.
    async fn block_traces(
        self,
        number: RequestBlockId,
    ) -> RpcResult<Option<Vec<TransactionTrace>>> {
        let mut trace_requester = self.trace_requester.clone();
        let (tx, rx) = oneshot::channel();

        // Send a message from the rpc handler to the service level task.
        trace_requester.send((trace::Request::Block(number), tx)).await?;

        // Receive a message from the service level task and send the rpc response.
        rx.await
            .map_err(|err| internal_err(format!("trace service dropped the channel : {:?}", err)))?
            .map(|res| match res {
                trace::Response::Traces(res) => Some(res),
            })
    }

    /// Executes the transaction with the given hash and returns a number of possible traces for it.
    async fn replay_transaction(self, _hash: H256, _opts: Vec<String>) -> RpcResult<TraceResults> {
        Err(internal_err("Current producer not support replay_transaction".to_string()))
    }

    /// Executes all the transactions at the given block and returns a number of possible traces for
    /// each transaction.
    async fn replay_block_transactions(
        self,
        _hash: RequestBlockId,
        _opts: Vec<String>,
    ) -> RpcResult<Vec<TraceResultsWithTransactionHash>> {
        Err(internal_err("Current producer not support replay_block_transactions".to_string()))
    }

    /// `trace_filter` endpoint (wrapped in the trait implementation with futures compatibilty)
    async fn filter(self, req: FilterRequest) -> RpcResult<Vec<TransactionTrace>> {
        let from_block = self.block_id(req.from_block)?;
        let to_block = self.block_id(req.to_block)?;
        let block_heights = from_block..=to_block;

        let count = req.count.unwrap_or(self.max_count);
        if count > self.max_count {
            return Err(internal_err(format!(
                "count ({}) can't be greater than maximum ({})",
                count, self.max_count
            )))
        }

        // Build a list of all the Substrate block hashes that need to be traced.
        let mut block_hashes = vec![];
        for block_height in block_heights {
            if block_height == 0 {
                continue // no traces for genesis block.
            }

            let block_id = BlockId::<B>::Number(block_height);
            let block_header = self
                .client
                .header(block_id)
                .map_err(|e| {
                    internal_err(format!(
                        "Error when fetching block {} header : {:?}",
                        block_height, e
                    ))
                })?
                .ok_or_else(|| {
                    internal_err(format!("Block with height {} don't exist", block_height))
                })?;

            let block_hash = block_header.hash();

            block_hashes.push(block_hash);
        }

        // Start a batch with these blocks.
        let batch_id = self.trace_filter_requester.start_batch(block_hashes.clone()).await?;
        // Fetch all the traces. It is done in another function to simplify error handling and allow
        // to call the following `stop_batch` regardless of the result. This is important for the
        // cache cleanup to work properly.
        let res = self.fetch_traces(req, &block_hashes, count as usize).await;
        // Stop the batch, allowing the cache task to remove useless non-started block traces and
        // start the expiration delay.
        self.trace_filter_requester.stop_batch(batch_id).await;

        res
    }
}

impl<B, C> Trace<B, C>
where
    B: BlockT<Hash = H256> + Send + Sync + 'static,
    B::Header: HeaderT<Number = u32>,
    C: HeaderMetadata<B, Error = BlockChainError> + HeaderBackend<B>,
    C: Send + Sync + 'static,
{
    async fn fetch_traces(
        &self,
        req: FilterRequest,
        block_hashes: &[H256],
        count: usize,
    ) -> RpcResult<Vec<TransactionTrace>> {
        let from_address = req.from_address.unwrap_or_default();
        let to_address = req.to_address.unwrap_or_default();

        let mut traces_amount: i64 = -(req.after.unwrap_or(0) as i64);
        let mut traces = vec![];

        for &block_hash in block_hashes {
            // Request the traces of this block to the cache service.
            // This will resolve quickly if the block is already cached, or wait until the block
            // has finished tracing.
            let block_traces =
                self.trace_filter_requester.get_traces_by_block_hash(block_hash).await?;

            // Filter addresses.
            let mut block_traces: Vec<_> = block_traces
                .iter()
                .filter(|trace| match trace.action {
                    block::TransactionTraceAction::Call { from, to, .. } => {
                        (from_address.is_empty() || from_address.contains(&from)) &&
                            (to_address.is_empty() || to_address.contains(&to))
                    },
                    block::TransactionTraceAction::Create { from, .. } => {
                        (from_address.is_empty() || from_address.contains(&from)) &&
                            to_address.is_empty()
                    },
                    block::TransactionTraceAction::Suicide { address, .. } => {
                        (from_address.is_empty() || from_address.contains(&address)) &&
                            to_address.is_empty()
                    },
                })
                .cloned()
                .collect();

            // Don't insert anything if we're still before "after"
            traces_amount += block_traces.len() as i64;
            if traces_amount > 0 {
                let traces_amount = traces_amount as usize;
                // If the current Vec of traces is across the "after" marker,
                // we skip some elements of it.
                if traces_amount < block_traces.len() {
                    let skip = block_traces.len() - traces_amount;
                    block_traces = block_traces.into_iter().skip(skip).collect();
                }

                traces.append(&mut block_traces);

                // If we go over "count" (the limit), we trim and exit the loop,
                // unless we used the default maximum, in which case we return an error.
                if traces_amount >= count {
                    if req.count.is_none() {
                        return Err(internal_err(format!(
                            "the amount of traces goes over the maximum ({}), please use 'after' \
                            and 'count' in your request",
                            self.max_count
                        )))
                    }

                    traces = traces.into_iter().take(count).collect();
                    break
                }
            }
        }

        Ok(traces)
    }
}

#[jsonrpsee::core::async_trait]
impl<B, C> TraceServer for Trace<B, C>
where
    B: BlockT<Hash = H256> + Send + Sync + 'static,
    B::Header: HeaderT<Number = u32>,
    C: HeaderMetadata<B, Error = BlockChainError> + HeaderBackend<B>,
    C: Send + Sync + 'static,
{
    async fn transaction_traces(&self, hash: H256) -> RpcResult<Option<Vec<TransactionTrace>>> {
        let server = self.clone();

        server.transaction_traces(hash).await.map_err(fc_rpc::internal_err)
    }

    async fn filter(&self, filter: FilterRequest) -> RpcResult<Vec<TransactionTrace>> {
        self.clone().filter(filter).await
    }

    async fn block_traces(
        &self,
        number: RequestBlockId,
    ) -> RpcResult<Option<Vec<TransactionTrace>>> {
        self.clone().block_traces(number).await.map_err(fc_rpc::internal_err)
    }

    /// Executes the transaction with the given hash and returns a number of possible traces for it.
    async fn replay_transaction(&self, hash: H256, opts: Vec<String>) -> RpcResult<TraceResults> {
        self.clone().replay_transaction(hash, opts).await
    }

    /// Executes all the transactions at the given block and returns a number of possible traces for
    /// each transaction.
    async fn replay_block_transactions(
        &self,
        number: RequestBlockId,
        opts: Vec<String>,
    ) -> RpcResult<Vec<TraceResultsWithTransactionHash>> {
        self.clone().replay_block_transactions(number, opts).await
    }
}
