//! The data structure of Nodes.

use ink_storage::traits::{SpreadAllocate, PackedLayout, SpreadLayout, PackedAllocate};
use ink_prelude::vec::Vec;
use ink_prelude::string::String;
use scale::{Decode, Encode};

use crate::ddc_bucket::{AccountId, Balance, VNodeToken, Error::*, Result};
use crate::ddc_bucket::cluster::entity::ClusterId;

use ink_storage::traits::KeyPtr;
use ink_primitives::Key;

pub type ProviderId = AccountId;
pub type NodeKey = AccountId;
pub type NodeParams = String;
pub type Resource = u32;

#[derive(Clone, PartialEq, Encode, Decode, SpreadAllocate, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct Node {
    pub provider_id: ProviderId,
    pub rent_v_node_per_month: Balance,
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
    ADDING,
    ACTIVE,
    DELETING,
    OFFLINE,
}

impl SpreadAllocate for NodeStatusInCluster { 
    fn allocate_spread(_: &mut KeyPtr) -> Self { 
        NodeStatusInCluster::ADDING 
    }
}

impl Default for NodeStatusInCluster {
    fn default() -> Self {
        NodeStatusInCluster::ADDING 
    }
}

#[derive(Clone, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct NodeInfo {
    pub node_key: NodeKey,
    pub node: Node,
    pub v_nodes: Vec<VNodeToken>
}

pub const NODE_PARAMS_MAX_LEN: usize = 100_000;

impl Node {

    pub fn new(
        provider_id: AccountId,
        node_params: NodeParams,
        capacity: Resource,
        rent_v_node_per_month: Balance,
    ) -> Result<Self> {
        let mut node = Node {
            provider_id,
            node_params: NodeParams::default(),
            free_resource: capacity,
            rent_v_node_per_month,
            cluster_id: None,
            status_in_cluster: None,
        };

        node.set_params(node_params)?;
        Ok(node)
    }

    pub fn only_provider(&self, caller: AccountId) -> Result<()> {
        (self.provider_id == caller)
            .then(|| ())
            .ok_or(OnlyNodeProvider)
    }

    pub fn only_without_cluster(&self) -> Result<()> {
        self.cluster_id
            .map_or(Ok(()), |cluster_id| Err(NodeIsAddedToCluster(cluster_id)))
    }

    pub fn only_with_cluster(&self, cluster_id: ClusterId) -> Result<()> {
        self.cluster_id
            .is_some()
            .then(|| ())
            .ok_or(NodeIsNotAddedToCluster(cluster_id))
    }

    pub fn set_params(&mut self, node_params: NodeParams) -> Result<()> {
        if node_params.len() > NODE_PARAMS_MAX_LEN {
            return Err(ParamsSizeExceedsLimit);
        }
        self.node_params = node_params;
        Ok(())
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
    
    pub fn release_resource(&mut self, amount: Resource) {
        self.free_resource += amount;
    }

    pub fn reserve_resource(&mut self, amount: Resource) -> Result<()> {
        if self.free_resource >= amount {
            self.free_resource -= amount;
            Ok(())
        } else {
            Err(InsufficientNodeResources)
        }
    }

}
