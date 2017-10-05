use std::collections::btree_map::Entry;
use std::collections::{HashSet, BTreeMap};
use std::io::{Read, Write, Seek};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{io, fs};

use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};
use parking_lot::RwLock;

use types::{BlockNumber, Transaction, Address};

error_chain! {
    foreign_links {
        Io(io::Error);
    }
    errors {
        SenderExists {
            description("Sender already scheduled.")
            display("Sender already scheduled.")
        }
    }
}

/// A storage for scheduled transactions.
/// Each block has a separate instance of `BlockDatabase`.
///
/// The database should store only valid transactions.
#[derive(Debug)]
pub struct Database {
    path: PathBuf,
    senders: Arc<RwLock<HashSet<Address>>>,
    blocks: RwLock<BTreeMap<BlockNumber, BlockDatabase>>,
}

impl Database {
    pub fn open<T: AsRef<Path>>(path: T) -> Result<Self> {
        // TODO Re-open all existing block database that are found
        Ok(Database {
            path: path.as_ref().to_owned(),
            blocks: Default::default(),
            senders: Default::default(),
        })
    }

    pub fn has_sender(&self, sender: &Address) -> bool {
        self.senders.read().contains(sender)
    }

    pub fn insert(&self, block_number: BlockNumber, transaction: Transaction) -> Result<()> {
        if self.senders.read().contains(&transaction.sender()) {
            trace!("[{:?}] Rejecting because sender is already in db.", transaction.hash());
            return Err(ErrorKind::SenderExists.into());
        }

        let mut senders = self.senders.write();
        senders.insert(*transaction.sender());
        let mut blocks = self.blocks.write();

        match blocks.entry(block_number) {
            Entry::Vacant(vacant) => {
                let path = self.path.clone();
                let db = BlockDatabase::new(&path, block_number)?;
                vacant.insert(db).insert(transaction)
            },
            Entry::Occupied(ref mut db) => db.get_mut().insert(transaction),
        }
    }

    pub fn has(&self, block_number: &BlockNumber) -> bool {
        self.blocks.read().contains_key(block_number)
    }

    pub fn drain(&self, block_number: &BlockNumber) -> Result<Option<TransactionsIterator>> {
        // TODO [ToDr] Drain all blocks below current.
        match self.blocks.write().remove(block_number) {
            Some(block) => Ok(Some(block.drain(self.senders.clone())?)),
            None => Ok(None),
        }
    }
}

/// A set of transactions to execute at particular block.
#[derive(Debug)]
struct BlockDatabase {
    path: PathBuf,
    file: fs::File,
}

impl BlockDatabase {
    pub fn new<T: AsRef<Path>>(path: T, block_number: BlockNumber) -> Result<Self> {
        let path = path.as_ref();
        let path = path.join(format!("{}.txs", block_number));
        let file = fs::OpenOptions::new()
            .read(true)
            .create(true)
            .append(true)
            .open(&path)?;

        Ok(BlockDatabase {
            path,
            file,
        })
    }

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

    fn drain(mut self, senders: Arc<RwLock<HashSet<Address>>>) -> Result<TransactionsIterator> {
        let path = self.path;
        self.file.seek(io::SeekFrom::Start(0))?;
        let mut content = Vec::new();
        self.file.read_to_end(&mut content)?;

        trace!("Drained transactions from: {} (length: {})", path.display(), content.len());
        let content = io::Cursor::new(content);
        Ok(TransactionsIterator {
            path,
            content,
            senders,
        })
    }
}

pub struct TransactionsIterator {
    path: PathBuf,
    content: io::Cursor<Vec<u8>>,
    senders: Arc<RwLock<HashSet<Address>>>,
}

impl Iterator for TransactionsIterator {
    type Item = Transaction;

    fn next(&mut self) -> Option<Self::Item> {
        if self.content.position() == self.content.get_ref().len() as u64 {
            // TODO [ToDr] Should remove file when drained.
            if let Err(err) = fs::remove_file(&self.path) {
                warn!("Unable to remove processed file at {}: {:?}", self.path.display(), err);
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
                self.senders.write().remove(transaction.sender());
                Some(transaction)
            },
            Err(err) => {
                // TODO [ToDr] Can we recover from that?
                format!("Error reading transaction from db: {:?}", err);
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
        let mut db = BlockDatabase::new(dir.path(), 5).unwrap();
        db.insert(tx(0)).unwrap();
        db.insert(tx(1)).unwrap();
        db.insert(tx(2)).unwrap();

        let mut iter = db.drain(Default::default()).unwrap();
        assert_eq!(iter.next(), Some(tx(0)));
        assert_eq!(iter.next(), Some(tx(1)));
        assert_eq!(iter.next(), Some(tx(2)));
        assert_eq!(iter.next(), None);
    }
}
