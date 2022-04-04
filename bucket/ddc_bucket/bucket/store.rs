//! The store to create and access Buckets.

use ink_storage::{
    collections::Vec as InkVec,
    traits,
};

use crate::ddc_bucket::{AccountId, Error::*, Result};
use crate::ddc_bucket::cluster::entity::ClusterId;
use crate::ddc_bucket::contract_fee::SIZE_VEC;
use crate::ddc_bucket::flow::Flow;
use crate::ddc_bucket::schedule::Schedule;

use super::entity::{Bucket, BucketId, BucketParams};

#[derive(traits::SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout, Debug))]
pub struct BucketStore {
    pub buckets: InkVec<Bucket>,
    pub params: InkVec<BucketParams>,
}

impl BucketStore {
    #[must_use]
    pub fn create(&mut self, owner_id: AccountId, bucket_params: BucketParams, cluster_id: ClusterId) -> (BucketId, usize) {
        let bucket = Bucket {
            owner_id,
            cluster_id,
            flow: Flow { from: owner_id, schedule: Schedule::empty() },
            resource_reserved: 0,
        };

        let record_size = Bucket::RECORD_SIZE + SIZE_VEC + bucket_params.len();
        let bucket_id = self.buckets.len();
        assert_eq!(bucket_id, self.params.len());

        self.buckets.push(bucket);
        self.params.push(bucket_params);

        (bucket_id, record_size)
    }

    pub fn get(&self, bucket_id: BucketId) -> Result<&Bucket> {
        self.buckets.get(bucket_id).ok_or(BucketDoesNotExist)
    }

    pub fn get_mut(&mut self, bucket_id: BucketId) -> Result<&mut Bucket> {
        self.buckets.get_mut(bucket_id).ok_or(BucketDoesNotExist)
    }

    pub fn get_params(&self, bucket_id: BucketId) -> Result<&BucketParams> {
        self.params.get(bucket_id).ok_or(BucketDoesNotExist)
    }
}
