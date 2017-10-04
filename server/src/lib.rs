#[macro_use]
extern crate log;

extern crate jsonrpc_core;
extern crate jsonrpc_http_server;

use std::net;

use jsonrpc_core::{Value, IoHandler};
use jsonrpc_http_server::{Server, Error, ServerBuilder};

pub fn start_server(address: &net::SocketAddr, threads: usize) -> Result<Server, Error> {
    let mut io = IoHandler::default();
    io.add_method("say_hello", |_| {
        Ok(Value::String("hello".into()))
    });

    ServerBuilder::new(io)
        .threads(threads)
        .start_http(address)
}
