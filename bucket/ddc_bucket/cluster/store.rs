//! The store where to create and access Clusters by ID.
use ink_prelude::vec::Vec;
use crate::ddc_bucket::node::entity::NodeId;
use crate::ddc_bucket::{AccountId, Error::*, Result};

use super::entity::{Cluster, ClusterId};

#[ink::storage_item]
#[derive(Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ClusterStore(pub Vec<Cluster>);

impl ClusterStore {
    pub fn create(
        &mut self,
        manager_id: AccountId,
        v_nodes: &Vec<Vec<u64>>,
        node_ids: &Vec<NodeId>,
    ) -> Result<ClusterId> {
        let cluster = Cluster::new(manager_id, v_nodes, node_ids);

        let cluster_id: ClusterId  = self.0.len().try_into().unwrap();
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
