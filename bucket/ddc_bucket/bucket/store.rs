use ink_storage::traits::{SpreadAllocate, SpreadLayout};
use ink_prelude::vec::Vec;
use ink_storage::Mapping;
use crate::ddc_bucket::{AccountId, Error::*, Result};
use crate::ddc_bucket::cluster::entity::ClusterId;
use crate::ddc_bucket::flow::Flow;
use crate::ddc_bucket::schedule::Schedule;
use super::entity::{Bucket, BucketId, BucketParams};



#[derive(SpreadAllocate, SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(ink_storage::traits::StorageLayout, Debug))]
pub struct BucketStore {
    pub next_bucket_id: u32,
    pub buckets: Mapping<BucketId, Bucket>,
    pub writers: Mapping<BucketId, Vec<AccountId>>,
    pub readers: Mapping<BucketId, Vec<AccountId>>
}

impl BucketStore {
    #[must_use]
    pub fn create(&mut self, owner_id: AccountId, cluster_id: ClusterId, bucket_params: BucketParams) -> BucketId {
        let bucket_id = self.next_bucket_id;
        self.next_bucket_id = self.next_bucket_id + 1;

        let bucket = Bucket {
            owner_id,
            cluster_id,
            flow: Flow { from: owner_id, schedule: Schedule::empty() },
            resource_reserved: 0,
            resource_consumption_cap: 0,
            public_availability: false,
            bucket_params
        };

        self.buckets.insert(&bucket_id, &bucket);
        bucket_id
    }

    pub fn get(&self, bucket_id: BucketId) -> Result<Bucket> {
        self.buckets.get(bucket_id).ok_or(BucketDoesNotExist)
    }

    pub fn update(&mut self, bucket_id: BucketId, bucket: &Bucket) -> Result<()> {
        if !self.buckets.contains(&bucket_id) {
            Err(BucketDoesNotExist)
        } else {
            self.buckets.insert(bucket_id, bucket);
            Ok(())
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
