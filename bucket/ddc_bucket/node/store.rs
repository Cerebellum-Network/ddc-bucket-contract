//! The store where to create and access Nodes.

use ink_storage::{collections::Vec as InkVec, traits};

use crate::ddc_bucket::node::entity::Resource;
use crate::ddc_bucket::{AccountId, Balance, Error::*, Result};

use super::entity::{Node, NodeId, NodeTier};

#[derive(traits::SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout, Debug))]
pub struct NodeStore(pub InkVec<Node>);

impl NodeStore {
    pub fn create(
        &mut self,
        provider_id: AccountId,
        rent_per_month: Balance,
        capacity: Resource,
        tier: NodeTier,
    ) -> NodeId {
        let node_id = self.0.len();
        let node = Node {
            provider_id,
            rent_per_month,
            free_resource: capacity,
            tier,
        };

        self.0.push(node);
        node_id
    }

    pub fn get(&self, node_id: NodeId) -> Result<&Node> {
        self.0.get(node_id).ok_or(NodeDoesNotExist)
    }

    pub fn get_mut(&mut self, node_id: NodeId) -> Result<&mut Node> {
        self.0.get_mut(node_id).ok_or(NodeDoesNotExist)
    }
}
