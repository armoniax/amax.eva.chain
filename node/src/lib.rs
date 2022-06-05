//! Armonia Eva Node CLI library.

#![warn(missing_docs)]
#![allow(clippy::type_complexity, clippy::too_many_arguments)]

mod chain_spec;
mod cli;
mod command;
#[cfg(feature = "runtime-benchmarks")]
mod command_helper;
#[cfg(feature = "manual-seal")]
mod manual_seal;
mod rpc;
mod service;

pub use self::{cli::*, command::*};
pub use sc_cli::{Error, Result};
