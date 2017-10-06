//! Blockchain state

use std::collections::HashMap;
use std::sync::Arc;
use std::{fmt, thread, time};

use futures::{sink, future, Sink, Future};
use futures::sync::mpsc;
use parking_lot::RwLock;
use web3::{self, Web3, Transport, contract, transports};
use web3::api::{Eth, Namespace};
use web3::transports::http::Http;

use types::{Address, BlockNumber, U256};
use TransportType;

type BN = (U256, U256);
/// A structure responsible for maintaining and caching latest blockchain state, like:
/// - latest block number
/// - nonce for particular sender
pub struct Blockchain<T: Transport = Arc<Http>> {
    web3: Web3<T>,
    _eloop: transports::EventLoopHandle,
    latest_block: RwLock<BlockNumber>,
    // TODO [ToDr] Caching can lead to OOM. Might be worth to introduce some eviction.
    cached_balance_and_nonce: Arc<RwLock<HashMap<Address, BN>>>,
    cached_certification: Arc<RwLock<HashMap<Address, bool>>>,
    certifier: Option<contract::Contract<T>>,
}

impl<T: Transport> fmt::Debug for Blockchain<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Blockchain")
            .field("latest_block", &self.latest_block)
            .field("cached", &self.cached_balance_and_nonce)
            .finish()
    }
}

impl Blockchain {
    /// Create a new cached blockchain client.
    pub fn new(url: &str, certifier: Option<Address>) -> Result<Self, web3::Error> {
        let (_eloop, http) = transports::http::Http::new(url)?;
        let http = Arc::new(http);
        let certifier = certifier.map(|address| {
            let eth = Eth::new(http.clone());
            let address = (*address).into();
            contract::Contract::from_json(eth, address, include_bytes!("./abi/MultiCertifier.json")).expect("Valid ABI provided; qed")
        });

        Ok(Blockchain {
            web3: Web3::new(http),
            _eloop,
            latest_block: Default::default(),
            cached_balance_and_nonce: Default::default(),
            cached_certification: Default::default(),
            certifier,
        })
    }
}

impl<T: Transport> Blockchain<T> where
    T::Out: Send + 'static,
{
    fn update_latest_block(&self, new: BlockNumber) {
        *self.latest_block.write() = new;
        self.cached_balance_and_nonce.write().clear();
        self.cached_certification.write().clear();
    }

    /// Returns current latest block.
    pub fn latest_block(&self) -> BlockNumber {
        *self.latest_block.read()
    }

    /// Queries the blockchain for given sender's balance and nonce.
    pub fn balance_and_nonce(&self, sender: Address) -> Box<Future<Item=BN, Error=web3::Error> + Send> {
        trace!("Fetching balance and nonce for {:?}", sender);
        if let Some(bn) = self.cached_balance_and_nonce.read().get(&sender) {
            trace!("Returning cached result for {:?} = {:?}", sender, bn);
            return Box::new(future::ok(bn.clone()));
        }

        let address = (*sender).into();
        let balance = self.web3.eth().balance(address, None).map(|balance| (*balance).into());
        let nonce = self.web3.eth().transaction_count(address, None).map(|nonce| (*nonce).into());

        let cbn = self.cached_balance_and_nonce.clone();
        Box::new(balance.join(nonce).map(move |res| {
            trace!("Got balance and nonce for {:?} = {:?}", sender, res);
            cbn.write().insert(sender, res.clone());
            res
        }))
    }

    /// Checks whether address is certified on blockchain.
    pub fn is_certified(&self, sender: Address) -> Box<Future<Item=bool, Error=contract::Error> + Send> {
        trace!("Checking certification status for {:?}", sender);
        let certifier = match self.certifier {
            None => return Box::new(future::ok(true)),
            Some(ref certifier) => certifier,
        };

        if let Some(is_certified) = self.cached_certification.read().get(&sender) {
            trace!("Returning cached result for {:?} = {:?}", sender, is_certified);
            return Box::new(future::ok(*is_certified));
        }

        let address: web3::types::Address = (*sender).into();
        let cc = self.cached_certification.clone();
        Box::new(
            certifier.query("certified", (address, ), None, Default::default(), None).map(move |res: bool| {
            trace!("Got certification status for {:?} = {:?}", sender, res);
              cc.write().insert(sender, res);
              res
            })
        )
    }
}

/// Blockchain updater.
/// Responsible for feeding in latest block number to blockchain structure and to a returned stream.
pub struct Updater {
    blockchain: Arc<Blockchain>,
    listener: sink::Wait<mpsc::Sender<BlockNumber>>,
}

impl Updater {
    /// Creates new blockchain updater.
    pub fn new(blockchain: Arc<Blockchain>) -> (Self, mpsc::Receiver<BlockNumber>) {
        let (listener, rx) = mpsc::channel(16);
        let listener = listener.wait();
        (Updater { blockchain, listener }, rx)
    }

    /// Starts the blockchain updater.
    /// This method will block until indefinitely.
    pub fn run(self, transport: TransportType) -> Result<(), web3::Error> {
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
        info!("Starting blockchain updater.");
        let web3 = Web3::new(transport);
        
        let mut last_block = None;
        let mut update = |block_number, last_block: &mut Option<BlockNumber>| {
            trace!("Updating latest block number: {}", block_number);
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
