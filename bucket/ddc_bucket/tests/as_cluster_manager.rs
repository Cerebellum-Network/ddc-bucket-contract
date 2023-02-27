use std::collections::HashMap;

use crate::ddc_bucket::cluster::entity::{VNodeId, VNodeIndex};
use crate::ddc_bucket::node::entity::{NodeId, Resource};
use crate::ddc_bucket::tests::as_storage::STORAGE_ENGINE;
use crate::ddc_bucket::tests::env_utils::{pop_caller, push_caller_value, CONTRACT_FEE_LIMIT};
use crate::ddc_bucket::tests::topology::Topology;
use crate::ddc_bucket::{AccountId, DdcBucket};

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
        Self {
            account_id,
            node_states: Default::default(),
        }
    }

    pub fn create_cluster(&self, contract: &mut DdcBucket, engine_name: &str, vnode_count: u32) {
        let (nodes, count) = contract.node_list(0, 20, None);
        if count > 20 {
            unimplemented!("full iteration of contract entities")
        }
        let node_ids = nodes
            .iter()
            .filter(|n| n.params.contains(engine_name))
            .map(|n| n.node_id)
            .collect();

        let topology = Topology::new(engine_name, vnode_count);

        push_caller_value(self.account_id, CONTRACT_FEE_LIMIT);
        let _id = contract.cluster_create(
            self.account_id,
            vnode_count,
            node_ids,
            topology.to_string().unwrap(),
        );
        pop_caller();

        push_caller_value(self.account_id, 0);
        // Reserve some resources.
        contract.cluster_reserve_resource(_id, 5);
        // Later, reserve more.
        contract.cluster_reserve_resource(_id, 10);
        pop_caller();
    }

    pub fn replace_node(&mut self, contract: &mut DdcBucket, old_node_id: NodeId) {
        self.node_states.insert(old_node_id, NodeState::Dead);

        let vnode_ids = self.find_vnodes_of_node(contract, old_node_id);

        for (cluster_id, vnode_i) in vnode_ids.iter() {
            let resource_needed = contract
                .cluster_get(*cluster_id)
                .unwrap()
                .cluster
                .resource_per_vnode;
            let new_node_id = self.find_best_storage_node(contract, resource_needed);
            contract.cluster_replace_node(*cluster_id, *vnode_i, new_node_id);
        }
    }

    pub fn find_vnodes_of_node(&self, contract: &DdcBucket, node_id: NodeId) -> Vec<VNodeId> {
        let mut vnode_ids = Vec::new();

        // Discover the available clusters.
        let (clusters, count) = contract.cluster_list(0, 20, None);
        if count > 20 {
            unimplemented!("full iteration of contract entities")
        }

        for cluster in clusters.iter() {
            if cluster.cluster.manager_id != self.account_id {
                continue; // Not our cluster, skip.
            }

            for (index, &some_node_id) in cluster.cluster.vnodes.iter().enumerate() {
                if some_node_id == node_id {
                    let vnode_id = (cluster.cluster_id, index as VNodeIndex);
                    vnode_ids.push(vnode_id);
                }
            }
        }

        vnode_ids
    }

    pub fn find_best_storage_node(
        &self,
        contract: &DdcBucket,
        resource_needed: Resource,
    ) -> NodeId {
        // Discover the nodes
        let (nodes, count) = contract.node_list(0, 20, None);
        if count > 20 {
            unimplemented!("full iteration of contract entities")
        }

        // Return the ID of the best available node.
        nodes
            .iter()
            .filter(|n| n.params.contains(STORAGE_ENGINE))
            .filter(|n| n.node.free_resource >= resource_needed)
            .filter(|n| {
                let node_state = self.node_states.get(&n.node_id);
                match node_state {
                    Some(&NodeState::Dead) => false,
                    _ => true,
                }
            })
            .max_by_key(|n| n.node.free_resource)
            .map(|n| n.node_id)
            .expect("no node available")
    }
}
