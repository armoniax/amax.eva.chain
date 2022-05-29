//! Armonia Eva Node CLI library.
#![warn(missing_docs)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

mod chain_spec;
#[macro_use]
mod service;
mod cli;
mod command;
mod command_helper;
mod key_helper;
mod rpc;

fn main() -> sc_cli::Result<()> {
    command::run()
}
