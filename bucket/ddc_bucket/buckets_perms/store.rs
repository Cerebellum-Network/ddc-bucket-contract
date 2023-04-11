//! The store to adjust write/read permissions for DDC Buckets
use crate::ddc_bucket::{AccountId, BucketId, Result};
use ink_prelude::vec::Vec;
use ink_storage::Mapping;


#[ink::storage_item]
#[derive(Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct BucketsPermsStore {
    writers: Mapping<BucketId, Vec<AccountId>>,
    readers: Mapping<BucketId, Vec<AccountId>>
}

impl BucketsPermsStore {
    pub fn new() -> BucketsPermsStore {
        BucketsPermsStore {
            writers: Default::default(),
            readers: Default::default()
        }
    }

    // get accounts with permission for bucket writing
    pub fn get_bucket_writers(&self, key: BucketId) -> Vec<AccountId> {
        let writers : Vec<AccountId> = self.writers.get(&key)
            .unwrap_or(Vec::new())
            .iter()
            .cloned()
            .collect();
        return writers;
    }

    // grant permission for bucket writing for some account
    pub fn grant_writer_permission(&mut self, key: BucketId, writer: AccountId) -> Result<()> {
        if !self.writers.contains(&key) {
            let empty_vec: Vec<AccountId> = Vec::new();
            self.writers.insert(key, &empty_vec);
        }

        let mut writers = self.writers.get(&key).unwrap();
        writers.push(writer);
        self.writers.insert(key, &writers);

        Ok(())
    }

    // revoke permission for bucket writing for some account
    pub fn revoke_writer_permission(&mut self, key: BucketId, writer: AccountId) -> Result<()> {
        let mut writers = self.writers.get(&key).unwrap();
        if let Some(pos) = writers.iter().position(|x| *x == writer) {
            writers.remove(pos);
            self.writers.insert(key, &writers);
        }

        Ok(())
    }

    // get accounts with permission for bucket reading
    pub fn get_bucket_readers(&self, key: BucketId) -> Vec<AccountId> {
        let readers = self.readers.get(&key)
            .unwrap_or(Vec::new())
            .iter()
            .cloned()
            .collect();

        return readers;
    }

    // grant permission for bucket reading for some account
    pub fn grant_reader_permission(&mut self, key: BucketId, reader: AccountId) -> Result<()> {
        if !self.readers.contains(&key) {
            let empty_vec: Vec<AccountId> = Vec::new();
            self.readers.insert(key, &empty_vec);
        }

        let mut readers = self.readers.get(&key).unwrap();
        readers.push(reader);
        self.readers.insert(key, &readers);

        Ok(())
    }

    // revoke permission for bucket writing for some account
    pub fn revoke_reader_permission(&mut self, key: BucketId, reader: AccountId) -> Result<()> {
        let mut readers = self.readers.get(&key).unwrap();

        if let Some(pos) = readers.iter().position(|x| *x == reader) {
            readers.remove(pos);
            self.readers.insert(key, &readers);
        }

        Ok(())
    }

}
