#[macro_use]
extern crate serde;

pub mod builder;
pub mod chain_spec;
pub mod tool_spec;
pub mod executor;
pub mod rpc;
pub mod service;

type Result<T> = std::result::Result<T, failure::Error>;
