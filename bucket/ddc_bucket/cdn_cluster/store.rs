//! The store where to create and access Clusters by ID.

use ink_storage::{
  traits
};
use ink_prelude::vec::Vec;

use crate::ddc_bucket::{AccountId, Error::*, NodeId, Result};

use super::entity::{CdnCluster, ClusterId};

pub const MAX_VNODES: u32 = 300;

#[derive(traits::SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout, Debug))]
pub struct CdnClusterStore(pub Vec<CdnCluster>);

impl CdnClusterStore {
  pub fn create(
      &mut self,
      manager_id: AccountId,
      cdn_nodes: Vec<NodeId>,
  ) -> Result<ClusterId> {
      let cluster = CdnCluster::new(manager_id, cdn_nodes);

      let cluster_id: ClusterId = self.0.len().try_into().unwrap();
      self.0.push(cluster);

      Ok(cluster_id)
  }

  pub fn get(&self, cluster_id: ClusterId) -> Result<&CdnCluster> {
      self.0.get(cluster_id as usize).ok_or(ClusterDoesNotExist)
  }

  pub fn get_mut(&mut self, cluster_id: ClusterId) -> Result<&mut CdnCluster> {
      self.0.get_mut(cluster_id as usize).ok_or(ClusterDoesNotExist)
  }
}
