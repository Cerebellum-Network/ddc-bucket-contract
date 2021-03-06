//! The data structure of Nodes.

use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};

use crate::ddc_bucket::{AccountId, Balance, Error::*, Result};
use crate::ddc_bucket::contract_fee::{SIZE_ACCOUNT_ID, SIZE_BALANCE, SIZE_PER_RECORD, SIZE_RESOURCE};
use crate::ddc_bucket::params::store::Params;

pub type ProviderId = AccountId;
pub type NodeId = u32;
pub type NodeParams = Params;
pub type Resource = u32;

#[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct Node {
    pub provider_id: ProviderId,
    pub rent_per_month: Balance,
    pub free_resource: Resource,
}

#[derive(Clone, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct NodeStatus {
    pub node_id: NodeId,
    pub node: Node,
    pub params: Params,
}

impl Node {
    pub const RECORD_SIZE: usize = SIZE_PER_RECORD
        + SIZE_ACCOUNT_ID + SIZE_BALANCE + SIZE_RESOURCE;

    pub fn revenue_account_id(&self) -> AccountId {
        self.provider_id
    }

    pub fn only_owner(&self, provider_id: AccountId) -> Result<()> {
        if self.provider_id == provider_id { Ok(()) } else { Err(UnauthorizedProvider) }
    }

    pub fn put_resource(&mut self, amount: Resource) {
        self.free_resource += amount;
    }

    pub fn take_resource(&mut self, amount: Resource) -> Result<()> {
        if self.free_resource >= amount {
            self.free_resource -= amount;
            Ok(())
        } else {
            Err(InsufficientResources)
        }
    }
}