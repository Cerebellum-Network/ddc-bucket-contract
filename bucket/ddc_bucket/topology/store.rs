//! The store where to create and access Nodes.
use ink_prelude::vec::Vec;
use ink_storage::Mapping;
use ink_storage::traits::{SpreadAllocate, SpreadLayout, StorageLayout};

use crate::ddc_bucket::cluster::entity::ClusterId;
use crate::ddc_bucket::node::entity::{NodeKey};
use crate::ddc_bucket::{Error::*, Result};

pub type VNodeToken = u64;
pub type ClusterVNode = (ClusterId, VNodeToken);

#[derive(SpreadAllocate, SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(StorageLayout, Debug))]
pub struct TopologyStore {
    // virtual nodes within a cluster
    clusters_map: Mapping<ClusterId, Vec<VNodeToken>>,
    // virtual nodes assigned to a physical node
    nodes_map: Mapping<NodeKey, Vec<VNodeToken>>,
    // physical node assigned to a virtual node
    vnodes_map: Mapping<ClusterVNode, NodeKey>,
}

impl TopologyStore {

    pub fn get_vnodes_by_cluster(&self, cluster_id: ClusterId) -> Result<Vec<VNodeToken>> {
        self.clusters_map.get(cluster_id).ok_or(TopologyDoesNotExist)
    }

    pub fn get_vnodes_by_node(&self, node_key: NodeKey) -> Result<Vec<VNodeToken>> {
        self.nodes_map.get(node_key).ok_or(VNodesDoNotExistsFor(node_key))
    }

    pub fn get_node_by_vnode(&self, cluster_id: ClusterId, v_node: VNodeToken) -> Result<NodeKey> {
        self.vnodes_map.get((cluster_id, v_node)).ok_or(VNodeIsNotAssigned(cluster_id, v_node))
    }

    pub fn create_topology(
        &mut self,
        cluster_id: ClusterId,
    ) -> Result<()> {
        if self.clusters_map.contains(&cluster_id) {
            Err(TopologyAlreadyExists)
        } else {
            let all_vnodes: Vec<VNodeToken> = Vec::new();
            self.clusters_map.insert(cluster_id, &all_vnodes);
            Ok(())
        }
    }

    pub fn add_node(
        &mut self,
        cluster_id: ClusterId,
        node_key: NodeKey,
        v_nodes: Vec<VNodeToken>,
    ) -> Result<()> {

        let mut all_vnodes = self.get_vnodes_by_cluster(cluster_id)?;

        for v_node in &v_nodes {

            // vnode that is being added should not exist in the cluster topology
            if let Some(node_key) = self.vnodes_map.get((cluster_id, v_node)) {
                return Err(VNodeIsAlreadyAssignedTo(node_key));
            }

            // vnode that is being added should be assined to the physical node
            self.vnodes_map.insert((cluster_id, v_node), &node_key);

            // vnode that is being added should be in added to the cluster topology
            all_vnodes.push(*v_node);
        }

        self.clusters_map.insert(cluster_id, &all_vnodes);
        
        // vnode that is being added should be assigned to the physical node
        self.nodes_map.insert(node_key, &v_nodes);

        Ok(())
    }

    pub fn replace_node(
        &mut self,
        cluster_id: ClusterId,
        new_node_key: NodeKey,
        v_nodes: Vec<VNodeToken>,
    ) -> Result<()> {

        let all_vnodes = self.get_vnodes_by_cluster(cluster_id)?;

        for v_node in &v_nodes {

            // vnode that is being reasigned should be in the cluster topology
            if None == all_vnodes.iter().position(|x| *x == *v_node) {
                return Err(VNodeDoesNotExists);
            };
            
            // vnode that is being reasigned should be already assigned to a physical node
            let old_node_key = self.get_node_by_vnode(cluster_id, *v_node)?;

            // vnode that is being reasigned should be among other vnodes assigned to the old physical node
            let mut old_node_vnodes = self.get_vnodes_by_node(old_node_key)?;

            // vnode that is being reasigned should be removed from the old physical node
            if let Some(pos) = old_node_vnodes.iter().position(|x| *x == *v_node) {
                old_node_vnodes.remove(pos);
            };
            self.nodes_map.insert(old_node_key, &old_node_vnodes);
            
            // vnode that is being reasigned should be assined to the new physical node
            self.vnodes_map.insert(&(cluster_id, *v_node), &new_node_key);
        }

        // vnode that is being reasigned should be among other vnodes assigned to the new physical node
        self.nodes_map.insert(new_node_key, &v_nodes);

        Ok(())
    }


}
