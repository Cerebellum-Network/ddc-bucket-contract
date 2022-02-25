use ink_lang::{EmitEvent, StaticEnv};
use ink_prelude::{vec, vec::Vec};

use crate::ddc_bucket::{AccountId, BucketCreated, DdcBucket, DealCreated, DealId, DealParams, Result, ServiceId};

use super::entity::{Bucket, BucketId, BucketParams, BucketStatus};

impl DdcBucket {
    pub fn message_bucket_create(&mut self, bucket_params: BucketParams) -> Result<BucketId> {
        let owner_id = Self::env().caller();
        let bucket_id = self.buckets.create(owner_id, bucket_params);
        Self::env().emit_event(BucketCreated { bucket_id, owner_id });
        Ok(bucket_id)
    }

    pub fn message_bucket_add_deal(&mut self, bucket_id: BucketId, service_id: ServiceId, deal_params: DealParams) -> Result<DealId> {
        // Receive the payable value.
        self.deposit()?;
        let owner_id = Self::env().caller();

        let deal_id = self.deal_create(service_id, deal_params)?;

        let bucket = self.buckets.get_mut(bucket_id)?;
        bucket.only_owner(owner_id)?;
        bucket.deal_ids.push(deal_id);

        Self::env().emit_event(DealCreated { deal_id, bucket_id, service_id });
        Ok(deal_id)
    }

    pub fn message_bucket_list_statuses(&self, offset: u32, limit: u32, filter_owner_id: Option<AccountId>) -> (Vec<BucketStatus>, u32) {
        let mut bucket_statuses = Vec::with_capacity(limit as usize);
        for bucket_id in offset..offset + limit {
            let bucket = match self.buckets.0.get(bucket_id) {
                None => break, // No more buckets, stop.
                Some(bucket) => bucket,
            };
            // Apply the filter if given.
            if let Some(owner_id) = filter_owner_id {
                if owner_id != bucket.owner_id {
                    continue; // Skip non-matches.
                }
            }
            // Collect all the details of the bucket.
            match self.bucket_collect_status(bucket_id, bucket.clone()) {
                Err(_) => continue, // Skip on unexpected error.
                Ok(status) =>
                    bucket_statuses.push(status),
            };
        }
        (bucket_statuses, self.buckets.0.len())
    }

    pub fn message_bucket_get_status(&self, bucket_id: BucketId) -> Result<BucketStatus> {
        let bucket = self.bucket_get(bucket_id)?;
        self.bucket_collect_status(bucket_id, bucket)
    }

    pub fn bucket_collect_status(&self, bucket_id: BucketId, bucket: Bucket) -> Result<BucketStatus> {
        let writer_ids = vec![bucket.owner_id];

        let mut deal_statuses = Vec::with_capacity(bucket.deal_ids.len());
        for deal_id in bucket.deal_ids.iter() {
            deal_statuses.push(self.deal_get_status(*deal_id)?);
        }

        Ok(BucketStatus { bucket_id, bucket, writer_ids, deal_statuses })
    }
}