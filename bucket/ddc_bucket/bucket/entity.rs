//! The data structure of Buckets.

use ink_prelude::{
    string::String,
    vec::Vec,
};
use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};

use crate::ddc_bucket::{
    AccountId, ClusterId, contract_fee::SIZE_PER_RECORD,
    deal::entity::{DealId, DealStatus}, Error::*,
    Result,
};
use crate::ddc_bucket::contract_fee::{SIZE_ACCOUNT_ID, SIZE_VEC};
use crate::ddc_bucket::node::entity::Resource;
use crate::ddc_bucket::flow::Flow;

pub type BucketId = u32;
pub type BucketParams = String;

#[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct Bucket {
    pub owner_id: AccountId,
    pub cluster_ids: Vec<ClusterId>,
    pub flows: Vec<Flow>,
    pub deal_ids: Vec<DealId>,
    pub bucket_params: BucketParams,
    pub resource_reserved: Resource,
}

#[derive(Clone, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct BucketStatus {
    pub bucket_id: BucketId,
    pub bucket: Bucket,
    pub writer_ids: Vec<AccountId>,
    pub deal_statuses: Vec<DealStatus>,
}

impl Bucket {
    pub fn new_size(&self) -> usize {
        // TODO: add flows.
        SIZE_PER_RECORD
            + SIZE_ACCOUNT_ID + SIZE_VEC + SIZE_VEC + SIZE_VEC
            + self.bucket_params.len()
        // Or to be more precise:    SIZE_PER_RECORD + self.encoded_size()
    }

    pub fn only_owner(&self, caller: AccountId) -> Result<()> {
        if self.owner_id == caller { Ok(()) } else { Err(UnauthorizedOwner) }
    }

    pub fn connect_cluster(&mut self, cluster_id: ClusterId) -> Result<()> {
        if self.cluster_ids.contains(&cluster_id) {
            Err(BucketClusterAlreadyConnected)
        } else {
            self.cluster_ids.push(cluster_id);
            Ok(())
        }
    }

    pub fn put_resource(&mut self, amount: Resource) {
        self.resource_reserved += amount;
    }
}
