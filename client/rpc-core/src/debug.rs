use ethereum_types::H256;
use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use serde::Deserialize;

pub use amax_eva_client_evm_tracing::types::single;

use crate::types::*;

#[derive(Clone, Eq, PartialEq, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceParams {
    pub disable_storage: Option<bool>,
    pub disable_memory: Option<bool>,
    pub disable_stack: Option<bool>,
    /// Javascript tracer (we just check if it's Blockscout tracer string)
    pub tracer: Option<String>,
    pub timeout: Option<String>,
}

#[rpc(server)]
#[async_trait]
pub trait DebugApi {
    /// The traceBlock method will return a full stack trace of all invoked opcodes of
    /// all transaction that were included in this block.
    /// Note, the parent of this block must be present or it will fail.
    ///
    /// For details, see [debug_traceBlock](https://geth.ethereum.org/docs/rpc/ns-debug#debug_traceblock)
    #[method(name = "debug_traceBlockByNumber", aliases = ["debug_traceBlockByHash"])]
    async fn trace_block(
        &self,
        id: RequestBlockId,
        params: Option<TraceParams>,
    ) -> RpcResult<Vec<single::TransactionTrace>>;

    /// The traceTransaction debugging method will attempt to run the transaction
    /// in the exact same manner as it was executed on the network.
    /// It will replay any transaction that may have been executed prior to
    /// this one before it will finally attempt to execute the transaction
    /// that corresponds to the given hash.
    ///
    /// For details, see [debug_traceTransaction](https://geth.ethereum.org/docs/rpc/ns-debug#debug_tracetransaction)
    ///
    /// In addition to the hash of the transaction you may give it a secondary optional argument,
    /// which specifies the options for this specific call. The possible options are:
    ///
    /// - disableStorage: BOOL. Setting this to true will disable storage capture (default = false).
    /// - disableStack: BOOL. Setting this to true will disable stack capture (default = false).
    /// - disableMemory: BOOL. Setting this to true will enable memory capture (default = false).
    #[method(name = "debug_traceTransaction")]
    async fn trace_transaction(
        &self,
        transaction_hash: H256,
        params: Option<TraceParams>,
    ) -> RpcResult<single::TransactionTrace>;
}
