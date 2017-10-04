use std::sync::Arc;
use std::{thread, time};

use futures::{sink, Sink, Future};
use futures::sync::mpsc;
use parking_lot::RwLock;
use web3::{self, Web3, Transport, transports};

use types::BlockNumber;
use TransportType;

/// A structure responsible for maintaining and caching latest blockchain state, like:
/// - latest block number
/// - nonce for particular sender
#[derive(Debug, Default)]
pub struct Blockchain {
    latest_block: RwLock<BlockNumber>,
}

impl Blockchain {
    pub fn latest_block(&self) -> BlockNumber {
        *self.latest_block.read()
    }

    fn update_latest_block(&self, new: BlockNumber) {
        *self.latest_block.write() = new;
    }
}

pub struct Updater {
    blockchain: Arc<Blockchain>,
    listener: sink::Wait<mpsc::Sender<BlockNumber>>,
}

impl Updater {
    pub fn new(blockchain: Arc<Blockchain>) -> (Self, mpsc::Receiver<BlockNumber>) {
        let (listener, rx) = mpsc::channel(1024);
        let listener = listener.wait();
        (Updater { blockchain, listener }, rx)
    }

    pub fn run(mut self, transport: TransportType) -> Result<(), web3::Error> {
        match transport {
            TransportType::Ipc(path) => {
                let (_eloop, ipc) = transports::ipc::Ipc::new(&path)?;
                self.run_internal(ipc)
            },
            TransportType::Http(url) => {
                let (_eloop, http) = transports::http::Http::new(&url)?;
                self.run_internal(http)
            }
        }
        Ok(())
    }

    fn run_internal<T: Transport>(mut self, transport: T) {
        let web3 = Web3::new(transport);
        
        let mut last_block = None;
        let mut update = |block_number, last_block: &mut Option<BlockNumber>| {
            self.blockchain.update_latest_block(block_number);
            *last_block = Some(block_number);
            if let Err(err) = self.listener.send(block_number) {
                error!("Listener died: {:?}", err);
            }
        };

        loop {
            match web3.eth().block_number().wait() {
                Err(err) => {
                    warn!("Cannot fetch latest block: {:?}", err);
                },
                Ok(block_number) => {
                    let block_number = block_number.low_u64();
                    match last_block {
                        Some(block) if block != block_number => update(block_number, &mut last_block),
                        None => update(block_number, &mut last_block),
                        _ => {},
                    }
                },
            }
            thread::sleep(time::Duration::from_millis(100));
        }
    }
}
