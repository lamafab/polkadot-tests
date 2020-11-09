#[macro_use]
extern crate serde;

pub mod builder;
pub mod chain_spec;
pub mod executor;
pub mod rpc;
pub mod service;
pub mod tool_spec;

type Result<T> = std::result::Result<T, failure::Error>;
