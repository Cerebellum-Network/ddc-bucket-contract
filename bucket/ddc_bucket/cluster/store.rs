use ink_prelude::vec::Vec;
use ink_storage::{
    collections::Vec as InkVec,
    traits,
};

use crate::ddc_bucket::{AccountId, Error::*, Result, VNodeId};
use crate::ddc_bucket::contract_fee::SIZE_INDEX;

use super::entity::{Cluster, ClusterId, ClusterParams};

#[derive(traits::SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout, Debug))]
pub struct ClusterStore(pub InkVec<Cluster>);

impl ClusterStore {
    pub fn create(
        &mut self,
        manager: AccountId,
        partition_count: u32,
        node_ids: Vec<VNodeId>,
        cluster_params: ClusterParams,
    ) -> (ClusterId, usize) {
        let cluster_id = self.0.len();
        let cluster = Cluster {
            cluster_id,
            manager,
            cluster_params,
            vnode_ids: Self::new_partitions(partition_count as usize, node_ids),
        };
        let record_size = cluster.new_size();
        self.0.push(cluster);
        (cluster_id, record_size)
    }

    fn new_partitions(partition_count: usize, node_ids: Vec<VNodeId>) -> Vec<VNodeId> {
        let node_count = node_ids.len();
        let mut vnode_ids = Vec::with_capacity(partition_count);
        for i in 0..partition_count {
            vnode_ids.push(node_ids[i % node_count]);
        }
        vnode_ids
    }

    pub fn add_vnode(&mut self, cluster_id: ClusterId, vnode_id: VNodeId) -> Result<usize> {
        let cluster = self.get_mut(cluster_id)?;
        cluster.vnode_ids.push(vnode_id);
        Ok(SIZE_INDEX)
    }

    pub fn get(&self, cluster_id: ClusterId) -> Result<&Cluster> {
        self.0.get(cluster_id).ok_or(ClusterDoesNotExist)
    }

    pub fn get_mut(&mut self, cluster_id: ClusterId) -> Result<&mut Cluster> {
        self.0.get_mut(cluster_id).ok_or(ClusterDoesNotExist)
    }

    pub fn list(&self, offset: u32, limit: u32) -> (Vec<Cluster>, u32) {
        let mut clusters = Vec::with_capacity(limit as usize);
        for cluster_id in offset..offset + limit {
            let cluster = match self.0.get(cluster_id) {
                None => break, // No more items, stop.
                Some(cluster) => cluster,
            };
            clusters.push(cluster.clone());
        }
        (clusters, self.0.len())
    }
}
