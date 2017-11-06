//! Ethereum Transaction Scheduler
//! 
//! Exposes a JSON-RPC `scheduleTransaction(block, rlp)` method
//! that schedules a transaction for submission in some future block.

#![warn(missing_docs)]

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate futures;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

extern crate byteorder;
extern crate ethcore;
extern crate ethcore_bigint;
extern crate futures_cpupool;
extern crate jsonrpc_core;
extern crate jsonrpc_http_server;
extern crate parking_lot;
extern crate rlp;
extern crate rustc_hex;
extern crate serde;
extern crate serde_json;
extern crate time;
extern crate web3;

#[cfg(test)]
extern crate tempdir;
#[cfg(test)]
extern crate env_logger;

pub mod blockchain;
pub mod database;
pub mod server;
pub mod submitter;

mod errors;
mod options;
mod types;
mod verifier;

pub use options::Options;

/// Type of the transport to instantiate.
#[derive(Debug, Clone)]
pub enum TransportType {
    /// IPC (local) transport
    Ipc(String),
    /// HTTP transport (can be remote)
    Http(String),
}
