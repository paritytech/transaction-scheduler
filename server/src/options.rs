/// Transaction Scheduler Server options.
#[derive(Debug, Clone, PartialEq)]
pub struct Options {
    /// Chain id
    pub chain_id: u64,
    /// Maximal gas of a transaction
    pub max_gas: u64,
    /// Minimal gas price
    pub min_gas_price: u64,
    /// Min number of blocks in future to schedule for.
    pub min_schedule_block: u64,
    /// Max number of blocks in future to schedule for.
    pub max_schedule_block: u64,
    /// Min number of seconds in future to schedule for.
    pub min_schedule_seconds: u64,
    /// Max number of seconds in future to schedule for.
    pub max_schedule_seconds: u64,
    /// JSON-RPC Listening address
    pub rpc_listen_address: ::std::net::SocketAddr,
    /// JSON-RPC Server threads
    pub rpc_server_threads: usize,
    /// Transactions processing threads
    pub processing_threads: usize,
}
