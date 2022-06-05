use frame_support::weights::{constants::WEIGHT_PER_SECOND, Weight};
use sp_core::{H160, U256};
use sp_runtime::Permill;

pub struct FixedGasPrice;
impl pallet_evm::FeeCalculator for FixedGasPrice {
    fn min_gas_price() -> U256 {
        // TODO check the min_gas_price implementation in moonbeam
        // (1 * currency::GIGAWEI * currency::SUPPLY_FACTOR).into()
        0.into()
    }
}

/// And implementation of Frontier's AddressMapping trait for Moonbeam Accounts.
/// This is basically identical to Frontier's own IdentityAddressMapping, but it works for any type
/// that is Into<H160> like AccountId20 for example.
pub struct IntoAddressMapping;
impl<T: From<H160>> pallet_evm::AddressMapping<T> for IntoAddressMapping {
    fn into_account_id(address: H160) -> T {
        address.into()
    }
}

/// From *** MOONBEAM ***
/// Current approximation of the gas/s consumption considering
/// EVM execution over compiled WASM (on 4.4Ghz CPU).
/// Given the 500ms Weight, from which 75% only are used for transactions,
/// the total EVM execution gas limit is: GAS_PER_SECOND * 0.500 * 0.75 ~= 15_000_000.
pub const GAS_PER_SECOND: u64 = 40_000_000;
/// Approximate ratio of the amount of Weight per Gas.
/// u64 works for approximations because Weight is a very small unit compared to gas.
pub const WEIGHT_PER_GAS: u64 = WEIGHT_PER_SECOND / GAS_PER_SECOND;

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
