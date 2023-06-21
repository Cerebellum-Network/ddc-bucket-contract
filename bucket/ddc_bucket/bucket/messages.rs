//! The public interface to manage Buckets.

use ink_lang::codegen::{EmitEvent, StaticEnv};
use ink_prelude::vec::Vec;

use crate::ddc_bucket::{AccountId, BucketAllocated, BucketCreated, BucketSettlePayment, BucketAvailabilityUpdated, DdcBucket, Result};
use crate::ddc_bucket::cluster::entity::{Cluster, ClusterId};
use crate::ddc_bucket::node::entity::Resource;

use super::entity::{Bucket, BucketId, BucketParams, BucketStatus};

impl DdcBucket {
    pub fn message_bucket_create(&mut self, bucket_params: BucketParams, cluster_id: ClusterId, owner_id: Option<AccountId>) -> Result<BucketId> {
        let owner_id = owner_id.unwrap_or(Self::env().caller());
        self.accounts.create_if_not_exist(owner_id);
        let bucket_id = self.buckets.create(owner_id, cluster_id);
        let params_id = self.bucket_params.create(bucket_params)?;
        assert_eq!(bucket_id, params_id);
        Self::env().emit_event(BucketCreated { bucket_id, owner_id });
        Ok(bucket_id)
    }

    pub fn message_bucket_change_owner(&mut self, bucket_id: BucketId, owner_id: AccountId) -> Result<()> {
        let caller = Self::env().caller();
        let bucket = self.buckets.get_mut(bucket_id)?;
        bucket.only_owner(caller)?;
        bucket.change_owner(owner_id);
        Ok(())
    }

    pub fn message_bucket_alloc_into_cluster(&mut self, bucket_id: BucketId, resource: Resource) -> Result<()> {
        let bucket = self.buckets.get_mut(bucket_id)?;
        let mut cluster = self.clusters.get(bucket.cluster_id)?;
        Self::only_owner_or_cluster_manager(bucket, &cluster)?;

        cluster.take_resource(resource)?;
        self.clusters.update(bucket.cluster_id, &cluster)?;
        bucket.put_resource(resource);

        // Start the payment flow to the cluster.
        let rent = cluster.get_rent(resource);
        let now_ms = Self::env().block_timestamp();

        self.accounts.increase_flow(now_ms, rent, &mut bucket.flow)?;

        Self::env().emit_event(BucketAllocated { bucket_id, cluster_id: bucket.cluster_id, resource });
        Ok(())
    }

    pub fn message_bucket_settle_payment(&mut self, bucket_id: BucketId) -> Result<()> {
        let bucket = self.buckets.get_mut(bucket_id)?;

        let now_ms = Self::env().block_timestamp();
        let cash = self.accounts.settle_flow(now_ms, &mut bucket.flow)?;

        let mut cluster = self.clusters.get(bucket.cluster_id)?;
        cluster.revenues.increase(cash);
        self.clusters.update(bucket.cluster_id, &cluster)?;

        Self::env().emit_event(BucketSettlePayment { bucket_id, cluster_id: bucket.cluster_id });
        Ok(())
    }


    pub fn message_bucket_change_params(&mut self, bucket_id: BucketId, params: BucketParams) -> Result<()> {
        let caller = Self::env().caller();
        let bucket = self.buckets.get(bucket_id)?;
        bucket.only_owner(caller)?;

        Self::impl_change_params(&mut self.bucket_params, bucket_id, params)
    }

    pub fn message_bucket_get(&self, bucket_id: BucketId) -> Result<BucketStatus> {
        let bucket = self.buckets.get(bucket_id)?.clone();
        self.bucket_calculate_status(bucket_id, bucket)
    }

