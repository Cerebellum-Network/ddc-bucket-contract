use std::collections::HashMap;

use crate::ddc_bucket::{AccountId, DdcBucket};
use crate::ddc_bucket::cluster::entity::{PartitionId, PartitionIndex};
use crate::ddc_bucket::vnode::entity::VNodeId;

pub struct ClusterManager {
    pub account_id: AccountId,

    node_states: HashMap<VNodeId, NodeState>,
}

enum NodeState {
    Default,
    Dead,
}

impl ClusterManager {
    pub fn new(account_id: AccountId) -> Self {
        Self { account_id, node_states: Default::default() }
    }

    pub fn replace_node(&mut self, contract: &mut DdcBucket, old_vnode_id: VNodeId) {
        self.node_states.insert(old_vnode_id, NodeState::Dead);

        let new_vnode_id = self.find_a_free_node(contract);

        let partition_ids = self.find_partitions_of_node(contract, old_vnode_id);

        for (cluster_id, partition_i) in partition_ids.iter() {
            contract.cluster_replace_vnode(*cluster_id, *partition_i, new_vnode_id).unwrap();
        }
    }

    pub fn find_partitions_of_node(&self, contract: &DdcBucket, vnode_id: VNodeId) -> Vec<PartitionId> {
        let mut partition_ids = Vec::new();

        // Discover the available clusters.
        let (clusters, _count) = contract.cluster_list(0, 20);
        if _count > 20 { unimplemented!("full iteration of contract entities") }

        for cluster in clusters.iter() {
            if cluster.manager != self.account_id {
                continue; // Not our cluster, skip.
            }

            for (index, &some_vnode_id) in cluster.vnode_ids.iter().enumerate() {
                if some_vnode_id == vnode_id {
                    let partition_id = (cluster.cluster_id, index as PartitionIndex);
                    partition_ids.push(partition_id);
                }
            }
        }

        partition_ids
    }

    pub fn find_a_free_node(&self, contract: &DdcBucket) -> VNodeId {
        // Discover the nodes
        let (vnodes, _count) = contract.vnode_list(0, 20, None);
        if _count > 20 { unimplemented!("full iteration of contract entities") }

        let vnode = vnodes.iter().find(|n| {
            let node_state = self.node_states.get(&n.vnode_id);
            match node_state {
                Some(&NodeState::Dead) => false,
                _ => true,
            }
        }).expect("no good nodes available");

        vnode.vnode_id
    }
}
