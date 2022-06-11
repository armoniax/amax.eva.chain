//! Types for tracing all Ethereum transactions

use super::{block::*, Bytes};
use codec::{Decode, Encode};
use ethereum_types::{H160, H256, U256};
use serde::{Serialize, Serializer};
use std::collections::BTreeMap;

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
/// A diff of some chunk of memory.
pub struct MemoryDiff {
    /// Offset into memory the change begins.
    pub off: u32,
    /// The changed data.
    pub data: Bytes,
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
/// A diff of some storage value.
pub struct StorageDiff {
    /// Which key in storage is changed.
    pub key: U256,
    /// What the value has been changed to.
    pub val: U256,
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
/// A record of an executed VM operation.
pub struct VMExecutedOperation {
    /// The total gas used.
    pub used: u64,
    /// The stack item placed, if any.
    pub push: Vec<U256>,
    /// If altered, the memory delta.
    pub mem: Option<MemoryDiff>,
    /// The altered storage value, if any.
    pub store: Option<StorageDiff>,
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
/// A record of the execution of a single VM operation.
pub struct VMOperation {
    /// The program counter.
    pub pc: u32,
    /// The gas cost for this instruction.
    pub cost: u64,
    /// Information concerning the execution of the operation.
    pub ex: Option<VMExecutedOperation>,
    /// Subordinate trace of the CALL/CREATE if applicable.
    #[serde(bound = "VMTrace: Serialize")]
    pub sub: Option<VMTrace>,
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
/// A record of a full VM trace for a CALL/CREATE.
pub struct VMTrace {
    /// The code to be executed.
    pub code: Bytes,
    /// The operations executed.
    pub ops: Vec<VMOperation>,
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
/// Aux type for Diff::Changed.
pub struct ChangedType<T>
where
    T: Serialize,
{
    from: T,
    to: T,
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
/// Serde-friendly `Diff` shadow.
pub enum Diff<T>
where
    T: Serialize,
{
    #[serde(rename = "=")]
    Same,
    #[serde(rename = "+")]
    Born(T),
    #[serde(rename = "-")]
    Died(T),
    #[serde(rename = "*")]
    Changed(ChangedType<T>),
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
/// Serde-friendly `AccountDiff` shadow.
pub struct AccountDiff {
    pub balance: Diff<U256>,
    pub nonce: Diff<U256>,
    pub code: Diff<Bytes>,
    pub storage: BTreeMap<H256, Diff<H256>>,
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode)]
/// Serde-friendly `StateDiff` shadow.
pub struct StateDiff(BTreeMap<H160, AccountDiff>);

impl Serialize for StateDiff {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serialize::serialize(&self.0, serializer)
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "camelCase")]
/// A diff of some chunk of memory.
pub struct TraceResults {
    /// The output of the call/create
    pub output: Bytes,
    /// The transaction trace.
    pub trace: Vec<TraceResult>,
    /// The transaction trace.
    pub vm_trace: Option<VMTrace>,
    /// The transaction trace.
    pub state_diff: Option<StateDiff>,
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "camelCase")]
/// A diff of some chunk of memory.
pub struct TraceResultsWithTransactionHash {
    /// The output of the call/create
    pub output: Bytes,
    /// The transaction trace.
    pub trace: Vec<TraceResult>,
    /// The transaction trace.
    pub vm_trace: Option<VMTrace>,
    /// The transaction trace.
    pub state_diff: Option<StateDiff>,
    /// The transaction Hash.
    pub transaction_hash: H256,
}
