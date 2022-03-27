//! The store where to create and access Nodes.

use ink_prelude::vec::Vec;
use ink_storage::{
    collections::Vec as InkVec,
    traits,
};

use crate::ddc_bucket::{AccountId, Balance, Error::*, Result};
use crate::ddc_bucket::node::entity::Resource;

use super::entity::{Node, NodeId, NodeParams};

#[derive(traits::SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout, Debug))]
pub struct NodeStore(pub InkVec<Node>);

impl NodeStore {
    pub fn create(&mut self,
                  provider_id: AccountId,
                  rent_per_month: Balance,
                  node_params: NodeParams,
                  capacity: Resource,
    ) -> (NodeId, usize) {
        let node_id = self.0.len();
        let node = Node { node_id, provider_id, rent_per_month, node_params, free_resource: capacity };

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

    pub fn list(&self, offset: u32, limit: u32, filter_provider_id: Option<AccountId>) -> (Vec<Node>, u32) {
        let mut nodes = Vec::with_capacity(limit as usize);
        for node_id in offset..offset + limit {
            let node = match self.0.get(node_id) {
                None => break, // No more items, stop.
                Some(node) => node,
            };
            // Apply the filter if given.
            if let Some(provider_id) = filter_provider_id {
                if provider_id != node.provider_id {
                    continue; // Skip non-matches.
                }
            }
            nodes.push(node.clone());
        }
        (nodes, self.0.len())
    }
}
