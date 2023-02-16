//! The store where to create and access Nodes.
use ink_prelude::string::String;
use ink_storage::traits::{SpreadLayout, StorageLayout};
use ink_storage::{collections::HashMap, collections::Vec as InkVec};

use crate::ddc_bucket::cluster::entity::ClusterId;
use crate::ddc_bucket::node::entity::{Node, Resource};
use crate::ddc_bucket::Error::UnknownNode;
use crate::ddc_bucket::{self, NodeId, Result};

// (clusterID + vnode) = nodeID
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
        v_nodes: Vec<Vec<u64>>,
        nodes: Vec<NodeId>,
    ) -> Result<u32> {
        let node_count = nodes.len();

        let mut total_rent = 0u32;
        for (nodeId, node) in nodes {
            let v_nodes_for_node = v_nodes[*nodeId];

            for v_node in v_nodes_for_node.iter() {
                self.0.insert((cluster_id, v_node), nodeId);

                total_rent += node.rent_per_month;
            }
        }

        Ok(total_rent)
    }

    pub fn get_nodeId(&mut self, cluster_id: ClusterId, v_node: u64) -> Result<&NodeId> {
        self.0.get(&(cluster_id, v_node)).ok_or(UnknownNode)
    }
}
