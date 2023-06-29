//! The data structure of Buckets.

use ink_prelude::vec::Vec;
use ink_prelude::string::String;
use ink_storage::traits::{SpreadAllocate, PackedAllocate, PackedLayout, SpreadLayout};
use scale::{Decode, Encode};
use ink_primitives::Key;
use crate::ddc_bucket::{
    AccountId, ClusterId,
    Error::*, Result,
};
use crate::ddc_bucket::flow::Flow;
use crate::ddc_bucket::node::entity::Resource;


pub type BucketId = u32;
pub type BucketParams = String;

#[derive(Clone, PartialEq, Encode, Decode, SpreadAllocate, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct Bucket {
    pub owner_id: AccountId,
    pub cluster_id: ClusterId,
    pub flow: Flow,
    pub resource_reserved: Resource,
    pub public_availability: bool,
    pub resource_consumption_cap: Resource, 
    pub bucket_params: BucketParams
}

// https://use.ink/3.x/ink-vs-solidity#nested-mappings--custom--advanced-structures
#[allow(unconditional_recursion)]
impl ink_storage::traits::PackedAllocate for Bucket {
    fn allocate_packed(&mut self, at: &Key) {
        PackedAllocate::allocate_packed(&mut *self, at)
    }
}

// Add to status field bucket availability
#[derive(Clone, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct BucketInStatus {
    pub owner_id: AccountId,
    pub cluster_id: ClusterId,
    // The field "flow" is not included because it triggers a bug in polkadot.js.
    // TODO: find a fix, then return the entire Bucket structure.
    pub resource_reserved: Resource,
    pub public_availability: bool,
    pub resource_consumption_cap: Resource, 
}

#[derive(Clone, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct BucketStatus {
    pub bucket_id: BucketId,
    pub bucket: BucketInStatus,
    pub params: BucketParams,
    pub writer_ids: Vec<AccountId>,
    pub reader_ids: Vec<AccountId>,
    pub rent_covered_until_ms: u64,
}

pub const BUCKET_PARAMS_MAX_LEN: usize = 100_000;

impl Bucket {
    pub fn only_owner(&self, caller: AccountId) -> Result<()> {
        if self.owner_id == caller { Ok(()) } else { Err(OnlyOwner) }
    }

    pub fn put_resource(&mut self, amount: Resource) {
        self.resource_reserved += amount;
    }

    pub fn set_cap(&mut self, amount: Resource) {
        self.resource_consumption_cap = amount;
    }

    pub fn set_availability(&mut self, availability: bool) {
        self.public_availability = availability;
    }

    pub fn change_owner(&mut self, owner_id: AccountId) {
        self.owner_id = owner_id;
    }

    pub fn set_params(&mut self, bucket_params: BucketParams) -> Result<()> {
        if bucket_params.len() > BUCKET_PARAMS_MAX_LEN {
            return Err(ParamsSizeExceedsLimit);
        }
        self.bucket_params = bucket_params;
        Ok(())
    }

}

impl From<Bucket> for BucketInStatus {
    fn from(bucket: Bucket) -> Self {
        Self {
            owner_id: bucket.owner_id,
            cluster_id: bucket.cluster_id,
            resource_reserved: bucket.resource_reserved,
            public_availability: bucket.public_availability,
            resource_consumption_cap: bucket.resource_consumption_cap,
        }
    }
}
