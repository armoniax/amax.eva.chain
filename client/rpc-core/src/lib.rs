use ethereum_types::{H160, H256};
use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use serde::Deserialize;

use amax_eva_client_evm_tracing::{
    formatters::deserialize::*,
    types::{
        block::TransactionTrace,
        replay::{TraceResults, TraceResultsWithTransactionHash},
    },
};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum RequestBlockId {
    Number(#[serde(deserialize_with = "deserialize_u32_0x")] u32),
    Hash(H256),
    Tag(RequestBlockTag),
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RequestBlockTag {
    Earliest,
    Latest,
    Pending,
}

#[rpc(server)]
#[jsonrpsee::core::async_trait]
pub trait Trace {
    /// Returns all traces of given transaction,
    #[method(name = "trace_transaction")]
    async fn transaction_traces(&self, hash: H256) -> RpcResult<Option<Vec<TransactionTrace>>>;

    /// Returns traces matching given filter,
    #[method(name = "trace_filter")]
    async fn filter(&self, filter: FilterRequest) -> RpcResult<Vec<TransactionTrace>>;

    /// Returns all traces produced at given block,
    #[method(name = "trace_block")]
    async fn block_traces(
        &self,
        number: RequestBlockId,
    ) -> RpcResult<Option<Vec<TransactionTrace>>>;

    /// Executes the transaction with the given hash and returns a number of possible traces for it.
    #[method(name = "trace_replayTransaction")]
    async fn replay_transaction(&self, hash: H256, opts: Vec<String>) -> RpcResult<TraceResults>;

    /// Executes all the transactions at the given block and returns a number of possible traces for
    /// each transaction.
    #[method(name = "trace_replayBlockTransactions")]
    async fn replay_block_transactions(
        &self,
        hash: RequestBlockId,
        opts: Vec<String>,
    ) -> RpcResult<Vec<TraceResultsWithTransactionHash>>;
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
