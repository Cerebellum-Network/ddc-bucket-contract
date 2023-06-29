//! The store where to create and access Clusters by ID.

use ink_storage::traits::{SpreadAllocate, SpreadLayout};
use ink_storage::Mapping;
use crate::ddc_bucket::{AccountId, Error::*, Result};
use super::entity::{Cluster, ClusterId, ClusterParams};


#[derive(SpreadAllocate, SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(ink_storage::traits::StorageLayout, Debug))]
pub struct ClusterStore {
    pub next_cluster_id: u32,
    pub clusters: Mapping<ClusterId, Cluster>,
}

impl ClusterStore {
    pub fn create(
        &mut self,
        manager_id: AccountId,
        cluster_params: ClusterParams,
    ) -> Result<ClusterId> {
        let cluster_id = self.next_cluster_id;
        self.next_cluster_id = self.next_cluster_id + 1;

        let cluster = Cluster::new(
            manager_id, 
            cluster_params
        )?;

        self.clusters.insert(&cluster_id, &cluster);
        Ok(cluster_id)
    }

    pub fn get(&self, cluster_id: ClusterId) -> Result<Cluster> {
        self.clusters.get(cluster_id).ok_or(ClusterDoesNotExist)
    }

    pub fn update(&mut self, cluster_id: ClusterId, cluster: &Cluster) -> Result<()> {
        if !self.clusters.contains(&cluster_id) {
            Err(ClusterDoesNotExist)
        } else {
            self.clusters.insert(cluster_id, cluster);
            Ok(())
        }
    }

    pub fn remove(&mut self, cluster_id: ClusterId) {
        self.clusters.remove(cluster_id);
    }

}
