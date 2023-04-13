//! The store where to create and access Clusters by ID.

use ink_prelude::vec::Vec;
use crate::ddc_bucket::{AccountId, Error::*, NodeId, Result};
use super::entity::{CdnCluster, ClusterId};

pub const MAX_VNODES: u32 = 300;

pub const CDN_CLUSTER_STORE_KEY: u32 = openbrush::storage_unique_key!(CdnClusterStore);
#[openbrush::upgradeable_storage(CDN_CLUSTER_STORE_KEY)]
#[derive(Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct CdnClusterStore {
    pub clusters: Vec<CdnCluster>,
    _reserved: Option<()>
}

impl CdnClusterStore {
  pub fn create(
      &mut self,
      manager_id: AccountId,
      cdn_nodes: Vec<NodeId>,
  ) -> Result<ClusterId> {
      let cluster = CdnCluster::new(manager_id, cdn_nodes);

      let cluster_id: ClusterId = self.clusters.len().try_into().unwrap();
      self.clusters.push(cluster);

      Ok(cluster_id)
  }

  pub fn get(&self, cluster_id: ClusterId) -> Result<&CdnCluster> {
      self.clusters.get(cluster_id as usize).ok_or(ClusterDoesNotExist)
  }

  pub fn get_mut(&mut self, cluster_id: ClusterId) -> Result<&mut CdnCluster> {
      self.clusters.get_mut(cluster_id as usize).ok_or(ClusterDoesNotExist)
  }
}