    pub fn message_bucket_list(&self, offset: u32, limit: u32, filter_owner_id: Option<AccountId>) -> (Vec<BucketStatus>, u32) {
        let mut bucket_statuses = Vec::with_capacity(limit as usize);
        for bucket_id in offset..offset + limit {
            let bucket = match self.buckets.0.get(bucket_id as usize) {
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
        (bucket_statuses, self.buckets.0.len().try_into().unwrap())
    }

    pub fn message_bucket_list_for_account(&self, owner_id: AccountId) -> Vec<Bucket> {
        self.buckets.0
            .iter()
            .filter(|bucket| bucket.owner_id == owner_id)
            .cloned()
            .collect()
    }

    pub fn bucket_calculate_status(&self, bucket_id: BucketId, bucket: Bucket) -> Result<BucketStatus> {
        let mut writer_ids = self.buckets_perms.get_bucket_readers(bucket_id);
        writer_ids.push(bucket.owner_id);
        let rent_covered_until_ms = self.accounts.flow_covered_until(&bucket.flow)?;
        let params = self.bucket_params.get(bucket_id)?.clone();
        let reader_ids = self.buckets_perms.get_bucket_readers(bucket_id);
        Ok(BucketStatus { bucket_id, bucket: bucket.into(), params, writer_ids, reader_ids, rent_covered_until_ms })
    }

    pub fn message_bucket_set_resource_cap(&mut self, bucket_id: BucketId, new_resource_cap: Resource) ->  Result<()> {
        let bucket = self.buckets.get_mut(bucket_id)?;
        let cluster = self.clusters.get(bucket.cluster_id)?;

        Self::only_owner_or_cluster_manager(bucket, &cluster)?;
        bucket.set_cap(new_resource_cap);
        Ok(())
    }

    pub fn message_bucket_set_availability(&mut self, bucket_id: BucketId, public_availability: bool) -> Result<()> {
        let bucket = self.buckets.get_mut(bucket_id)?;
        let cluster = self.clusters.get(bucket.cluster_id)?;
        
        Self::only_owner_or_cluster_manager(bucket, &cluster)?;
        bucket.set_availability(public_availability);

        Self::env().emit_event(BucketAvailabilityUpdated { bucket_id, public_availability });
        Ok(())
    }

    pub fn message_get_bucket_writers(&mut self, bucket_id: BucketId) -> Result<Vec<AccountId>> {
        let writers = self.buckets_perms.get_bucket_writers(bucket_id);
        Ok(writers)
    }

    pub fn message_grant_writer_permission(&mut self, bucket_id: BucketId, writer: AccountId) -> Result<()> {
        let bucket = self.buckets.get_mut(bucket_id)?;
        let cluster = self.clusters.get(bucket.cluster_id)?;

        Self::only_owner_or_cluster_manager(bucket, &cluster)?;
        self.buckets_perms.grant_writer_permission(bucket_id, writer).unwrap();
        Ok(())
    }

    pub fn message_revoke_writer_permission(&mut self, bucket_id: BucketId, writer: AccountId) -> Result<()> { 
        let bucket = self.buckets.get_mut(bucket_id)?;
        let cluster = self.clusters.get(bucket.cluster_id)?;

        Self::only_owner_or_cluster_manager(bucket, &cluster)?;
        self.buckets_perms.revoke_writer_permission(bucket_id, writer).unwrap();
        Ok(())
    }

    pub fn message_get_bucket_readers(&mut self, bucket_id: BucketId) -> Result<Vec<AccountId>> {
        let readers = self.buckets_perms.get_bucket_readers(bucket_id);
        Ok(readers)
    }

    pub fn message_grant_reader_permission(&mut self, bucket_id: BucketId, reader: AccountId) -> Result<()> {
        let bucket = self.buckets.get_mut(bucket_id)?;
        let cluster = self.clusters.get(bucket.cluster_id)?;

        Self::only_owner_or_cluster_manager(bucket, &cluster)?;
        self.buckets_perms.grant_reader_permission(bucket_id, reader).unwrap();
        Ok(())
    }

    pub fn message_revoke_reader_permission(&mut self, bucket_id: BucketId, reader: AccountId) -> Result<()> { 
        let bucket = self.buckets.get_mut(bucket_id)?;
        let cluster = self.clusters.get(bucket.cluster_id)?;

        Self::only_owner_or_cluster_manager(bucket, &cluster)?;
        self.buckets_perms.revoke_reader_permission(bucket_id, reader).unwrap();
        Ok(())
    }

    fn only_owner_or_cluster_manager(bucket: &Bucket, cluster: &Cluster) -> Result<()> {
        let caller = Self::env().caller();
        cluster.only_manager(caller)
            .or_else(|_|
                bucket.only_owner(caller))
    }
}