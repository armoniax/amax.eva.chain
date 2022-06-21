use codec::{Decode, Encode};
use sp_runtime::traits::Extrinsic;
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
