//! The data structure of Nodes.
use scale::{Decode, Encode};
use crate::ddc_bucket::params::store::Params;
use crate::ddc_bucket::{AccountId, Balance, Error::*, Result};

pub type ProviderId = AccountId;
pub type NodeId = u32;
pub type NodeParams = Params;
pub type Resource = u32;

#[derive(Encode, Decode, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout, Debug))]
pub struct Node {
    pub provider_id: ProviderId,
    pub rent_per_month: Balance,
    pub free_resource: Resource,
    pub node_tag: NodeTag,
}

#[derive(Encode, Decode, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout, Debug))]
pub enum NodeTag {
    ACTIVE,
    ADDING,
    DELETING,
    OFFLINE,
}

#[derive(Encode, Decode, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, Debug))]
pub struct NodeStatus {
    pub node_id: NodeId,
    pub node: Node,
    pub params: Params,
}

impl Node {
    pub fn revenue_account_id(&self) -> AccountId {
        self.provider_id
    }

    pub fn only_owner(&self, provider_id: AccountId) -> Result<()> {
        if self.provider_id == provider_id {
            Ok(())
        } else {
            Err(UnauthorizedProvider)
        }
    }

    pub fn change_tag(&mut self, new_tag: NodeTag) {
        self.node_tag = new_tag;
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
