// Substrate
use sp_core::H160;
use sp_runtime::Permill;
// Substrate FRAME
use frame_support::weights::Weight;
// Local
use runtime_common_constants::evm::WEIGHT_PER_GAS;

/// And implementation of Frontier's AddressMapping trait for Moonbeam Accounts.
/// This is basically identical to Frontier's own IdentityAddressMapping, but it works for any type
/// that is Into<H160> like AccountId20 for example.
pub struct IntoAddressMapping;
impl<T: From<H160>> pallet_evm::AddressMapping<T> for IntoAddressMapping {
    fn into_account_id(address: H160) -> T {
        address.into()
    }
}

pub struct GasWeightMapping;
impl pallet_evm::GasWeightMapping for GasWeightMapping {
    fn gas_to_weight(gas: u64) -> Weight {
        gas.saturating_mul(WEIGHT_PER_GAS)
    }
    fn weight_to_gas(weight: Weight) -> u64 {
        weight.wrapping_div(WEIGHT_PER_GAS)
    }
}

pub struct BaseFeeThreshold;
impl pallet_base_fee::BaseFeeThreshold for BaseFeeThreshold {
    fn lower() -> Permill {
        Permill::zero()
    }
    fn ideal() -> Permill {
        Permill::from_parts(500_000)
    }
    fn upper() -> Permill {
        Permill::from_parts(1_000_000)
    }
}
