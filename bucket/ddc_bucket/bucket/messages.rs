//! The public interface to manage Buckets.

use ink_lang::{EmitEvent, StaticEnv};
use ink_prelude::{vec, vec::Vec};

use crate::ddc_bucket::{AccountId, BucketAllocated, BucketCreated, DdcBucket, Result};
use crate::ddc_bucket::cluster::entity::ClusterId;
use crate::ddc_bucket::Error::BucketClusterNotSetup;
use crate::ddc_bucket::node::entity::Resource;

use super::entity::{Bucket, BucketId, BucketParams, BucketStatus};

impl DdcBucket {
    pub fn message_bucket_create(&mut self, bucket_params: BucketParams, cluster_id: ClusterId) -> Result<BucketId> {
        let owner_id = Self::env().caller();
        let record_size0 = self.accounts.create_if_not_exist(owner_id);
        let (bucket_id, record_size1) = self.buckets.create(owner_id, bucket_params, cluster_id);
        Self::capture_fee_and_refund(record_size0 + record_size1)?;
        Self::env().emit_event(BucketCreated { bucket_id, owner_id });
        Ok(bucket_id)
    }

    pub fn message_bucket_alloc_into_cluster(&mut self, bucket_id: BucketId) -> Result<()> {
        let owner_id = Self::env().caller();

        let bucket = self.buckets.get_mut(bucket_id)?;
        bucket.only_owner(owner_id)?;

        let rent = self.clusters.get(bucket.cluster_id)?.get_rent();

        // Start the payment flow to the cluster.
        let start_ms = Self::env().block_timestamp();
        let flow = self.accounts.start_flow(start_ms, owner_id, rent)?;
        bucket.flows.push(flow);

        Self::env().emit_event(BucketAllocated { bucket_id, cluster_id: bucket.cluster_id });

        // Capture the contract storage fee.
        let record_size = 0; // TODO.
        Self::capture_fee_and_refund(record_size)?;
        Ok(())
    }

    pub fn message_bucket_settle_payment(&mut self, bucket_id: BucketId) -> Result<()> {
        let bucket = self.buckets.get_mut(bucket_id)?;
        let flow = bucket.flows.get_mut(0).ok_or(BucketClusterNotSetup)?;

        let now_ms = Self::env().block_timestamp();
        let cash = self.accounts.settle_flow(now_ms, flow)?;

        let cluster = self.clusters.get_mut(bucket.cluster_id)?;
        cluster.revenues.increase(cash);

        Ok(())
    }

    fn _message_bucket_reserve_resource(&mut self, bucket_id: BucketId, amount: Resource) -> Result<()> {
        let bucket = self.buckets.get_mut(bucket_id)?;
        let cluster = self.clusters.get_mut(bucket.cluster_id)?;

        let owner_id = Self::env().caller();
        bucket.only_owner(owner_id)?;

        cluster.take_resource(amount)?;
        bucket.put_resource(amount);
        Ok(())
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
            match self.bucket_calculate_status(bucket_id, bucket.clone()) {
                Err(_) => continue, // Skip on unexpected error.
                Ok(status) =>
                    bucket_statuses.push(status),
            };
        }
        (bucket_statuses, self.buckets.0.len())
    }

    pub fn message_bucket_get_status(&self, bucket_id: BucketId) -> Result<BucketStatus> {
        let bucket = self.bucket_get(bucket_id)?;
        self.bucket_calculate_status(bucket_id, bucket)
    }

    pub fn bucket_calculate_status(&self, bucket_id: BucketId, bucket: Bucket) -> Result<BucketStatus> {
        let writer_ids = vec![bucket.owner_id];

        let rent_covered_until_ms = match bucket.flows.first() {
            Some(flow) =>
                self.accounts.flow_covered_until(flow)?,
            None => 0,
        };

        Ok(BucketStatus { bucket_id, bucket, writer_ids, rent_covered_until_ms })
    }
}