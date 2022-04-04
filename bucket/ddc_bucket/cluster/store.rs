//! The store where to create and access Clusters by ID.

use ink_prelude::vec::Vec;
use ink_storage::{
    collections::Vec as InkVec,
    traits,
};

use crate::ddc_bucket::{AccountId, Balance, Error::*, NodeId, Result};
use crate::ddc_bucket::cash::Cash;
use crate::ddc_bucket::contract_fee::SIZE_INDEX;
use crate::ddc_bucket::node::entity::Node;

use super::entity::{Cluster, ClusterId};

#[derive(traits::SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout, Debug))]
pub struct ClusterStore(pub InkVec<Cluster>);

impl ClusterStore {
    pub fn create(
        &mut self,
        manager_id: AccountId,
        partition_count: u32,
        nodes: &[(NodeId, &Node)],
    ) -> (ClusterId, usize) {
        let cluster_id = self.0.len();
        let (vnodes, total_rent) = Self::new_vnodes(partition_count as usize, nodes);
        let cluster = Cluster {
            cluster_id,
            manager_id,
            vnodes,
            resource_per_vnode: 0,
            resource_used: 0,
            revenues: Cash(0),
            total_rent,
        };
        let record_size = cluster.new_size();
        self.0.push(cluster);
        (cluster_id, record_size)
    }

    fn new_vnodes(partition_count: usize, nodes: &[(NodeId, &Node)]) -> (Vec<NodeId>, Balance) {
        let node_count = nodes.len();
        let mut vnode_ids = Vec::with_capacity(partition_count);
        let mut total_rent = 0;
        for i in 0..partition_count {
            let (node_id, node) = &nodes[i % node_count];
            vnode_ids.push(*node_id);
            total_rent += node.rent_per_month;
        }
        // TODO: consider using the max rent instead of average rent.
        (vnode_ids, total_rent)
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
