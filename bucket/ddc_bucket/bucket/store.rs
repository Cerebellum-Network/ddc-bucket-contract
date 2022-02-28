use ink_prelude::vec::Vec;
use ink_storage::{
    collections::Vec as InkVec,
    traits,
};

use crate::ddc_bucket::{AccountId, Error::*, Result};

use super::entity::{Bucket, BucketId, BucketParams};

#[derive(traits::SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout, Debug))]
pub struct BucketStore(pub InkVec<Bucket>);

impl BucketStore {
    pub fn create(&mut self, owner_id: AccountId, bucket_params: BucketParams) -> BucketId {
        let bucket = Bucket {
            owner_id,
            cluster_ids: Vec::new(),
            deal_ids: Vec::new(),
            bucket_params,
        };
        let bucket_id = self.0.len();
        self.0.push(bucket);
        bucket_id
    }

    pub fn get(&self, bucket_id: BucketId) -> Result<&Bucket> {
        self.0.get(bucket_id).ok_or(BucketDoesNotExist)
    }

    pub fn get_mut(&mut self, bucket_id: BucketId) -> Result<&mut Bucket> {
        self.0.get_mut(bucket_id).ok_or(BucketDoesNotExist)
    }
}
