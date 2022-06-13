//! A set of constant values used in amax-eva runtime.

#![cfg_attr(not(feature = "std"), no_std)]
use runtime_common::constants::ConstU32;
pub use runtime_common::constants::{balances, currency, evm, fee, system, time};

/// Governance constants.
pub mod governance {
    use super::*;
    pub use runtime_common::constants::governance::{MaxProposals, TechnicalMaxMembers};
    use runtime_common::constants::time::DAYS;

    /// The maximum amount of time (in blocks) for technical committee members to vote on motions.
    /// Motions may end in fewer blocks if enough votes are cast to determine the result.
    pub type MotionDuration = ConstU32<{ 7 * DAYS }>;
}

/// Consensus constants.
pub mod consensus {
    use super::*;
    pub use runtime_common::constants::consensus::MaxAuthorities;
    use runtime_common::constants::time::HOURS;

    pub type Period = ConstU32<{ 4 * HOURS }>;
}
