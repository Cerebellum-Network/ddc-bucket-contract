use ink_prelude::vec::Vec;
use ink_storage::{
    collections::Vec as InkVec,
    traits,
};

use crate::ddc_bucket::{Balance, Error::*, Result};

use super::entity::{Cluster, ClusterId, ClusterParams};

#[derive(traits::SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout, Debug))]
pub struct ClusterStore(pub InkVec<Cluster>);

impl ClusterStore {
    pub fn create(&mut self, rent_per_month: Balance, cluster_params: ClusterParams) -> ClusterId {
        let cluster_id = self.0.len();
        let cluster = Cluster {
            cluster_id,
            rent_per_month,
            cluster_params,
            service_ids: Vec::new(),
        };
        self.0.push(cluster);
        cluster_id
    }

    pub fn get(&self, cluster_id: ClusterId) -> Result<&Cluster> {
        self.0.get(cluster_id).ok_or(ClusterDoesNotExist)
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
