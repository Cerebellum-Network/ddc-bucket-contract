use std::collections::HashMap;

use crate::ddc_bucket::{AccountId, DdcBucket};
use crate::ddc_bucket::cluster::entity::{PartitionId, PartitionIndex};
use crate::ddc_bucket::node::entity::NodeId;

pub struct ClusterManager {
    pub account_id: AccountId,

    node_states: HashMap<NodeId, NodeState>,
}

enum NodeState {
    _Default,
    Dead,
}

impl ClusterManager {
    pub fn new(account_id: AccountId) -> Self {
        Self { account_id, node_states: Default::default() }
    }

    pub fn replace_node(&mut self, contract: &mut DdcBucket, old_node_id: NodeId) {
        self.node_states.insert(old_node_id, NodeState::Dead);

        let new_node_id = self.find_a_free_node(contract);

        let partition_ids = self.find_partitions_of_node(contract, old_node_id);

        for (cluster_id, partition_i) in partition_ids.iter() {
            contract.cluster_replace_node(*cluster_id, *partition_i, new_node_id).unwrap();
        }
    }

    pub fn find_partitions_of_node(&self, contract: &DdcBucket, node_id: NodeId) -> Vec<PartitionId> {
        let mut partition_ids = Vec::new();

        // Discover the available clusters.
        let (clusters, _count) = contract.cluster_list(0, 20);
        if _count > 20 { unimplemented!("full iteration of contract entities") }

        for cluster in clusters.iter() {
            if cluster.manager != self.account_id {
                continue; // Not our cluster, skip.
            }

            for (index, &some_node_id) in cluster.vnodes.iter().enumerate() {
                if some_node_id == node_id {
                    let partition_id = (cluster.cluster_id, index as PartitionIndex);
                    partition_ids.push(partition_id);
                }
            }
        }

        partition_ids
    }

    pub fn find_a_free_node(&self, contract: &DdcBucket) -> NodeId {
        // Discover the nodes
        let (nodes, _count) = contract.node_list(0, 20, None);
        if _count > 20 { unimplemented!("full iteration of contract entities") }

        let node = nodes.iter().find(|n| {
            let node_state = self.node_states.get(&n.node_id);
            match node_state {
                Some(&NodeState::Dead) => false,
                _ => true,
            }
        }).expect("no good nodes available");

        node.node_id
    }
}
