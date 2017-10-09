//! Transactions storage

use std::collections::btree_map::Entry;
use std::collections::hash_map;
use std::collections::{HashMap, BTreeMap};
use std::io::{Read, Write, Seek};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{io, fs, mem};

use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};
use parking_lot::RwLock;

use types::{BlockNumber, Transaction, Address};

mod error {
    #![allow(unknown_lints)]
    #![allow(missing_docs)]
    error_chain! {
        foreign_links {
            Io(::std::io::Error);
        }
        errors {
            SenderExists {
                description("Sender already scheduled.")
                display("Sender already scheduled.")
            }
        }
    }
}

pub use self::error::*;

/// A storage for scheduled transactions.
/// Each block has a separate instance of `BlockDatabase`.
///
/// The database should store only valid transactions.
#[derive(Debug)]
pub struct Database {
    path: PathBuf,
    senders: Arc<RwLock<HashMap<Address, usize>>>,
    blocks: RwLock<BTreeMap<BlockNumber, BlockDatabase>>,
    max_txs_per_sender: usize,
}

impl Database {
    const EXT: &'static str = "txs";

    /// Open and load existing database in given directory.
    pub fn open<T: AsRef<Path>>(path: T, max_txs_per_sender: usize) -> Result<Self> {
        fs::create_dir_all(&path)?;
        let mut blocks = BTreeMap::new();
        let mut senders = HashMap::new();

        // Re-open all existing block database that are found
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                continue;
            }
            let extension = path.extension().and_then(|s| s.to_str());
            if let Some(Self::EXT) = extension {
                let file_stem = path.file_stem().and_then(|s| s.to_str()).and_then(|s| s.parse().ok());
                if let Some(number) = file_stem {
                    match BlockDatabase::open(&path, &mut senders) {
                        Ok(block) => {
                            blocks.insert(number, block);
                        },
                        Err(err) => {
                            warn!("Ignoring invalid db file at {}: {:?}", path.display(), err);
                        }
                    }
                }
            }
        }

        Ok(Database {
            path: path.as_ref().to_owned(),
            senders: Arc::new(RwLock::new(senders)),
            blocks: RwLock::new(blocks),
            max_txs_per_sender,
        })
    }

    /// Returns number of transactions already scheduled from given sender.
    pub fn sender_allowed(&self, sender: &Address) -> bool {
        *self.senders.read().get(sender).unwrap_or(&0) < self.max_txs_per_sender
    }

    /// Inserts new transactions to the store.
    pub fn insert(&self, block_number: BlockNumber, transaction: Transaction) -> Result<()> {
        if !self.sender_allowed(transaction.sender()) {
            trace!("[{:?}] Rejecting because sender already has too many transactions in db.", transaction.hash());
            return Err(ErrorKind::SenderExists.into());
        }

        let mut senders = self.senders.write();
        match senders.entry(*transaction.sender()) {
            hash_map::Entry::Vacant(entry) => {
                entry.insert(1);
            },
            hash_map::Entry::Occupied(mut entry) => {
                *entry.get_mut() += 1;
            },
        }
        let mut blocks = self.blocks.write();

        match blocks.entry(block_number) {
            Entry::Vacant(vacant) => {
                let path = self.path.join(format!("{}.{}", block_number, Self::EXT));
                let db = BlockDatabase::new(&path)?;
                vacant.insert(db).insert(transaction)
            },
            Entry::Occupied(ref mut db) => db.get_mut().insert(transaction),
        }
    }

    /// Returns true if there are any transactions scheduled for given block.
    pub fn has(&self, block_number: &BlockNumber) -> bool {
        match self.blocks.read().keys().next() {
            Some(b) if b <= block_number => true,
            _ => false,
        }
    }

    /// Drains transactions scheduled for submission up to given block number.
    pub fn drain(&self, block_number: BlockNumber) -> Result<Option<TransactionsIterator>> {
        let blocks = {
            let mut blocks = self.blocks.write();
            let mut new = blocks.split_off(&(block_number + 1));
            mem::swap(&mut *blocks, &mut new);
            new
        };
        let mut it = blocks.into_iter();
        let mut tx_it = match it.next() {
            None => return Ok(None),
            Some((num, block)) => {
                debug!("Draining transactions for block: {}", num);
                block.drain(self.senders.clone())?
            }
        };
        for (num, block) in it {
            debug!("Draining transactions for block: {}", num);
            tx_it.append(block.drain(self.senders.clone())?);
        }

        Ok(Some(tx_it))
    }
}

