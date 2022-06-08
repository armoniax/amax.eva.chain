//! A set of constant values used in substrate runtime.

use frame_support::{parameter_types, traits::ConstU32};
use sp_core::U256;

/// Consensus constants.
pub mod consensus {
    use super::*;
    parameter_types! {
        pub const MaxAuthorities: u32 = 32;
    }
}

/// Currency constants.
pub mod balances {
    use super::*;
    parameter_types! {
        // do not kill accounts when balances low.
        pub const ExistentialDeposit: u128 = 0;
        pub const MaxLocks: u32 = 50;
        pub const OperationalFeeMultiplier: u8 = 5;
    }
}

/// System constructs
pub mod system {
    use super::*;
    pub type MaxConsumers = ConstU32<16>;
}

/// Time constants.
pub mod time {
    use super::*;
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

    parameter_types! {
        pub const MinimumPeriod: Moment = SLOT_DURATION / 2;
    }
}

pub mod ethereum {
    use super::*;
    // TODO need to check gaslimt with team
    parameter_types! {
        pub BlockGasLimit: U256 = U256::from(u32::max_value());

        // TODO need to check this two with team
        pub IsActive: bool = true;
        pub DefaultBaseFeePerGas: U256 = U256::from(1_000_000_000);
    }
}
