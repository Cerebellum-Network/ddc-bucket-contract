//! The store where to create and access Nodes.
use ink_prelude::vec::Vec;
use ink_storage::Mapping;
use ink_storage::traits::{SpreadAllocate, SpreadLayout};

use crate::ddc_bucket::cluster::entity::ClusterId;
use crate::ddc_bucket::node::entity::{NodeKey};
use crate::ddc_bucket::{Error::*, Result};

pub type VNodeToken = u64;
pub type ClusterVNode = (ClusterId, VNodeToken);

// https://use.ink/datastructures/storage-layout#packed-vs-non-packed-layout
// There is a buffer with only limited capacity (around 16KB in the default configuration) available.
pub const MAX_V_NODE_IN_VECTOR: usize = 1800;

#[derive(SpreadAllocate, SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(ink_storage::traits::StorageLayout, Debug))]
pub struct TopologyStore {
    // virtual nodes assigned to a physical node
    v_nodes_map: Mapping<NodeKey, Vec<VNodeToken>>,
    // physical node assigned to a virtual node
    nodes_map: Mapping<ClusterVNode, NodeKey>,
    // virtual nodes within a cluster assigned to all its physical nodes
    cluster_v_nodes_map: Mapping<ClusterId, Vec<VNodeToken>>,
}

impl TopologyStore {

    pub fn get_v_nodes_by_cluster(&self, cluster_id: ClusterId) -> Vec<VNodeToken> {
        self.cluster_v_nodes_map.get(cluster_id).unwrap_or(Vec::new())
    }

    pub fn get_v_nodes_by_node(&self, node_key: NodeKey) -> Vec<VNodeToken> {
        self.v_nodes_map.get(node_key).unwrap_or(Vec::new())
    }

    pub fn get_node_by_v_node(&self, cluster_id: ClusterId, v_node: VNodeToken) -> Result<NodeKey> {
        self.nodes_map.get((cluster_id, v_node)).ok_or(VNodeIsNotAssignedToNode(cluster_id, v_node))
    }

    pub fn v_node_has_node(&self, cluster_id: ClusterId, v_node: VNodeToken) -> bool {
        self.nodes_map.contains((cluster_id, v_node))
    }

    pub fn create_topology(
        &mut self,
        cluster_id: ClusterId,
    ) -> Result<()> {
        if self.cluster_v_nodes_map.contains(&cluster_id) {
            Err(TopologyAlreadyExists)
        } else {
            let cluster_v_nodes: Vec<VNodeToken> = Vec::new();
            self.cluster_v_nodes_map.insert(cluster_id, &cluster_v_nodes);
            Ok(())
        }
    }

    pub fn add_node(
        &mut self,
        cluster_id: ClusterId,
        node_key: NodeKey,
        v_nodes: Vec<VNodeToken>,
    ) -> Result<()> {

        if v_nodes.is_empty() {
            return Err(AtLeastOneVNodeHasToBeAssigned(cluster_id, node_key));
        }

        if v_nodes.len() > MAX_V_NODE_IN_VECTOR {
            return Err(VNodesSizeExceedsLimit);
        }

        let mut cluster_v_nodes = self.get_v_nodes_by_cluster(cluster_id);

        if cluster_v_nodes.len() + v_nodes.len() > MAX_V_NODE_IN_VECTOR {
            return Err(VNodesSizeExceedsLimit);
        }

        for v_node in &v_nodes {

            // vnode that is being added should not exist in the cluster topology
            if self.v_node_has_node(cluster_id, *v_node) {
                return Err(VNodeIsAlreadyAssignedToNode(node_key));
            }

            // vnode that is being added should be assigned to the physical node
            self.nodes_map.insert((cluster_id, v_node), &node_key);

            // vnode that is being added should be added to the cluster topology
            cluster_v_nodes.push(*v_node);
        }

        self.cluster_v_nodes_map.insert(cluster_id, &cluster_v_nodes);
        
        // vnodes that are being added should be assigned to the physical node
        self.v_nodes_map.insert(node_key, &v_nodes);

        Ok(())
    }


