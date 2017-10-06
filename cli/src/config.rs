#[derive(Debug, Deserialize)]
pub struct Config {
    pub rpc: Rpc,
    pub verification: Verification,
    pub nodes: Nodes,
}

#[derive(Debug, Deserialize)]
pub struct Rpc {
    pub interface: String,
    pub port: u16,
    pub server_threads: usize,
    pub processing_threads: usize,
    pub db_path: String,
    pub submit_earlier: u64,
}

#[derive(Debug, Deserialize)]
pub struct Verification {
    pub chain_id: u64,
    pub max_gas: u64,
    pub min_gas_price: u64,
    pub min_schedule_block: u64,
    pub max_schedule_block: u64,
    pub certifier: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Nodes {
    pub blockchain: String,
    pub transactions: Vec<String>,
}
