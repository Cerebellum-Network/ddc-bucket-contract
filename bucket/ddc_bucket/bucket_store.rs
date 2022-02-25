use ink_prelude::{
    vec, vec::Vec,
};
use ink_storage::{
    collections::{HashMap, hashmap::Entry::*},
    collections::Stash,
    collections::Vec as InkVec,
    traits::{PackedLayout, SpreadLayout, StorageLayout},
};

use crate::ddc_bucket::{AccountId, Balance, Error::*, Result};

use super::bucket::{Bucket, BucketId, BucketParams, BucketStatus};

#[derive(SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(StorageLayout, Debug))]
pub struct BucketStore(pub InkVec<Bucket>);

impl BucketStore {
    pub fn get(&self, bucket_id: BucketId) -> Result<Bucket> {
        self.0.get(bucket_id).cloned()
            .ok_or(BucketDoesNotExist)
    }
}
