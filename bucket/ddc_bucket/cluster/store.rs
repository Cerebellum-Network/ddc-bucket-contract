//! The store where to create and access Clusters by ID.

use ink_storage::{collections::Vec as InkVec, traits};

use crate::ddc_bucket::node::entity::Node;
use crate::ddc_bucket::{AccountId, Balance, Error::*, NodeId, Result};

use super::entity::{Cluster, ClusterId};

#[derive(traits::SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout, Debug))]
pub struct ClusterStore(pub InkVec<Cluster>);

impl ClusterStore {
    pub fn create(&mut self, manager_id: AccountId) -> Result<ClusterId> {
        let cluster = Cluster::new(manager_id);

        let cluster_id = self.0.len();
        self.0.push(cluster);

        Ok(cluster_id)
    }

    pub fn add_node(&mut self, cluster_id: ClusterId, node_id: NodeId) -> Result<()> {
        let cluster = self.get_mut(cluster_id)?;
        cluster.vnodes.push(node_id);
        Ok(())
    }

    pub fn get(&self, cluster_id: ClusterId) -> Result<&Cluster> {
        self.0.get(cluster_id).ok_or(ClusterDoesNotExist)
    }

    pub fn get_mut(&mut self, cluster_id: ClusterId) -> Result<&mut Cluster> {
        self.0.get_mut(cluster_id).ok_or(ClusterDoesNotExist)
    }
}
