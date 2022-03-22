use ink_prelude::vec::Vec;
use ink_storage::{
    collections::Vec as InkVec,
    traits,
};

use crate::ddc_bucket::{Error::*, Result, VNodeId};

use super::entity::{Cluster, ClusterId, ClusterParams};
use crate::ddc_bucket::contract_fee::SIZE_INDEX;

#[derive(traits::SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout, Debug))]
pub struct ClusterStore(pub InkVec<Cluster>);

impl ClusterStore {
    pub fn create(&mut self, cluster_params: ClusterParams) -> (ClusterId, usize) {
        let cluster_id = self.0.len();
        let cluster = Cluster {
            cluster_id,
            cluster_params,
            vnode_ids: Vec::new(),
        };
        let record_size = cluster.new_size();
        self.0.push(cluster);
        (cluster_id, record_size)
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
