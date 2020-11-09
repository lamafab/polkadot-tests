#[macro_use]
extern crate serde;

mod builder;
mod chain_spec;
mod executor;
mod tool_spec;
mod primitives;
mod command;
mod cli;

pub use command::run;

pub type Result<T> = std::result::Result<T, failure::Error>;
