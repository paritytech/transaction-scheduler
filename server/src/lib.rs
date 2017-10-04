#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate futures;
#[macro_use]
extern crate log;

extern crate ethcore_bigint;
extern crate futures_cpupool;
extern crate jsonrpc_core;
extern crate jsonrpc_http_server;
extern crate parking_lot;
// extern crate parity_rpc;
extern crate rustc_hex;
extern crate serde;
extern crate serde_json;
extern crate web3;

pub mod blockchain;
pub mod database;
pub mod server;
pub mod submitter;

mod types;
mod verifier;

/// Type of the transport to instantiate.
#[derive(Debug, Clone)]
pub enum TransportType {
    /// IPC (local) transport
    Ipc(String),
    /// HTTP transport (can be remote)
    Http(String),
}
