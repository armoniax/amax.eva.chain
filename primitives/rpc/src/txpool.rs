use codec::{Decode, Encode};
use ethereum::TransactionV2 as Transaction;
// Substrate
use sp_runtime::{traits::Block as BlockT, RuntimeDebug};
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
    pub trait TxPoolRuntimeApi {
        fn extrinsic_filter(
            xt_ready: Vec<<Block as BlockT>::Extrinsic>,
            xt_future: Vec<<Block as BlockT>::Extrinsic>,
        ) -> TxPoolResponse;
    }
}

#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub struct TxPoolResponse {
    pub ready: Vec<Transaction>,
    pub future: Vec<Transaction>,
}
