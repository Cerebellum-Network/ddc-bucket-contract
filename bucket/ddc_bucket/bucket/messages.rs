//! The public interface to manage Buckets.

use ink_lang::{EmitEvent, StaticEnv};
use ink_prelude::{vec, vec::Vec};

use crate::ddc_bucket::{AccountId, BucketAllocated, BucketCreated, DdcBucket, DealCreated, Result};
use crate::ddc_bucket::cluster::entity::ClusterId;
use crate::ddc_bucket::contract_fee::SIZE_INDEX;
use crate::ddc_bucket::deal::entity::Deal;
use crate::ddc_bucket::Error::ClusterDoesNotExist;
use crate::ddc_bucket::node::entity::Resource;

use super::entity::{Bucket, BucketId, BucketParams, BucketStatus};

impl DdcBucket {
    pub fn message_bucket_create(&mut self, bucket_params: BucketParams) -> Result<BucketId> {
        let owner_id = Self::env().caller();
        let (bucket_id, record_size) = self.buckets.create(owner_id, bucket_params);
        Self::capture_fee_and_refund(record_size)?;
        Self::env().emit_event(BucketCreated { bucket_id, owner_id });
        Ok(bucket_id)
    }

    pub fn message_bucket_alloc_into_cluster(&mut self, bucket_id: BucketId, cluster_id: ClusterId) -> Result<()> {
        let owner_id = Self::env().caller();

        let node_ids = self.clusters.get(cluster_id)?.vnodes.clone();
        let mut deal_ids = Vec::with_capacity(node_ids.len());

        for node_id in node_ids.iter() {
            let deal_id = self.deal_create(*node_id)?;
            deal_ids.push(deal_id);
        }

        let bucket = self.buckets.get_mut(bucket_id)?;
        bucket.only_owner(owner_id)?;
        bucket.connect_cluster(cluster_id)?;

        Self::env().emit_event(BucketAllocated { bucket_id, cluster_id });

        for (&node_id, deal_id) in node_ids.iter().zip(deal_ids) {
            bucket.deal_ids.push(deal_id);
            Self::env().emit_event(DealCreated { deal_id, bucket_id, node_id: node_id });
        }

        // Capture the contract storage fee.
        let record_size = node_ids.len() * (Deal::RECORD_SIZE + SIZE_INDEX) + SIZE_INDEX;
        Self::capture_fee_and_refund(record_size)?;
        Ok(())
    }

    fn _message_bucket_reserve_resource(&mut self, bucket_id: BucketId, amount: Resource) -> Result<()> {
        let bucket = self.buckets.get_mut(bucket_id)?;
        let cluster_id = *bucket.cluster_ids.last().ok_or(ClusterDoesNotExist)?;
        let cluster = self.clusters.get_mut(cluster_id)?;

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