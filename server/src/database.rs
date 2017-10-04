use std::io;
use std::path::{Path, PathBuf};
use std::collections::{HashSet, BTreeMap};

use types::{BlockNumber, Transaction, Address};
use parking_lot::RwLock;

error_chain! {
    foreign_links {
        Io(io::Error);
    }
}

/// A storage for scheduled transactions.
/// Each block has a separate instance of `BlockDatabase`.
///
/// The database should store only valid transactions.
pub struct Database {
    path: PathBuf,
    blocks: RwLock<BTreeMap<BlockNumber, BlockDatabase>>,
    senders: RwLock<HashSet<Address>>,
}

impl Database {
    pub fn open<T: AsRef<Path>>(path: T) -> Self {
        // TODO Re-open all existing block database that are found
        Database {
            path: path.as_ref().to_owned(),
            blocks: Default::default(),
            senders: Default::default(),
        }
    }

    pub fn insert(&self, block_number: BlockNumber, transaction: Transaction) -> Result<()> {
        let path = self.path.clone();
        let mut blocks = self.blocks.write();
        let mut db = blocks.entry(block_number).or_insert_with(|| BlockDatabase::new(&path, block_number));
        db.insert(transaction)
    }

    pub fn has(&self, block_number: &BlockNumber) -> bool {
        self.blocks.read().contains_key(block_number)
    }

    pub fn drain(&self, block_number: &BlockNumber) -> Option<BlockDatabase> {
        self.blocks.write().remove(block_number)
    }
}

/// A set of transactions to execute at particular block.
pub struct BlockDatabase {
    path: PathBuf,
}

impl BlockDatabase {
    pub fn new<T: AsRef<Path>>(path: T, block_number: BlockNumber) -> Self {
        let path = path.as_ref();

        // TODO [ToDr] Open or mmap the file.
        BlockDatabase {
            path: path.join(format!("{}.txs", block_number)),
        }
    }

    pub fn insert(&mut self, transaction: Transaction) -> Result<()> {
        unimplemented!()
    }
}

impl IntoIterator for BlockDatabase {
    type Item = Transaction;
    type IntoIter = TransactionsIterator;

    fn into_iter(self) -> Self::IntoIter {
        TransactionsIterator {
            db: self,
        }
    }
}

pub struct TransactionsIterator {
    db: BlockDatabase,
}

impl Iterator for TransactionsIterator {
    type Item = Transaction;

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}
