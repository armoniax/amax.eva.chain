use ethereum_types::{H160, H256};
use futures::future::BoxFuture;
use jsonrpc_core::Result;
use jsonrpc_derive::rpc;
use serde::Deserialize;

use amax_eva_client_evm_tracing::types::{
    block::TransactionTrace,
    replay::{TraceResults, TraceResultsWithTransactionHash},
};
use amax_eva_rpc_core_types::RequestBlockId;

pub use rpc_impl_Trace::gen_server::Trace as TraceServer;

#[rpc(server)]
pub trait Trace {
    /// Returns all traces of given transaction,
    #[rpc(name = "trace_transaction")]
    fn transaction_traces(
        &self,
        hash: H256,
    ) -> BoxFuture<'static, Result<Option<Vec<TransactionTrace>>>>;

    /// Returns traces matching given filter,
    #[rpc(name = "trace_filter")]
    fn filter(&self, filter: FilterRequest) -> BoxFuture<'static, Result<Vec<TransactionTrace>>>;

    /// Returns all traces produced at given block,
    #[rpc(name = "trace_block")]
    fn block_traces(
        &self,
        number: RequestBlockId,
    ) -> BoxFuture<'static, Result<Option<Vec<TransactionTrace>>>>;

    /// Executes the transaction with the given hash and returns a number of possible traces for it.
    #[rpc(name = "trace_replayTransaction")]
    fn replay_transaction(
        &self,
        _: H256,
        _: Vec<String>,
    ) -> BoxFuture<'static, Result<TraceResults>>;

    /// Executes all the transactions at the given block and returns a number of possible traces for
    /// each transaction.
    #[rpc(name = "trace_replayBlockTransactions")]
    fn replay_block_transactions(
        &self,
        _: RequestBlockId,
        _: Vec<String>,
    ) -> BoxFuture<'static, Result<Vec<TraceResultsWithTransactionHash>>>;
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterRequest {
    /// (optional?) From this block.
    pub from_block: Option<RequestBlockId>,

    /// (optional?) To this block.
    pub to_block: Option<RequestBlockId>,

    /// (optional) Sent from these addresses.
    pub from_address: Option<Vec<H160>>,

    /// (optional) Sent to these addresses.
    pub to_address: Option<Vec<H160>>,

    /// (optional) The offset trace number
    pub after: Option<u32>,

    /// (optional) Integer number of traces to display in a batch.
    pub count: Option<u32>,
}
