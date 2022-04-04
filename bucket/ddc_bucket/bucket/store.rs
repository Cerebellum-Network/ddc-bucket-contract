//! The store to create and access Buckets.

use ink_storage::{
    collections::Vec as InkVec,
    traits,
};

use crate::ddc_bucket::{AccountId, Error::*, Result};
use crate::ddc_bucket::cluster::entity::ClusterId;
use crate::ddc_bucket::flow::Flow;
use crate::ddc_bucket::schedule::Schedule;

use super::entity::{Bucket, BucketId, BucketParams};

#[derive(traits::SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout, Debug))]
pub struct BucketStore(pub InkVec<Bucket>);

impl BucketStore {
    #[must_use]
    pub fn create(&mut self, owner_id: AccountId, bucket_params: BucketParams, cluster_id: ClusterId) -> (BucketId, usize) {
        let bucket = Bucket {
            owner_id,
            cluster_id,
            flow: Flow { from: owner_id, schedule: Schedule::empty() },
            bucket_params,
            resource_reserved: 0,
        };

        let record_size = bucket.new_size();
        let bucket_id = self.0.len();
        self.0.push(bucket);

        (bucket_id, record_size)
    }

    pub fn get(&self, bucket_id: BucketId) -> Result<&Bucket> {
        self.0.get(bucket_id).ok_or(BucketDoesNotExist)
    }

    pub fn get_mut(&mut self, bucket_id: BucketId) -> Result<&mut Bucket> {
        self.0.get_mut(bucket_id).ok_or(BucketDoesNotExist)
    }
}
