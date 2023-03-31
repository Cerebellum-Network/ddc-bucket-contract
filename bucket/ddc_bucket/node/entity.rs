//! The data structure of Nodes.

use ink_storage::traits::{SpreadAllocate, PackedLayout, SpreadLayout, PackedAllocate};
use scale::{Decode, Encode};

use crate::ddc_bucket::params::store::Params;
use crate::ddc_bucket::{AccountId, Balance, Error::*, Result};
use ink_storage::traits::KeyPtr;
use ink_primitives::Key;

pub type ProviderId = AccountId;
pub type NodeId = u32;
pub type NodeParams = Params;
pub type Resource = u32;

#[derive(Clone, PartialEq, Encode, Decode, SpreadAllocate, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct Node {
    pub provider_id: ProviderId,
    pub rent_per_month: Balance,
    pub free_resource: Resource,
    pub node_tag: NodeTag,
}

impl ink_storage::traits::PackedAllocate for Node {
    fn allocate_packed(&mut self, at: &Key) {
        PackedAllocate::allocate_packed(&mut *self, at)
    }
}

#[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub enum NodeTag {
    UNKNOWN,
    ACTIVE,
    ADDING,
    DELETING,
    OFFLINE,
}

impl SpreadAllocate for NodeTag { 
    fn allocate_spread(_: &mut KeyPtr) -> Self { 
        NodeTag::UNKNOWN 
    }
}

impl Default for NodeTag {
    fn default() -> Self {
        NodeTag::UNKNOWN 
    }
}

#[derive(Clone, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
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
