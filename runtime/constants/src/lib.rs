//! A set of constant values used in amax-eva runtime.

#![cfg_attr(not(feature = "std"), no_std)]

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

    // /// The existential deposit.
    // pub const EXISTENTIAL_DEPOSIT: Balance = 1 * CENTS;

    pub const UNITS: Balance = 1_000_000_000_000;
    pub const CENTS: Balance = UNITS / 30_000;
    pub const GRAND: Balance = CENTS * 100_000;
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
    use sp_runtime::Perbill;

    /// The block saturation level. Fees will be updates based on this value.
    pub const TARGET_BLOCK_FULLNESS: Perbill = Perbill::from_percent(25);

    /// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
    /// node's balance type.
    pub type WeightToFee = IdentityFee<Balance>;
}

/// EVM-related constants.
pub mod evm {
    use frame_support::weights::constants::WEIGHT_PER_SECOND;

    /// From ** MOONBEAM **
    /// Current approximation of the gas/s consumption considering
    /// EVM execution over compiled WASM (on 4.4Ghz CPU).
    /// Given the 500ms Weight, from which 75% only are used for transactions,
    /// the total EVM execution gas limit is: GAS_PER_SECOND * 0.500 * 0.75 ~= 15_000_000.
    pub const GAS_PER_SECOND: u64 = 40_000_000;

    /// Approximate ratio of the amount of Weight per Gas.
    /// u64 works for approximations because Weight is a very small unit compared to gas.
    pub const WEIGHT_PER_GAS: u64 = WEIGHT_PER_SECOND / GAS_PER_SECOND;
}