    pub fn remove_node(
        &mut self,
        cluster_id: ClusterId,
        node_key: NodeKey
    ) -> Result<()> {

        let mut cluster_v_nodes = self.get_v_nodes_by_cluster(cluster_id);
        let v_nodes = self.get_v_nodes_by_node(node_key);

        for v_node in &v_nodes {

            // vnode that is being removed should exist in the cluster topology
            if !self.v_node_has_node(cluster_id, *v_node) {
                return Err(VNodeIsNotAssignedToNode(cluster_id, *v_node));
            }
            
            // vnode that is being removed should be unusigned from the physical node
            self.nodes_map.remove((cluster_id, v_node));

            // vnode that is being removed should be removed from the cluster topology
            if let Some(pos) = cluster_v_nodes.iter().position(|x| *x == *v_node) {
                cluster_v_nodes.remove(pos);
            };
        }

        self.cluster_v_nodes_map.insert(cluster_id, &cluster_v_nodes);

        // vnodes that are being removed should be unusigned from the physical node
        self.v_nodes_map.remove(node_key);

        Ok(())

    }
    

    pub fn replace_node(
        &mut self,
        cluster_id: ClusterId,
        new_node_key: NodeKey,
        v_nodes_to_reasign: Vec<VNodeToken>,
    ) -> Result<()> {

        if v_nodes_to_reasign.is_empty() {
            return Err(AtLeastOneVNodeHasToBeAssigned(cluster_id, new_node_key));
        }

        if v_nodes_to_reasign.len() > MAX_V_NODE_IN_VECTOR {
            return Err(VNodesSizeExceedsLimit);
        }

        let cluster_v_nodes = self.get_v_nodes_by_cluster(cluster_id);

        for v_node in &v_nodes_to_reasign {

            // vnode that is being reasigned should be in the cluster topology
            if None == cluster_v_nodes.iter().position(|x| *x == *v_node) {
                return Err(VNodeDoesNotExistsInCluster(cluster_id));
            };
            
            // vnode that is being reasigned should be already assigned to a physical node
            let old_node_key = self.get_node_by_v_node(cluster_id, *v_node)?;

            // vnode that is being reasigned should be among other vnodes assigned to the old physical node
            let mut old_node_v_nodes = self.get_v_nodes_by_node(old_node_key);

            // vnode that is being reasigned should be unasigned from the old physical node
            if let Some(pos) = old_node_v_nodes.iter().position(|x| *x == *v_node) {
                old_node_v_nodes.remove(pos);
            };

            if old_node_v_nodes.is_empty() {
                return Err(AtLeastOneVNodeHasToBeAssigned(cluster_id, old_node_key));
            }

            self.v_nodes_map.insert(old_node_key, &old_node_v_nodes);
            
            // vnode that is being reasigned should be assined to the new physical node
            self.nodes_map.insert(&(cluster_id, *v_node), &new_node_key);
        }

        // vnodes that are being reasigned should be among other vnodes assigned to the new physical node
        let mut new_node_v_nodes = self.get_v_nodes_by_node(new_node_key);

        if new_node_v_nodes.len() + v_nodes_to_reasign.len() > MAX_V_NODE_IN_VECTOR {
            return Err(VNodesSizeExceedsLimit);
        }

        new_node_v_nodes.extend(v_nodes_to_reasign);
        self.v_nodes_map.insert(new_node_key, &new_node_v_nodes);

        Ok(())
    }


    pub fn remove_topology(
        &mut self,
        cluster_id: ClusterId,
    ) -> Result<()> {

        let cluster_v_nodes: Vec<u64> = self.get_v_nodes_by_cluster(cluster_id);

        for v_node in &cluster_v_nodes {
            let node_key = self.get_node_by_v_node(cluster_id, *v_node)?;
            self.nodes_map.remove((cluster_id, v_node));
            self.v_nodes_map.remove(node_key);   
        }

        self.cluster_v_nodes_map.remove(cluster_id);

        Ok(())
    }


}
