//! The store where to create and access Nodes.
use ink_prelude::vec::Vec;
use ink_storage::Mapping;
use ink_storage::traits::{SpreadAllocate, SpreadLayout, StorageLayout};

use crate::ddc_bucket::cluster::entity::ClusterId;
use crate::ddc_bucket::node::entity::Node;
use crate::ddc_bucket::Error::UnknownNode;
use crate::ddc_bucket::{Balance, NodeId, Result};

#[derive(SpreadAllocate, SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(StorageLayout, Debug,))]
pub struct TopologyStore(Mapping<(ClusterId, u64), NodeId>);

impl TopologyStore {

    pub fn create_topology(
        &mut self,
        cluster_id: ClusterId,
        v_nodes: Vec<Vec<u64>>,
        nodes: Vec<(NodeId, &Node)>,
    ) -> Result<Balance> {
        let mut total_rent = 0u128;
        let mut vnodes_wrapper_index = 0;

        for node in &nodes {
            let v_nodes_for_node = &v_nodes[vnodes_wrapper_index as usize];
            for v_node in v_nodes_for_node.iter() {
                self.0.insert((cluster_id, *v_node), &(node.0));

                total_rent += node.1.rent_per_month as Balance;
            }

            vnodes_wrapper_index += 1;
        }

        Ok(total_rent)
    }

    pub fn replace_node(
        &mut self,
        cluster_id: u32,
        v_nodes: Vec<u64>,
        new_node_id: NodeId,
    ) -> Result<()> {
        for v_node in v_nodes {
            if self.0.contains(&(cluster_id, v_node)) {
                self.0.insert(&(cluster_id, v_node), &new_node_id);
            } else {
                return Err(UnknownNode)
            }
        }

        Ok(())
    }

    pub fn add_node(
        &mut self,
        cluster_id: u32,
        old_v_nodes: &Vec<u64>,
        v_nodes: &Vec<Vec<u64>>,
        nodes: Vec<(NodeId, &Node)>,
    ) -> Result<u32> {
        // remove old nodes from topology
        for &old_v_node in old_v_nodes {
            self.0.insert((cluster_id, old_v_node), &0);
        }

        let mut total_rent = 0u32;
        let mut index = 0u32;

        // reassign v_nodes to physical ones
        for node in nodes {
            let v_nodes_for_node = &v_nodes[index as usize];

            for v_node in v_nodes_for_node.iter() {
                self.0.insert((cluster_id, *v_node), &(node.0));

                total_rent += node.1.rent_per_month as u32;
            }

            index += 1;
        }

        Ok(total_rent)
    }

    pub fn get(&self, cluster_id: ClusterId, v_node: u64) -> Result<NodeId> {
        self.0.get((cluster_id, v_node)).ok_or(UnknownNode)
    }

    pub fn save(&mut self, cluster_id: ClusterId, v_node: u64, node_id: NodeId) {
        self.0.insert(&(cluster_id, v_node), &node_id)
    }
}