/// A set of transactions to execute at particular block.
#[derive(Debug)]
struct BlockDatabase {
    path: PathBuf,
    file: fs::File,
}

impl BlockDatabase {
    /// Open existing transactions store and load senders to given `HashMap`.
    pub fn open<T: AsRef<Path>>(path: T, senders: &mut HashMap<Address, usize>) -> Result<Self> {
        let mut file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)?;

        trace!("Reading transactions from: {}", path.as_ref().display());
        let mut it = TransactionsIterator::from_file(&mut file, IteratorMode::ReadOnly)?;
        while let Some(tx) = it.next() {
            trace!("Populating sender: {}", tx.sender());
            match senders.entry(*tx.sender()) {
                hash_map::Entry::Vacant(entry) => {
                    entry.insert(1);
                },
                hash_map::Entry::Occupied(mut entry) => {
                    *entry.get_mut() += 1;
                },
            }
        }
        file.seek(io::SeekFrom::Start(0))?;

        Ok(BlockDatabase {
            path: path.as_ref().to_owned(),
            file,
        })
    }

    /// Creates new transactions store.
    pub fn new<T: AsRef<Path>>(path: T) -> Result<Self> {
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;

        Ok(BlockDatabase {
            path: path.as_ref().to_owned(),
            file,
        })
    }

    /// Inserts new transaction to the store.
    pub fn insert(&mut self, transaction: Transaction) -> Result<()> {
        trace!("[{:?}] Inserting to db.", transaction.hash());
        let rlp_len = transaction.rlp().len();
        let mut vec = Vec::with_capacity(4 + 20 + 32 + rlp_len);
        vec.write_u32::<LittleEndian>(rlp_len as u32)?;
        vec.extend_from_slice(&**transaction.sender());
        vec.extend_from_slice(&**transaction.hash());
        vec.extend_from_slice(transaction.rlp());

        self.file.write_all(&vec)?;
        self.file.flush()?;
        Ok(())
    }

    fn drain(mut self, senders: Arc<RwLock<HashMap<Address, usize>>>) -> Result<TransactionsIterator> {
        self.file.seek(io::SeekFrom::Start(0))?;
        trace!("Draining transactions from: {}", self.path.display());
        TransactionsIterator::from_file(&mut self.file, IteratorMode::Drain(senders, vec![self.path]))
    }
}

/// Iteration mode
pub enum IteratorMode {
    /// Remove the files after iterator is drained and remove senders from the set.
    Drain(Arc<RwLock<HashMap<Address, usize>>>, Vec<PathBuf>),
    /// Only read transactions.
    ReadOnly,
}

/// Transactions iterator
pub struct TransactionsIterator {
    content: io::Cursor<Vec<u8>>,
    mode: IteratorMode,
}

impl TransactionsIterator {
    /// Iterate over transactions from given file.
    pub fn from_file(file: &mut fs::File, mode: IteratorMode) -> Result<Self> {
        let mut content = Vec::new();
        file.read_to_end(&mut content)?;

        let content = io::Cursor::new(content);
        Ok(TransactionsIterator {
            content,
            mode,
        })
    }

    /// Join two iterators together.
    /// Both files will be removed if both iterators are in `Drain` mode.
    pub fn append(&mut self, other: Self) {
        if let IteratorMode::Drain(_, ref mut paths) = self.mode {
            if let IteratorMode::Drain(_, other_paths) = other.mode {
                paths.extend_from_slice(&other_paths);
            }
        }
        self.content.get_mut().extend_from_slice(&other.content.into_inner())
    }
}

impl Iterator for TransactionsIterator {
    type Item = Transaction;

