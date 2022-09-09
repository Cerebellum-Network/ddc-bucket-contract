//! The public interface to manage Buckets.

use ink_lang::{EmitEvent, StaticEnv};
use ink_prelude::{vec, vec::Vec};

use crate::ddc_bucket::{AccountId, Balance, BucketAllocated, BucketCreated, BucketSettlePayment, Payable, DdcBucket, Error::*, Result};
use crate::ddc_bucket::cluster::entity::{Cluster, ClusterId};
use crate::ddc_bucket::node::entity::Resource;

use super::entity::{Bucket, BucketId, BucketParams, BucketStatus, BalancePerMonth};

const PRICE_PER_GB: Balance = 1;
const KB_PER_GB: Balance = 1000000;
const BASE: u32 = 10;

impl DdcBucket {
    pub fn message_bucket_create(&mut self, bucket_params: BucketParams, cluster_id: ClusterId) -> Result<BucketId> {
        let owner_id = Self::env().caller();
        let record_size0 = self.accounts.create_if_not_exist(owner_id);
        let bucket_id = self.buckets.create(owner_id, cluster_id);
        let (params_id, record_size2) = self.bucket_params.create(bucket_params)?;
        assert_eq!(bucket_id, params_id);
        Self::capture_fee_and_refund(record_size0 + Bucket::RECORD_SIZE + record_size2)?;
        Self::env().emit_event(BucketCreated { bucket_id, owner_id });
        Ok(bucket_id)
    }

    pub fn message_buy_resources(&mut self, bucket_id: BucketId, prepaid_resources: Balance, max_rate: BalancePerMonth) -> Result<()> {
        let owner_id = Self::env().caller();
        let bucket = self.buckets.get_mut(bucket_id)?;
        let account = self.accounts.0.get_mut(&owner_id).ok_or(InsufficientBalance)?;
        let time_ms = Self::env().block_timestamp();
        let conv = &self.accounts.1;
        account.withdraw(time_ms, conv, Payable(prepaid_resources))?;
        bucket.prepaid_resources += prepaid_resources;
        bucket.max_rate = max_rate;
        bucket.period_start = time_ms;
        bucket.period_prepaid_remaining = prepaid_resources;
        Ok(())
    }

    pub fn message_consume_resources(&mut self, bucket_id: BucketId, resource: u128) -> Result<()> {

        let bucket = self.buckets.get_mut(bucket_id)?;
        let cluster = self.clusters.get_mut(bucket.cluster_id)?;
        Self::only_owner_or_cluster_manager(bucket, cluster)?;

        let parsed_resource_cere = self.accounts.1.to_cere((resource / KB_PER_GB) * PRICE_PER_GB);
        if bucket.period_prepaid_remaining >= parsed_resource_cere {
            bucket.period_prepaid_remaining -= parsed_resource_cere;
            Ok(())
        } else {
            Err(MaxRateExceeded)
        }
    }

    pub fn message_calculate_prepaid(&self, bucket_id: BucketId) -> Result<u128> {
        let bucket = self.buckets.get(bucket_id)?;
        core::prelude::v1::Ok((self.accounts.1.to_usd(bucket.period_prepaid_remaining) / PRICE_PER_GB) * KB_PER_GB)
    }

    pub fn message_bucket_alloc_into_cluster(&mut self, bucket_id: BucketId, resource: Resource) -> Result<()> {
        let bucket = self.buckets.get_mut(bucket_id)?;
        let cluster = self.clusters.get_mut(bucket.cluster_id)?;
        Self::only_owner_or_cluster_manager(bucket, cluster)?;

        cluster.take_resource(resource)?;
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

        let cluster = self.clusters.get_mut(bucket.cluster_id)?;
        cluster.revenues.increase(cash);

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

    pub fn bucket_calculate_status(&self, bucket_id: BucketId, bucket: Bucket) -> Result<BucketStatus> {
        let writer_ids = vec![bucket.owner_id];
        let rent_covered_until_ms = self.accounts.flow_covered_until(&bucket.flow)?;
        let params = self.bucket_params.get(bucket_id)?.clone();
        Ok(BucketStatus { bucket_id, bucket: bucket.into(), params, writer_ids, rent_covered_until_ms })
    }

    fn only_owner_or_cluster_manager(bucket: &Bucket, cluster: &Cluster) -> Result<()> {
        let caller = Self::env().caller();
        cluster.only_manager(caller)
            .or_else(|_|
                bucket.only_owner(caller))
    }
}