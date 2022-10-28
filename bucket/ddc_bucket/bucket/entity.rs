//! The data structure of Buckets.

use ink_prelude::vec::Vec;
use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};

use crate::ddc_bucket::{
    AccountId, ClusterId, contract_fee::SIZE_PER_RECORD,
    Error::*, Result,
};
use crate::ddc_bucket::contract_fee::{SIZE_ACCOUNT_ID, SIZE_INDEX, SIZE_RESOURCE};
use crate::ddc_bucket::flow::Flow;
use crate::ddc_bucket::node::entity::Resource;
use crate::ddc_bucket::params::store::Params;

pub type BucketId = u32;
pub type BucketParams = Params;

#[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct Bucket {
    pub owner_id: AccountId,
    pub cluster_id: ClusterId,
    pub flow: Flow,
    pub resource_reserved: Resource,
    pub public_availability: bool,
    pub resource_consumption_cap: Resource, 
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
    pub rent_covered_until_ms: u64,
}

impl Bucket {
    pub const RECORD_SIZE: usize = SIZE_PER_RECORD
        + SIZE_ACCOUNT_ID + SIZE_INDEX + Flow::RECORD_SIZE + SIZE_RESOURCE;

    pub fn only_owner(&self, caller: AccountId) -> Result<()> {
        if self.owner_id == caller { Ok(()) } else { Err(UnauthorizedOwner) }
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
