//! The data structure of Clusters.

use ink_prelude::vec::Vec;
use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};

use crate::ddc_bucket::{AccountId, Balance, Error::InsufficientResources, NodeId, Result};
use crate::ddc_bucket::cash::Cash;
use crate::ddc_bucket::contract_fee::{SIZE_ACCOUNT_ID, SIZE_BALANCE, SIZE_INDEX, SIZE_PER_RECORD, SIZE_RESOURCE, SIZE_VEC};
use crate::ddc_bucket::Error::UnauthorizedClusterManager;
use crate::ddc_bucket::node::entity::{Node, Resource};
use crate::ddc_bucket::params::store::Params;

pub type ClusterId = u32;
pub type ClusterParams = Params;
pub type VNodeIndex = u32;
pub type VNodeId = (ClusterId, VNodeIndex);

#[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct Cluster {
    pub manager_id: AccountId,
    pub vnodes: Vec<NodeId>,
    pub resource_per_vnode: Resource,
    pub resource_used: Resource,
    pub revenues: Cash,
    pub total_rent: Balance,
}

#[derive(Clone, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct ClusterStatus {
    pub cluster_id: ClusterId,
    pub cluster: Cluster,
    pub params: Params,
}

impl Cluster {
    pub fn new(
        manager_id: AccountId,
        vnode_count: u32,
        nodes: &[(NodeId, &Node)],
    ) -> Self {
        let (vnodes, total_rent) = Self::new_vnodes(vnode_count as usize, nodes);
        Cluster {
            manager_id,
            vnodes,
            resource_per_vnode: 0,
            resource_used: 0,
            revenues: Cash(0),
            total_rent,
        }
    }

    fn new_vnodes(vnode_count: usize, nodes: &[(NodeId, &Node)]) -> (Vec<NodeId>, Balance) {
        let node_count = nodes.len();
        let mut vnode_ids = Vec::with_capacity(vnode_count);
        let mut total_rent = 0;
        for i in 0..vnode_count {
            let (node_id, node) = &nodes[i % node_count];
            vnode_ids.push(*node_id);
            total_rent += node.rent_per_month;
        }
        // TODO: consider using the max rent instead of average rent.
        (vnode_ids, total_rent)
    }

    pub fn new_size(&self) -> usize {
        SIZE_PER_RECORD
            + SIZE_ACCOUNT_ID
            + SIZE_VEC + self.vnodes.len() * SIZE_INDEX
            + SIZE_RESOURCE
            + SIZE_RESOURCE
            + SIZE_BALANCE
            + SIZE_BALANCE
        // Or to be more precise:    SIZE_PER_RECORD + self.encoded_size()
    }

    pub fn get_rent(&self, resource: Resource) -> Balance {
        return self.total_rent * resource as Balance;
    }

    pub fn put_resource(&mut self, amount: Resource) {
        self.resource_per_vnode += amount;
    }

    pub fn take_resource(&mut self, amount: Resource) -> Result<()> {
        let used = self.resource_used + amount;
        if used > self.resource_per_vnode {
            return Err(InsufficientResources);
        }
        self.resource_used = used;
        Ok(())
    }

    pub fn only_manager(&self, caller: AccountId) -> Result<()> {
        if self.manager_id == caller { Ok(()) } else { Err(UnauthorizedClusterManager) }
    }
}