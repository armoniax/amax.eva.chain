//! Types for tracing all Ethereum transactions of a block.

use super::{serialization::*, Bytes};
use serde::Serialize;

use codec::{Decode, Encode};
use ethereum_types::{H160, H256, U256};

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionTrace {
    #[serde(flatten)]
    pub action: TransactionTraceAction,
    #[serde(serialize_with = "h256_0x_serialize")]
    pub block_hash: H256,
    pub block_number: u32,
    #[serde(flatten)]
    pub output: TransactionTraceOutput,
    pub subtraces: u32,
    pub trace_address: Vec<u32>,
    #[serde(serialize_with = "h256_0x_serialize")]
    pub transaction_hash: H256,
    pub transaction_position: u32,
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "action")]
pub enum TransactionTraceAction {
    #[serde(rename_all = "camelCase")]
    Call { call_type: super::CallType, from: H160, gas: U256, input: Bytes, to: H160, value: U256 },
    #[serde(rename_all = "camelCase")]
    Create { creation_method: super::CreateType, from: H160, gas: U256, init: Bytes, value: U256 },
    #[serde(rename_all = "camelCase")]
    Suicide { address: H160, balance: U256, refund_address: H160 },
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TransactionTraceOutput {
    Result(TransactionTraceResult),
    Error(#[serde(serialize_with = "string_serialize")] Vec<u8>),
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum TransactionTraceResult {
    #[serde(rename_all = "camelCase")]
    Call {
        gas_used: U256,
        output: Bytes,
    },
    #[serde(rename_all = "camelCase")]
    Create {
        address: H160,
        code: Bytes,
        gas_used: U256,
    },
    Suicide,
}

/// Trace
#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceResult {
    /// Trace address
    trace_address: Vec<u32>,
    /// Subtraces
    subtraces: u32,
    /// Action
    action: TransactionTraceAction,
    /// Result
    result: TransactionTraceResult,
}
