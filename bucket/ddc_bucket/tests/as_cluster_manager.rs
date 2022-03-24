use std::collections::HashMap;

use crate::ddc_bucket::{AccountId, DdcBucket};
use crate::ddc_bucket::cluster::entity::{PartitionId, PartitionIndex};
use crate::ddc_bucket::node::entity::{NodeId, Resource};
use crate::ddc_bucket::tests::env_utils::{CONTRACT_FEE_LIMIT, pop_caller, push_caller_value};
use crate::ddc_bucket::tests::topology::Topology;

pub struct ClusterManager {
    pub account_id: AccountId,

    node_states: HashMap<NodeId, NodeState>,
}

#[derive(PartialEq)]
enum NodeState {
    _Default,
    Dead,
}

impl ClusterManager {
    pub fn new(account_id: AccountId) -> Self {
        Self { account_id, node_states: Default::default() }
    }

    pub fn create_cluster(&self, contract: &mut DdcBucket, engine_name: &str, partition_count: u32) {
        let (nodes, count) = contract.node_list(0, 20, None);
        if count > 20 { unimplemented!("full iteration of contract entities") }
        let node_ids = nodes.iter()
            .filter(|n| n.node_params.contains(engine_name))
            .map(|n| n.node_id)
            .collect();

        let topology = Topology::new(engine_name, partition_count);

        push_caller_value(self.account_id, CONTRACT_FEE_LIMIT);
        let _id = contract.cluster_create(
            self.account_id,
            partition_count,
            node_ids,
            topology.to_string().unwrap(),
        ).unwrap();
        pop_caller();
    }

    pub fn replace_node(&mut self, contract: &mut DdcBucket, old_node_id: NodeId) {
        self.node_states.insert(old_node_id, NodeState::Dead);

        let partition_ids = self.find_partitions_of_node(contract, old_node_id);

        for (cluster_id, partition_i) in partition_ids.iter() {
            let resource_needed = contract.cluster_get(*cluster_id).unwrap().resource_per_vnode;
            let new_node_id = self.find_a_free_node(contract, resource_needed);
            contract.cluster_replace_node(*cluster_id, *partition_i, new_node_id).unwrap();
        }
    }

    pub fn find_partitions_of_node(&self, contract: &DdcBucket, node_id: NodeId) -> Vec<PartitionId> {
        let mut partition_ids = Vec::new();

        // Discover the available clusters.
        let (clusters, count) = contract.cluster_list(0, 20);
        if count > 20 { unimplemented!("full iteration of contract entities") }

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

    pub fn find_a_free_node(&self, contract: &DdcBucket, resource_needed: Resource) -> NodeId {
        // Discover the nodes
        let (nodes, count) = contract.node_list(0, 20, None);
        if count > 20 { unimplemented!("full iteration of contract entities") }

        let node = nodes.iter().find(|node| {
            if node.free_resource < resource_needed { return false; }

            let node_state = self.node_states.get(&node.node_id);
            if node_state == Some(&NodeState::Dead) { return false; }

            return true;
        }).expect("no good nodes available");

        node.node_id
    }
}
