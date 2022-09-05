//! Armonia Eva Node CLI library.

#![allow(missing_docs)]
#![allow(clippy::type_complexity, clippy::too_many_arguments)]

pub mod chain_spec;
#[macro_use]
pub mod client;
pub mod cli;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
mod command;
#[cfg(feature = "manual-seal")]
mod manual_seal;
mod rpc;
mod service;
mod tracing;

pub use self::{cli::*, command::*};
pub use sc_cli::{Error, Result};
