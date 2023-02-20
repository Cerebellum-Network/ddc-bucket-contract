//! The store where to create and access Clusters by ID.

// use ink_storage::{collections::Vec as InkVec, traits};
// use ink_storage::{collections::Vec as InkVec, traits};
use ink_prelude::vec::Vec;
use ink_storage::traits::{SpreadLayout, StorageLayout};

use crate::ddc_bucket::{AccountId, Error::*, Result};

use super::entity::{Cluster, ClusterId};

#[derive(SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(StorageLayout, Debug))]
pub struct ClusterStore(pub Vec<Cluster>);

impl ClusterStore {
    pub fn create(&mut self, manager_id: AccountId, v_nodes: &Vec<Vec<u64>>) -> Result<ClusterId> {
        let cluster = Cluster::new(manager_id, v_nodes);

        let cluster_id = self.0.len();
        self.0.push(cluster);

        Ok(cluster_id.try_into().unwrap())
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
