//! The store to adjust write/read permissions for DDC Buckets

use crate::ddc_bucket::{AccountId, BucketId, Result};

use ink_storage::{
    collections::{
        HashMap as StorageHashMap,
    },
    traits,
};

#[derive(traits::SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout, Debug))]
pub struct BucketsPermsStore {
    writers: StorageHashMap<(BucketId, AccountId), bool>,
    readers: StorageHashMap<(BucketId, AccountId), bool>
}

impl BucketsPermsStore {
    pub fn new() -> BucketsPermsStore {
        BucketsPermsStore {
            writers: Default::default(),
            readers: Default::default()
        }
    }

    // get permission for bucket writer 
    pub fn get_permission_writer(&self, key: (BucketId, AccountId)) -> bool {
        *self.writers.get(&key).unwrap()
    }

    // // get bucket writer 
    // pub fn get_bucket_writers(&self, bucket: BucketId) -> Vec<AccountId> {
    //     *self.writers.get(&key).unwrap()
    // }

    // set permission for bucket writer
    pub fn set_permission_writer(&mut self, key: (BucketId, AccountId), permission: bool) -> Result<()> {
        self.writers.insert(key, permission);
        Ok(())
    }

    // get permission for bucket reader 
    pub fn get_permission_reader(&self, key: (BucketId, AccountId)) -> bool {
        *self.readers.get(&key).unwrap()
    }

    // // get bucket readers 
    // pub fn get_bucket_readers(&self, bucket: BucketId) -> Vec<AccountId> {
    //     *self.writers.get(&key).unwrap()
    // }

    // set permission for bucket reader
    pub fn set_permission_reader(&mut self, key: (BucketId, AccountId), permission: bool) -> Result<()> {
        self.readers.insert(key, permission);
        Ok(())
    }
}