    fn next(&mut self) -> Option<Self::Item> {
        if self.content.position() == self.content.get_ref().len() as u64 {
            if let IteratorMode::Drain(_, ref paths) = self.mode {
                for path in paths {
                    let mut new = path.clone();
                    new.set_extension("old");

                    if let Err(err) = fs::rename(&path, new) {
                        warn!("Unable to rename processed file at {}: {:?}", path.display(), err);
                    }
                }
            }
            return None;
        }

        let read_transaction = |mut content: &mut io::Cursor<_>| -> Result<_> {
            let mut sender = [0u8; 20];
            let mut hash = [0u8; 32];
            let rlp_len = content.read_u32::<LittleEndian>()? as usize;
            let mut rlp = Vec::with_capacity(rlp_len);
            rlp.resize(rlp_len, 0);
            content.read_exact(&mut sender)?;
            content.read_exact(&mut hash)?;
            content.read_exact(&mut rlp)?;
            Ok(Transaction::new(sender.into(), hash.into(), rlp))
        };

        match read_transaction(&mut self.content) {
            Ok(transaction) => {
                if let IteratorMode::Drain(ref senders, _) = self.mode {
                    if let hash_map::Entry::Occupied(mut entry) = senders.write().entry(*transaction.sender()) {
                        if entry.get() > &1 {
                            *entry.get_mut() -= 1;
                        } else {
                            entry.remove();
                        }
                    }
                }
                Some(transaction)
            },
            Err(err) => {
                // TODO [ToDr] Can we recover from that?
                warn!("Error reading transaction from db: {:?}", err);
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use ethcore::transaction::SignedTransaction;
    use rlp::UntrustedRlp;
    use rustc_hex::FromHex;
    use tempdir::TempDir;
    use super::*;

    fn tx(id: u64) -> Transaction {
        let tx = match id {
            0 => "f864808504a817c800825208943535353535353535353535353535353535353535808025a0044852b2a670ade5407e78fb2863c51de9fcb96542a07186fe3aeda6bb8a116da0044852b2a670ade5407e78fb2863c51de9fcb96542a07186fe3aeda6bb8a116d",
            1 => "f864018504a817c80182a410943535353535353535353535353535353535353535018025a0489efdaa54c0f20c7adf612882df0950f5a951637e0307cdcb4c672f298b8bcaa0489efdaa54c0f20c7adf612882df0950f5a951637e0307cdcb4c672f298b8bc6",
            2 => "f864028504a817c80282f618943535353535353535353535353535353535353535088025a02d7c5bef027816a800da1736444fb58a807ef4c9603b7848673f7e3a68eb14a5a02d7c5bef027816a800da1736444fb58a807ef4c9603b7848673f7e3a68eb14a5",
            3 => "f865038504a817c803830148209435353535353535353535353535353535353535351b8025a02a80e1ef1d7842f27f2e6be0972bb708b9a135c38860dbe73c27c3486c34f4e0a02a80e1ef1d7842f27f2e6be0972bb708b9a135c38860dbe73c27c3486c34f4de",
            _ => panic!("Unknown id."),
        };
        let transaction = FromHex::from_hex(tx).unwrap();
        let rlp = UntrustedRlp::new(&transaction).as_val().unwrap();
        SignedTransaction::new(rlp).unwrap().into()
    }

    #[test]
    fn should_save_transactions_to_disk() {
        let dir = TempDir::new("db1").unwrap();
        let mut db = BlockDatabase::new(dir.path().join("test.txs")).unwrap();
        db.insert(tx(0)).unwrap();
        db.insert(tx(1)).unwrap();
        db.insert(tx(2)).unwrap();

        let mut iter = db.drain(Default::default()).unwrap();
        assert_eq!(iter.next(), Some(tx(0)));
        assert_eq!(iter.next(), Some(tx(1)));
        assert_eq!(iter.next(), Some(tx(2)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn should_save_transaction() {
        let dir = TempDir::new("db1").unwrap();
        let db = Database::open(dir.path(), 2).unwrap();
        db.insert(5, tx(0)).unwrap();
        db.insert(3, tx(1)).unwrap();
        db.insert(3, tx(2)).unwrap();
        db.insert(6, tx(0)).unwrap();
        // This should be an error, cause there is already a transaction from the same sender.
        db.insert(6, tx(0)).unwrap_err();

        assert_eq!(db.has(&2), false);
        assert_eq!(db.has(&3), true);
        assert_eq!(db.has(&4), true);
        assert_eq!(db.has(&5), true);
        assert_eq!(db.has(&6), true);

        let mut iter = db.drain(5).unwrap().unwrap();
        assert_eq!(iter.next(), Some(tx(1)));
        assert_eq!(iter.next(), Some(tx(2)));
        assert_eq!(iter.next(), Some(tx(0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn should_restore_db_from_disk() {
        let dir = TempDir::new("db1").unwrap();
        {
            let db = Database::open(dir.path(), 1).unwrap();
            db.insert(5, tx(0)).unwrap();
            db.insert(3, tx(1)).unwrap();
            db.insert(3, tx(2)).unwrap();
        }

        let db = Database::open(dir.path(), 1).unwrap();
        let mut iter = db.drain(5).unwrap().unwrap();
        assert_eq!(iter.next(), Some(tx(1)));
        assert_eq!(iter.next(), Some(tx(2)));
        assert_eq!(iter.next(), Some(tx(0)));
        assert_eq!(iter.next(), None);
    }
}
