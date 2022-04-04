//! The public interface to manage params.

use crate::ddc_bucket::{DdcBucket, Result};
use crate::ddc_bucket::bucket::entity::BucketId;
use crate::ddc_bucket::Error::*;

use super::store::Params;

impl DdcBucket {
    pub fn message_bucket_params_get(&self, bucket_id: BucketId) -> Result<Params> {
        self.bucket_params.0.get(bucket_id)
            .cloned().ok_or(BucketDoesNotExist)
    }
}
