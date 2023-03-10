//! The store where to create and access Nodes.
use ink_prelude::vec::Vec as InkVec;
use ink_storage::collections::HashMap;
use ink_storage::traits::{SpreadLayout, StorageLayout};

use crate::ddc_bucket::cluster::entity::ClusterId;
use crate::ddc_bucket::node::entity::Node;
use crate::ddc_bucket::Error::UnknownNode;
use crate::ddc_bucket::{Balance, NodeId, Result};

#[derive(SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(StorageLayout, Debug,))]
pub struct TopologyStore(HashMap<(ClusterId, u64), NodeId>);

impl TopologyStore {
    pub fn new_topology_store() -> Self {
        TopologyStore(HashMap::<(ClusterId, u64), NodeId>::new())
    }

    pub fn create_topology(
        &mut self,
        cluster_id: ClusterId,
        v_nodes: InkVec<InkVec<u64>>,
        nodes: InkVec<(NodeId, &Node)>,
    ) -> Result<Balance> {
        let mut total_rent = 0u128;
        for node in &nodes {
            let vnodes_wrapper_index = node.0 - 1;
            let v_nodes_for_node = &v_nodes[vnodes_wrapper_index as usize];

            for v_node in v_nodes_for_node.iter() {
                self.0.insert((cluster_id, *v_node), node.0);

                total_rent += node.1.rent_per_month as Balance;
            }
        }

        Ok(total_rent)
    }

    pub fn replace_node(
        &mut self,
        cluster_id: u32,
        v_nodes: InkVec<u64>,
        new_node_id: NodeId,
    ) -> Result<()> {
        for v_node in v_nodes {
            let node_id = match self.0.get_mut(&(cluster_id, v_node)) {
                Some(node_id) => node_id,
                None => return Err(UnknownNode),
            };

            // remap physical node to virtual one
            *node_id = new_node_id;
        }

        Ok(())
    }

    pub fn add_node(
        &mut self,
        cluster_id: u32,
        old_v_nodes: &InkVec<u64>,
        v_nodes: &InkVec<InkVec<u64>>,
        nodes: InkVec<(NodeId, &Node)>,
    ) -> Result<u32> {
        // remove old nodes from topology
        for old_v_node in old_v_nodes {
            self.0.insert((cluster_id, *old_v_node), 0);
        }

        let mut total_rent = 0u32;

        // reassign v_nodes to physical ones
        for node in nodes {
            let v_nodes_for_node = &v_nodes[node.0 as usize];

            for v_node in v_nodes_for_node.iter() {
                self.0.insert((cluster_id, *v_node), node.0);

                total_rent += node.1.rent_per_month as u32;
            }
        }

        Ok(total_rent)
    }

    pub fn get_node_id(&self, cluster_id: ClusterId, v_node: u64) -> Result<&NodeId> {
        self.0.get(&(cluster_id, v_node)).ok_or(UnknownNode)
    }

    pub fn get_node_id_mut(&mut self, cluster_id: ClusterId, v_node: u64) -> Result<&mut NodeId> {
        self.0.get_mut(&(cluster_id, v_node)).ok_or(UnknownNode)
    }
}
