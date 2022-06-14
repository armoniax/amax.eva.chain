use codec::{Decode, Encode};
use frame_support::traits::FindAuthor;
use primitives_core::AccountId20;
use sp_core::H160;
use sp_runtime::{traits::Extrinsic, ConsensusEngineId};
use sp_std::marker::PhantomData;

/// EthTransaction for rpc.
pub struct EthTransactionConverter<UE, R>(PhantomData<(UE, R)>);
impl<UE, R> EthTransactionConverter<UE, R> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        EthTransactionConverter(Default::default())
    }
}

impl<UncheckedExtrinsic, Runtime> fp_rpc::ConvertTransaction<primitives_core::UncheckedExtrinsic>
    for EthTransactionConverter<UncheckedExtrinsic, Runtime>
where
    UncheckedExtrinsic: Extrinsic + Encode,
    <UncheckedExtrinsic as Extrinsic>::Call: From<pallet_ethereum::Call<Runtime>>,
    Runtime: pallet_ethereum::Config,
    Result<pallet_ethereum::RawOrigin, <Runtime as frame_system::Config>::Origin>:
        From<<Runtime as frame_system::Config>::Origin>,
{
    fn convert_transaction(
        &self,
        transaction: crate::EthereumTransaction,
    ) -> primitives_core::UncheckedExtrinsic {
        let extrinsic = UncheckedExtrinsic::new(
            pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
            None,
        )
        .expect("must be some for `generic::UncheckedExtrinsic`");
        let encoded = extrinsic.encode();
        primitives_core::UncheckedExtrinsic::decode(&mut &encoded[..])
            .expect("Encoded extrinsic is always valid")
    }
}

pub struct CoinbaseAuthor<Runtime, F>(PhantomData<(Runtime, F)>);
impl<Runtime: pallet_session::Config<ValidatorId = AccountId20>, F: FindAuthor<u32>>
    FindAuthor<H160> for CoinbaseAuthor<Runtime, F>
{
    fn find_author<'a, I>(_digests: I) -> Option<H160>
    where
        I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
    {
        #[cfg(feature = "aura")]
        {
            pallet_session::FindAccountFromAuthorIndex::<Runtime, F>::find_author(_digests)
                .map(Into::into)
        }
        #[cfg(feature = "manual-seal")]
        {
            None
        }
    }
}
