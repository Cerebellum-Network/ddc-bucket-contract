//! The store to adjust write/read permissions for DDC Buckets

use crate::ddc_bucket::{AccountId, BucketId, Result};

use ink_prelude::vec::Vec;
use ink_storage::{
    collections::{
        HashMap as StorageHashMap,
    },
    traits,
};

#[derive(traits::SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout, Debug))]
pub struct BucketsPermsStore {
    writers: StorageHashMap<BucketId, Vec<AccountId>>,
    readers: StorageHashMap<BucketId, Vec<AccountId>>
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
        let writers = (*self.writers.get(&key).unwrap_or(&Vec::new())).clone();
        return writers;
    }

    // grant permission for bucket writing for some account
    pub fn grant_writer_permission(&mut self, key: BucketId, writer: AccountId) -> Result<()> {
        if !self.writers.contains_key(&key) {
            let empty_vec = Vec::new();
            self.writers.insert(key, empty_vec);
        }

        (*self.writers.get_mut(&key).unwrap()).push(writer);
        Ok(())
    }

    // revoke permission for bucket writing for some account
    pub fn revoke_writer_permission(&mut self, key: BucketId, writer: AccountId) -> Result<()> {
        let writers = &mut *self.writers.get_mut(&key).unwrap();

        if let Some(pos) = writers.iter().position(|x| *x == writer) {
            writers.remove(pos);
        }
        Ok(())
    }

    // get accounts with permission for bucket reading
    pub fn get_bucket_readers(&self, key: BucketId) -> Vec<AccountId> {
        let readers = (*self.readers.get(&key).unwrap_or(&Vec::new())).clone();
        return readers;
    }

    // grant permission for bucket reading for some account
    pub fn grant_reader_permission(&mut self, key: BucketId, reader: AccountId) -> Result<()> {
        if !self.readers.contains_key(&key) {
            let empty_vec = Vec::new();
            self.readers.insert(key, empty_vec);
        }

        (*self.readers.get_mut(&key).unwrap()).push(reader);
        Ok(())
    }

    // revoke permission for bucket writing for some account
    pub fn revoke_reader_permission(&mut self, key: BucketId, reader: AccountId) -> Result<()> {
        let readers = &mut *self.readers.get_mut(&key).unwrap();

        if let Some(pos) = readers.iter().position(|x| *x == reader) {
            readers.remove(pos);
        }
        Ok(())
    }

}
