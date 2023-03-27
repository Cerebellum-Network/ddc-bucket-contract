//! The data structure of Clusters.
// use ink_storage::Mapping;
// use ink_prelude::vec::Vec;
use ink_prelude::vec::Vec;
use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};

use crate::ddc_bucket::cash::Cash;
use crate::ddc_bucket::node::entity::NodeId;
use crate::ddc_bucket::node::entity::Resource;
use crate::ddc_bucket::params::store::Params;
use crate::ddc_bucket::Error::UnauthorizedClusterManager;
use crate::ddc_bucket::{AccountId, Balance, Error::InsufficientResources, Result};

pub type ClusterId = u32;
pub type ClusterParams = Params;
pub type VNodeIndex = u32;
pub type VNodeId = (ClusterId, VNodeIndex);

#[derive(Clone, PartialEq, Encode, Decode, PackedLayout, SpreadLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct Cluster {
    pub manager_id: AccountId,
    pub resource_per_vnode: Resource,
    pub resource_used: Resource,
    pub revenues: Cash,
    pub node_ids: Vec<NodeId>,
    pub v_nodes: Vec<Vec<u64>>,
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
    pub fn new(manager_id: AccountId, v_nodes_arr: &Vec<Vec<u64>>, node_ids: &Vec<NodeId>) -> Self {
        Cluster {
            manager_id,
            resource_per_vnode: 0,
            resource_used: 0,
            revenues: Cash(0),
            v_nodes: v_nodes_arr.clone(),
            node_ids: node_ids.clone(),
            total_rent: 0,
        }
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

    // v_nodes should be sorted
    pub fn replace_v_node(&mut self, v_nodes: Vec<u64>, node_id: NodeId) {
        let old_v_nodes = &self.v_nodes;
        let old_node_ids = &self.node_ids;

        let mut new_v_nodes = Vec::<Vec<u64>>::new();
        let mut new_node_ids = Vec::<NodeId>::new();

        let mut new_v_nodes_idx = 0;
        let mut v_nodes_for_new_node = Vec::<u64>::new();

        for wrapper_idx in 0..old_v_nodes.len() {
            let mut v_nodes_wrapper = Vec::<u64>::new();
            for idx in 0..old_v_nodes.get(wrapper_idx).unwrap().len() {
                let new_v_node = match v_nodes.get(new_v_nodes_idx) {
                    Some(v) => *v,
                    None => 0,
                };

                if old_v_nodes
                    .get(wrapper_idx)
                    .unwrap()
                    .get(idx)
                    .unwrap()
                    .clone()
                    == new_v_node
                {
                    v_nodes_for_new_node.push(new_v_node);
                    new_v_nodes_idx += 1;
                } else {
                    v_nodes_wrapper.push(
                        old_v_nodes
                            .get(wrapper_idx)
                            .unwrap()
                            .get(idx)
                            .unwrap()
                            .clone(),
                    );
                }
            }

            new_v_nodes.push(v_nodes_wrapper);
            new_node_ids.push(*old_node_ids.get(wrapper_idx).unwrap());
        }

        new_v_nodes.push(v_nodes_for_new_node);
        new_node_ids.push(node_id);

        self.v_nodes = new_v_nodes;
        self.node_ids = new_node_ids;
    }

    pub fn only_manager(&self, caller: AccountId) -> Result<()> {
        if self.manager_id == caller {
            Ok(())
        } else {
            Err(UnauthorizedClusterManager)
        }
    }

    pub fn change_rent(&mut self, rent: Balance) {
        self.total_rent = rent;
    }
}
