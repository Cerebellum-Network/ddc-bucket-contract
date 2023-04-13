//! The store to create and access Buckets.
use ink_prelude::vec::Vec;
use crate::ddc_bucket::{AccountId, Error::*, Result};
use crate::ddc_bucket::cluster::entity::ClusterId;
use crate::ddc_bucket::flow::Flow;
use crate::ddc_bucket::schedule::Schedule;

use super::entity::{Bucket, BucketId};


pub const BUCKET_STORE_KEY: u32 = openbrush::storage_unique_key!(BucketStore);
#[openbrush::upgradeable_storage(BUCKET_STORE_KEY)]
#[derive(Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct BucketStore {
    pub buckets: Vec<Bucket>,
    _reserved: Option<()>
}

impl BucketStore {
    #[must_use]
    pub fn create(&mut self, owner_id: AccountId, cluster_id: ClusterId) -> BucketId {
        let bucket = Bucket {
            owner_id,
            cluster_id,
            flow: Flow { from: owner_id, schedule: Schedule::empty() },
            resource_reserved: 0,
            resource_consumption_cap: 0,
            public_availability: false,
        };
        let bucket_id: BucketId = self.buckets.len().try_into().unwrap();
        self.buckets.push(bucket);
        bucket_id
    }

    pub fn get(&self, bucket_id: BucketId) -> Result<&Bucket> {
        self.buckets.get(bucket_id as usize).ok_or(BucketDoesNotExist)
    }

    pub fn get_mut(&mut self, bucket_id: BucketId) -> Result<&mut Bucket> {
        self.buckets.get_mut(bucket_id as usize).ok_or(BucketDoesNotExist)
    }
}
