//! The store where to create and access Nodes.

use ink_storage::{
    collections::Vec as InkVec,
    traits,
};

use crate::ddc_bucket::{AccountId, Balance, Error::*, Result};
use crate::ddc_bucket::node::entity::Resource;

use super::entity::{Node, NodeId};

#[derive(traits::SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout, Debug))]
pub struct NodeStore(pub InkVec<Node>);

impl NodeStore {
    pub fn create(&mut self,
                  provider_id: AccountId,
                  rent_per_month: Balance,
                  capacity: Resource,
    ) -> (NodeId, usize) {
        let node_id = self.0.len();
        let node = Node { node_id, provider_id, rent_per_month, free_resource: capacity };

        let record_size = node.new_size();
        self.0.push(node);
        (node_id, record_size)
    }

    pub fn get(&self, node_id: NodeId) -> Result<&Node> {
        self.0.get(node_id).ok_or(NodeDoesNotExist)
    }

    pub fn get_mut(&mut self, node_id: NodeId) -> Result<&mut Node> {
        self.0.get_mut(node_id).ok_or(NodeDoesNotExist)
    }
}
