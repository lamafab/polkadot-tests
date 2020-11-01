//! Substrate Node Template CLI library.
#![warn(missing_docs)]
#[macro_use]
extern crate serde;

mod chain_spec;
#[macro_use]
mod service;
mod cli;
mod command;
mod rpc;
mod builder;
mod executor;

type Result<T> = std::result::Result<T, failure::Error>;

fn main() -> sc_cli::Result<()> {
    command::run()
}
