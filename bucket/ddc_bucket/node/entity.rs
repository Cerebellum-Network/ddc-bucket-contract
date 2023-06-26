//! The data structure of Nodes.

use ink_storage::traits::{SpreadAllocate, PackedLayout, SpreadLayout, PackedAllocate};
use scale::{Decode, Encode};

use crate::ddc_bucket::params::store::Params;
use crate::ddc_bucket::{AccountId, Balance, Error::*, Result};
use crate::ddc_bucket::cluster::entity::ClusterId;

use ink_storage::traits::KeyPtr;
use ink_primitives::Key;

pub type ProviderId = AccountId;
pub type NodeKey = AccountId;
pub type NodeParams = Params;
pub type Resource = u32;

#[derive(Clone, PartialEq, Encode, Decode, SpreadAllocate, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct Node {
    pub provider_id: ProviderId,
    pub rent_per_month: Balance,
    pub free_resource: Resource,
    pub node_params: NodeParams,
    pub cluster_id: Option<ClusterId>,
    pub status_in_cluster: Option<NodeStatusInCluster>,
}

// https://use.ink/3.x/ink-vs-solidity#nested-mappings--custom--advanced-structures
#[allow(unconditional_recursion)]
impl ink_storage::traits::PackedAllocate for Node {
    fn allocate_packed(&mut self, at: &Key) {
        PackedAllocate::allocate_packed(&mut *self, at)
    }
}

#[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub enum NodeStatusInCluster {
    ACTIVE,
    ADDING,
    DELETING,
    OFFLINE,
}

impl SpreadAllocate for NodeStatusInCluster { 
    fn allocate_spread(_: &mut KeyPtr) -> Self { 
        NodeStatusInCluster::ACTIVE 
    }
}

impl Default for NodeStatusInCluster {
    fn default() -> Self {
        NodeStatusInCluster::ACTIVE 
    }
}

#[derive(Clone, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct NodeInfo {
    pub node_key: NodeKey,
    pub node: Node,
}

impl Node {

    pub fn new(
        provider_id: AccountId,
        node_params: NodeParams,
        capacity: Resource,
        rent_per_month: Balance,
    ) -> Self {
        Node {
            provider_id,
            node_params,
            free_resource: capacity,
            rent_per_month,
            cluster_id: None,
            status_in_cluster: None,
        }
    }

    pub fn revenue_account_id(&self) -> AccountId {
        self.provider_id
    }

    pub fn only_owner(&self, owner_id: AccountId) -> Result<()> {
        if self.provider_id == owner_id {
            Ok(())
        } else {
            Err(UnauthorizedNodeOwner)
        }
    }

    pub fn set_cluster(&mut self, cluster_id: ClusterId, status: NodeStatusInCluster) {
        self.cluster_id = Some(cluster_id);
        self.status_in_cluster = Some(status);
    }

    pub fn unset_cluster(&mut self) {
        self.cluster_id = None;
        self.status_in_cluster = None;
    }

    pub fn change_status_in_cluster(&mut self, status: NodeStatusInCluster) {
        self.status_in_cluster = Some(status);
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

    pub fn only_without_cluster(&self) -> Result<()> {
        if let Some(cluster_id) = self.cluster_id {
            Err(NodeIsAlreadyAddedToCluster(cluster_id))
        } else {
            Ok(())
        }
    }

    pub fn only_with_cluster(&self, cluster_id: ClusterId) -> Result<()> {
        if let Some(_) = self.cluster_id {
            Ok(())
        } else {
            Err(NodeIsNotAddedToCluster(cluster_id))
        }
    }
}
