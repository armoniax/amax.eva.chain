use crate::{
    listeners::call_list::Listener,
    types::{
        single::{Call, TransactionTrace},
        Bytes, CallResult, CallType, CreateResult,
    },
};
use codec::{Decode, Encode};
use ethereum_types::{H160, U256};
use serde::Serialize;

pub struct Formatter;

impl super::ResponseFormatter for Formatter {
    type Listener = Listener;
    type Response = TransactionTrace;

    fn format(listener: Listener) -> Option<TransactionTrace> {
        if let Some(entry) = listener.entries.last() {
            return Some(TransactionTrace::CallList(
                entry.iter().map(|(_, value)| Call::Blockscout(value.clone())).collect(),
            ))
        }
        None
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum BlockscoutCallInner {
    Call {
        #[serde(rename(serialize = "callType"))]
        /// Type of call.
        call_type: CallType,
        to: H160,
        input: Bytes,
        /// "output" or "error" field
        #[serde(flatten)]
        res: CallResult,
    },
    Create {
        init: Bytes,
        #[serde(flatten)]
        res: CreateResult,
    },
    SelfDestruct {
        #[serde(skip)]
        balance: U256,
        to: H160,
    },
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockscoutCall {
    pub from: H160,
    /// Indices of parent calls.
    pub trace_address: Vec<u32>,
    /// Number of children calls.
    /// Not needed for Blockscout, but needed for `crate::block`
    /// types that are build from this type.
    #[serde(skip)]
    pub subtraces: u32,
    /// Sends funds to the (payable) function
    pub value: U256,
    /// Remaining gas in the runtime.
    pub gas: U256,
    /// Gas used by this context.
    pub gas_used: U256,
    #[serde(flatten)]
    pub inner: BlockscoutCallInner,
}
