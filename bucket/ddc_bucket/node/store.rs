//! The store where to create and access Nodes.
use ink_prelude::vec::Vec;
use crate::ddc_bucket::node::entity::Resource;
use crate::ddc_bucket::{AccountId, Balance, Error::*, Result};

use super::entity::{Node, NodeId, NodeTag};

pub const NODE_STORE_KEY: u32 = openbrush::storage_unique_key!(NodeStore);
#[openbrush::upgradeable_storage(NODE_STORE_KEY)]
#[derive(Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct NodeStore {
    pub nodes: Vec<Node>,
    _reserved: Option<()>
}

impl NodeStore {
    pub fn create(
        &mut self,
        provider_id: AccountId,
        rent_per_month: Balance,
        capacity: Resource,
        node_tag: NodeTag,
    ) -> NodeId {
        let node_id: NodeId = self.nodes.len().try_into().unwrap();
        let node = Node {
            provider_id,
            rent_per_month,
            free_resource: capacity,
            node_tag,
        };

        self.nodes.push(node);
        node_id
    }

    pub fn get(&self, node_id: NodeId) -> Result<&Node> {
        self.nodes.get(node_id as usize).ok_or(NodeDoesNotExist)
    }

    pub fn get_mut(&mut self, node_id: NodeId) -> Result<&mut Node> {
        self.nodes.get_mut(node_id as usize).ok_or(NodeDoesNotExist)
    }
}
