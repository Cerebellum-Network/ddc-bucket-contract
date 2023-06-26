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
        self.clusters_map.get(cluster_id).ok_or(ClusterIsNotInitialized(cluster_id))
    }

    pub fn get_vnodes_by_node(&self, node_key: NodeKey) -> Result<Vec<VNodeToken>> {
        self.nodes_map.get(node_key).ok_or(NodeHasNoAssignedVNodes(node_key))
    }

    pub fn get_node_by_vnode(&self, cluster_id: ClusterId, v_node: VNodeToken) -> Result<NodeKey> {
        self.vnodes_map.get((cluster_id, v_node)).ok_or(VNodeInNotAssignedToNode(cluster_id, v_node))
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

        let mut all_vnodes: Vec<u64> = self.get_vnodes_by_cluster(cluster_id)?;

        for v_node in &v_nodes {

            // vnode that is being added should not exist in the cluster topology
            if let Some(node_key) = self.vnodes_map.get((cluster_id, v_node)) {
                return Err(VNodeIsAlreadyAssignedToNode(node_key));
            }

            // vnode that is being added should be assigned to the physical node
            self.vnodes_map.insert((cluster_id, v_node), &node_key);

            // vnode that is being added should be added to the cluster topology
            all_vnodes.push(*v_node);
        }

        self.clusters_map.insert(cluster_id, &all_vnodes);
        
        // vnodes that are being added should be assigned to the physical node
        self.nodes_map.insert(node_key, &v_nodes);

        Ok(())
    }


    pub fn remove_node(
        &mut self,
        cluster_id: ClusterId,
        node_key: NodeKey
    ) -> Result<()> {

        let mut all_vnodes: Vec<u64> = self.get_vnodes_by_cluster(cluster_id)?;
        let v_nodes = self.get_vnodes_by_node(node_key)?;

        for v_node in &v_nodes {

            // vnode that is being removed should exist in the cluster topology
            if None == self.vnodes_map.get((cluster_id, *v_node)) {
                return Err(VNodeInNotAssignedToNode(cluster_id, *v_node));
            }
            
            // vnode that is being removed should be unusigned from the physical node
            self.vnodes_map.remove((cluster_id, v_node));

            // vnode that is being removed should be removed from the cluster topology
            if let Some(pos) = all_vnodes.iter().position(|x| *x == *v_node) {
                all_vnodes.remove(pos);
            };
        }

        self.clusters_map.insert(cluster_id, &all_vnodes);

        // vnodes that are being removed should be unusigned from the physical node
        self.nodes_map.remove(node_key);

        Ok(())

    }
    

    pub fn replace_node(
        &mut self,
        cluster_id: ClusterId,
        new_node_key: NodeKey,
        v_nodes_to_reasign: Vec<VNodeToken>,
    ) -> Result<()> {

        let all_vnodes = self.get_vnodes_by_cluster(cluster_id)?;

        for v_node in &v_nodes_to_reasign {

            // vnode that is being reasigned should be in the cluster topology
            if None == all_vnodes.iter().position(|x| *x == *v_node) {
                return Err(VNodeDoesNotExistsInCluster(cluster_id));
            };
            
            // vnode that is being reasigned should be already assigned to a physical node
            let old_node_key = self.get_node_by_vnode(cluster_id, *v_node)?;

            // vnode that is being reasigned should be among other vnodes assigned to the old physical node
            let mut old_node_vnodes = self.get_vnodes_by_node(old_node_key)?;

            // vnode that is being reasigned should be unasigned from the old physical node
            if let Some(pos) = old_node_vnodes.iter().position(|x| *x == *v_node) {
                old_node_vnodes.remove(pos);
            };
            self.nodes_map.insert(old_node_key, &old_node_vnodes);
            
            // vnode that is being reasigned should be assined to the new physical node
            self.vnodes_map.insert(&(cluster_id, *v_node), &new_node_key);
        }

        // vnodes that are being reasigned should be among other vnodes assigned to the new physical node
        self.nodes_map.insert(new_node_key, &v_nodes_to_reasign);

        Ok(())
    }


    pub fn remove_topology(
        &mut self,
        cluster_id: ClusterId,
    ) -> Result<()> {

        let all_vnodes: Vec<u64> = self.get_vnodes_by_cluster(cluster_id)?;

        for v_node in &all_vnodes {
            let node_key = self.get_node_by_vnode(cluster_id, *v_node)?;
            self.vnodes_map.remove((cluster_id, v_node));
            self.nodes_map.remove(node_key);   
        }

        self.clusters_map.remove(cluster_id);

        Ok(())
    }


}
