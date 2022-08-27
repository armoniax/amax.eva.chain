//! Armonia Eva Node CLI library.

#![warn(missing_docs)]
#![allow(clippy::type_complexity, clippy::too_many_arguments, clippy::needless_return)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
mod chain_spec;
mod cli;
mod client;
mod command;
#[cfg(feature = "manual-seal")]
mod manual_seal;
mod rpc;
mod service;
mod tracing;

pub use self::{cli::*, command::*};
pub use sc_cli::{Error, Result};

use client::{FullBackend, FullClient};
