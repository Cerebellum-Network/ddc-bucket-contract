//! The store where to create and access Clusters by ID.

use super::entity::{Cluster, ClusterId, ClusterParams};
use crate::ddc_bucket::{AccountId, Error::*, Resource, Result};
use ink_prelude::vec::Vec;
use ink_storage::traits::{SpreadAllocate, SpreadLayout};
use ink_storage::Mapping;

#[derive(SpreadAllocate, SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(ink_storage::traits::StorageLayout, Debug))]
pub struct ClusterStore {
    pub next_cluster_id: ClusterId,
    pub clusters: Mapping<ClusterId, Cluster>,
    pub clusters_ids: Vec<ClusterId>,
}

// https://use.ink/datastructures/storage-layout#packed-vs-non-packed-layout
// There is a buffer with only limited capacity (around 16KB in the default configuration) available.
pub const MAX_CLUSTERS_LEN_IN_VEC: usize = 3900;

impl ClusterStore {
    pub fn create(
        &mut self,
        manager_id: AccountId,
        cluster_params: ClusterParams,
        resource_per_v_node: Resource,
    ) -> Result<ClusterId> {
        if self.clusters_ids.len() + 1 > MAX_CLUSTERS_LEN_IN_VEC {
            return Err(NodesSizeExceedsLimit);
        }

        let cluster_id = self.next_cluster_id;
        self.next_cluster_id = self.next_cluster_id + 1;

        let cluster = Cluster::new(manager_id, cluster_params, resource_per_v_node)?;

        self.clusters.insert(&cluster_id, &cluster);
        self.clusters_ids.push(cluster_id);
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
        if let Some(pos) = self.clusters_ids.iter().position(|x| *x == cluster_id) {
            self.clusters_ids.remove(pos);
        };
    }
}
