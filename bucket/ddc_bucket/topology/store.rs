//! The store where to create and access Nodes.
use ink_prelude::vec::Vec as InkVec;
use ink_storage::collections::HashMap;
use ink_storage::traits::{SpreadLayout, StorageLayout};

use crate::ddc_bucket::cluster::entity::ClusterId;
use crate::ddc_bucket::node::entity::Node;
use crate::ddc_bucket::Error::UnknownNode;
use crate::ddc_bucket::{NodeId, Result};

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
    ) -> Result<u32> {
        let mut total_rent = 0u32;
        for node in &nodes {
            let v_nodes_for_node = &v_nodes[node.0 as usize];

            for v_node in v_nodes_for_node.iter() {
                self.0.insert((cluster_id, *v_node), node.0);

                total_rent += node.1.rent_per_month as u32;
            }
        }

        Ok(total_rent)
    }

    pub fn topology_replace_node(
        &mut self,
        cluster_id: u64,
        v_nodes: Vec<u64>,
        node_id: NodeId,
    ) -> Result<()> {
        for v_node in v_nodes {
            let node_id = match self.0.get_mut(&(cluster_id, v_node)) {
                Some(node_id) => node_id,
                None => Err(UnknownNode),
            };

            // remap physical node to virtual one
            node_id = node_id;
        }

        Ok(())
    }

    pub fn get_node_id(&mut self, cluster_id: ClusterId, v_node: u64) -> Result<&NodeId> {
        self.0.get(&(cluster_id, v_node)).ok_or(UnknownNode)
    }
}
