//! The store where to create and access Clusters by ID.

use ink_prelude::vec::Vec;
use ink_storage::{
    collections::Vec as InkVec,
    traits,
};

use crate::ddc_bucket::{AccountId, Balance, Error::*, NodeId, Result};
use crate::ddc_bucket::cash::Cash;
use crate::ddc_bucket::contract_fee::SIZE_INDEX;
use crate::ddc_bucket::node::entity::Node;

use super::entity::{Cluster, ClusterId, ClusterParams};

#[derive(traits::SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout, Debug))]
pub struct ClusterStore(pub InkVec<Cluster>);

impl ClusterStore {
    pub fn create(
        &mut self,
        manager_id: AccountId,
        partition_count: u32,
        nodes: &[&Node],
        cluster_params: ClusterParams,
    ) -> (ClusterId, usize) {
        let cluster_id = self.0.len();
        let (vnodes, total_rent) = Self::new_vnodes(partition_count as usize, nodes);
        let cluster = Cluster {
            cluster_id,
            manager_id,
            cluster_params,
            vnodes,
            resource_per_vnode: 0,
            resource_used: 0,
            revenues: Cash(0),
            total_rent,
        };
        let record_size = cluster.new_size();
        self.0.push(cluster);
        (cluster_id, record_size)
    }

    fn new_vnodes(partition_count: usize, node_ids: &[&Node]) -> (Vec<NodeId>, Balance) {
        let node_count = node_ids.len();
        let mut vnode_ids = Vec::with_capacity(partition_count);
        let mut total_rent = 0;
        for i in 0..partition_count {
            let node = &node_ids[i % node_count];
            vnode_ids.push(node.node_id);
            total_rent += node.rent_per_month;
        }
        // TODO: consider using the max rent instead of average rent.
        (vnode_ids, total_rent)
    }

    pub fn add_node(&mut self, cluster_id: ClusterId, node_id: NodeId) -> Result<usize> {
        let cluster = self.get_mut(cluster_id)?;
        cluster.vnodes.push(node_id);
        Ok(SIZE_INDEX)
    }

    pub fn get(&self, cluster_id: ClusterId) -> Result<&Cluster> {
        self.0.get(cluster_id).ok_or(ClusterDoesNotExist)
    }

    pub fn get_mut(&mut self, cluster_id: ClusterId) -> Result<&mut Cluster> {
        self.0.get_mut(cluster_id).ok_or(ClusterDoesNotExist)
    }

    pub fn list(&self, offset: u32, limit: u32, filter_manager_id: Option<AccountId>) -> (Vec<Cluster>, u32) {
        let mut clusters = Vec::with_capacity(limit as usize);
        for cluster_id in offset..offset + limit {
            let cluster = match self.0.get(cluster_id) {
                None => break, // No more items, stop.
                Some(cluster) => cluster,
            };
            // Apply the filter if given.
            if let Some(manager_id) = filter_manager_id {
                if manager_id != cluster.manager_id {
                    continue; // Skip non-matches.
                }
            }
            clusters.push(cluster.clone());
        }
        (clusters, self.0.len())
    }
}
