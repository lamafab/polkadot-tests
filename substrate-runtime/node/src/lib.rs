#[macro_use]
extern crate serde;

mod builder;
mod chain_spec;
mod cli;
mod command;
mod executor;
mod primitives;
mod tool_spec;

pub use command::run;

pub type Result<T> = std::result::Result<T, failure::Error>;
