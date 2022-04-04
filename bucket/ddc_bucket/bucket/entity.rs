//! The data structure of Buckets.

use ink_prelude::{
    string::String,
    vec::Vec,
};
use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};

use crate::ddc_bucket::{
    AccountId, ClusterId, contract_fee::SIZE_PER_RECORD,
    Error::*, Result,
};
use crate::ddc_bucket::contract_fee::{SIZE_ACCOUNT_ID, SIZE_INDEX, SIZE_RESOURCE, SIZE_VEC};
use crate::ddc_bucket::flow::Flow;
use crate::ddc_bucket::node::entity::Resource;

pub type BucketId = u32;
pub type BucketParams = String;

#[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct Bucket {
    pub owner_id: AccountId,
    pub cluster_id: ClusterId,
    pub flow: Flow,
    // TODO: make lazy.
    pub bucket_params: BucketParams,
    pub resource_reserved: Resource,
}

#[derive(Clone, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct BucketStatus {
    pub bucket_id: BucketId,
    pub bucket: Bucket,
    pub writer_ids: Vec<AccountId>,
    pub rent_covered_until_ms: u64,
}

impl Bucket {
    pub fn new_size(&self) -> usize {
        SIZE_PER_RECORD
            + SIZE_ACCOUNT_ID + SIZE_INDEX + Flow::RECORD_SIZE
            + SIZE_VEC + self.bucket_params.len()
            + SIZE_RESOURCE
        // Or to be more precise:    SIZE_PER_RECORD + self.encoded_size()
    }

    pub fn only_owner(&self, caller: AccountId) -> Result<()> {
        if self.owner_id == caller { Ok(()) } else { Err(UnauthorizedOwner) }
    }

    pub fn put_resource(&mut self, amount: Resource) {
        self.resource_reserved += amount;
    }
}
