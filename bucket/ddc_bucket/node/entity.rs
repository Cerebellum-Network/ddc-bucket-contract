//! The data structure of Nodes.

use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};

use crate::ddc_bucket::cluster::entity::ClusterId;
use crate::ddc_bucket::params::store::Params;
use crate::ddc_bucket::{AccountId, Balance, Error::*, Result};

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
    pub node_tag: NodeTag,
    pub cluster_id: ClusterId,
}

#[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub enum NodeTag {
    ACTIVE,
    ADDING,
    DELETING,
    OFFLINE,
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

    pub fn assign_cluster_id(&mut self, cluster_id: ClusterId) {
        self.cluster_id = cluster_id;
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
