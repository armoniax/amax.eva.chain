//! A set of constant values used in substrate runtime.

#![cfg_attr(not(feature = "std"), no_std)]

/// System constants.
pub mod system {
    use frame_support::weights::{constants::WEIGHT_PER_SECOND, Weight};
    use sp_runtime::Perbill;

    /// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used
    /// by  Operational  extrinsics.
    pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
    /// We allow for 2 seconds of compute with a 6 second average block time.
    // TODO [discuss]need to change to a suitable value.
    pub const MAXIMUM_BLOCK_WEIGHT: Weight = 2 * WEIGHT_PER_SECOND;
    /// The maximum block length is 5 MiB.
    pub const MAXIMUM_BLOCK_LENGTH: u32 = 5 * 1024 * 1024;
}

/// Time constants.
pub mod time {
    use primitives_core::{BlockNumber, Moment};

    /// This determines the average expected block time that we are targeting.
    /// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
    /// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
    /// up by `pallet_aura` to implement `fn slot_duration()`.
    ///
    /// Change this to adjust the block time.
    pub const MILLISECS_PER_BLOCK: Moment = 2000;
    pub const SECS_PER_BLOCK: Moment = MILLISECS_PER_BLOCK / 1000;

    // NOTE: Currently it is not possible to change the slot duration after the chain has started.
    //       Attempting to do so will brick block production.
    pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;

    // Time is measured by number of blocks.
    pub const MINUTES: BlockNumber = 60 / (SECS_PER_BLOCK as BlockNumber);
    pub const HOURS: BlockNumber = MINUTES * 60;
    pub const DAYS: BlockNumber = HOURS * 24;
}

/// Currency constants.
pub mod currency {
    use primitives_core::Balance;

    // The decimal for 1 token is 18.
    pub const UNITS: Balance = 1_000_000_000_000_000_000;
    // we assume one unit value to a dollars.
    pub const DOLLARS: Balance = UNITS;
    pub const CENTS: Balance = DOLLARS / 100;
    pub const MILLICENTS: Balance = CENTS / 1_000;

    // TODO: need to design
    pub const fn deposit(items: u32, bytes: u32) -> Balance {
        items as Balance * 15 * CENTS + (bytes as Balance) * 6 * CENTS
    }
}

/// Fee-related constants.
pub mod fee {
    use frame_support::weights::IdentityFee;
    use primitives_core::Balance;

    /// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
    /// node's balance type.
    pub type WeightToFee = IdentityFee<Balance>;
}

/// EVM-related constants.
pub mod evm {
    use frame_support::weights::{constants::WEIGHT_PER_SECOND, Weight};

    /// From ** MOONBEAM **
    /// Current approximation of the gas/s consumption considering
    /// EVM execution over compiled WASM (on 4.4Ghz CPU).
    /// Given the 500ms Weight, from which 75% only are used for transactions,
    /// the total EVM execution gas limit is: GAS_PER_SECOND * 0.500 * 0.75 ~= 15_000_000.
    pub const GAS_PER_SECOND: Weight = 40_000_000;

    /// Approximate ratio of the amount of Weight per Gas.
    /// u64 works for approximations because Weight is a very small unit compared to gas.
    pub const WEIGHT_PER_GAS: Weight = WEIGHT_PER_SECOND / GAS_PER_SECOND;
}
