//! The store where to create and access Clusters by ID.

use ink_storage::{
    collections::Vec as InkVec,
    traits,
};

use crate::ddc_bucket::{AccountId, Error::*, NodeId, Result};
use crate::ddc_bucket::contract_fee::SIZE_INDEX;
use crate::ddc_bucket::node::entity::Node;

use super::entity::{Cluster, ClusterId};

pub const MAX_VNODES: u32 = 1000;

#[derive(traits::SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout, Debug))]
pub struct ClusterStore(pub InkVec<Cluster>);

impl ClusterStore {
    pub fn create(
        &mut self,
        manager_id: AccountId,
        vnode_count: u32,
        nodes: &[(NodeId, &Node)],
    ) -> Result<(ClusterId, usize)> {
        if vnode_count > MAX_VNODES {
            return Err(TooManyVNodes);
        }
        let cluster = Cluster::new(manager_id, vnode_count, nodes);

        let record_size = cluster.new_size();
        let cluster_id = self.0.len();
        self.0.push(cluster);

        Ok((cluster_id, record_size))
    }

    pub fn add_node(&mut self, cluster_id: ClusterId, node_id: NodeId) -> Result<usize> {
        let cluster = self.get_mut(cluster_id)?;
        cluster.vnodes.push(node_id);
        Ok(SIZE_INDEX)
    }

    pub fn get(&self, cluster_id: ClusterId) -> Result<&Cluster> {
        self.0.get(cluster_id).ok_or(ClusterDoesNotExist)
    }

    pub fn get_mut(&mut self, cluster_id: ClusterId) -> Result<&mut Cluster> {
        self.0.get_mut(cluster_id).ok_or(ClusterDoesNotExist)
    }
}
