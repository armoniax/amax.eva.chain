use codec::{Decode, Encode};
use ethereum::TransactionV2 as Transaction;
use ethereum_types::H256;
use sp_runtime::DispatchError;
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
    pub trait DebugRuntimeApi {
        fn trace_transaction(
            extrinsics: Vec<Block::Extrinsic>,
            transaction: &Transaction,
        ) -> Result<(), DispatchError>;

        fn trace_block(
            extrinsics: Vec<Block::Extrinsic>,
            known_transactions: Vec<H256>,
        ) -> Result<(), DispatchError>;
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Encode, Decode)]
pub enum TracerInput {
    None,
    Blockscout,
    CallTracer,
}

/// DebugRuntimeApi V2 result. Trace response is stored in client and runtime api call response is
/// empty.
#[derive(Debug)]
pub enum Response {
    Single,
    Block,
}
