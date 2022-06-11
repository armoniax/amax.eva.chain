#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unnecessary_mut_passed)]
#![allow(clippy::too_many_arguments)]

/// Runtime API for Geth debug RPC.
pub mod debug;
/// Runtime API for Geth txpool RPC.
pub mod txpool;

pub use self::{debug::*, txpool::*};

pub trait ConvertDebugTx<E, Moment> {
    fn convert_set_next_block_timestamp_extrinsic(&self, timestamp: Moment) -> E;
}
