//! The store where to create and access Clusters by ID.

use ink_prelude::vec::Vec;
use ink_storage::traits::{SpreadAllocate, SpreadLayout, StorageLayout};

use crate::ddc_bucket::node::entity::{NodeKey};
use crate::ddc_bucket::{AccountId, Error::*, Result};

use super::entity::{Cluster, ClusterId};

#[derive(SpreadAllocate, SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(StorageLayout, Debug))]
pub struct ClusterStore(pub Vec<Cluster>);

impl ClusterStore {
    pub fn create(
        &mut self,
        manager_id: AccountId,
        v_nodes: &Vec<Vec<u64>>,
        node_keys: &Vec<NodeKey>,
    ) -> Result<ClusterId> {
        let cluster = Cluster::new(manager_id, v_nodes, node_keys);

        let cluster_id: ClusterId = self.0.len().try_into().unwrap();
        self.0.push(cluster);

        Ok(cluster_id)
    }

    pub fn get(&self, cluster_id: ClusterId) -> Result<&Cluster> {
        self.0.get(cluster_id as usize).ok_or(ClusterDoesNotExist)
    }

    pub fn get_mut(&mut self, cluster_id: ClusterId) -> Result<&mut Cluster> {
        self.0
            .get_mut(cluster_id as usize)
            .ok_or(ClusterDoesNotExist)
    }
}
