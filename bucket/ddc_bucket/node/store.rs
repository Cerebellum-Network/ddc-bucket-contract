//! The store where to create and access Nodes.

use ink_storage::traits::{SpreadAllocate, SpreadLayout, StorageLayout};
use ink_prelude::vec::Vec;
use ink_storage::Mapping;

use crate::ddc_bucket::node::entity::Resource;
use crate::ddc_bucket::{AccountId, Balance, Error::*, Result};

use super::entity::{Node, NodeId, NodeTag};

#[derive(SpreadAllocate, SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(StorageLayout, Debug))]
pub struct NodeStore {
    pub account_node: Mapping<AccountId, NodeId>,
    pub nodes: Vec<Node>,
}

impl NodeStore {
    pub fn create(
        &mut self,
        provider_id: AccountId,
        rent_per_month: Balance,
        capacity: Resource,
        node_tag: NodeTag,
        pubkey: AccountId,
    ) -> Result<NodeId> {
        let node_id: NodeId = self.nodes.len().try_into().unwrap();
        let node = Node {
            provider_id,
            rent_per_month,
            free_resource: capacity,
            node_tag,
            node_pub_key: pubkey,
        };

        let exists = self.account_node.contains(&pubkey);
        if exists {
            return Err(NodeAlreadyExists);
        }

        self.nodes.push(node);
        self.account_node.insert(&pubkey, &node_id);

        Ok(node_id)
    }

    pub fn get_by_pub_key(&self, pubkey: AccountId) -> Result<NodeId> {
        self.account_node.get(&pubkey).ok_or(NodeDoesNotExist)
    }

    pub fn get(&self, node_id: NodeId) -> Result<&Node> {
        self.nodes.get(node_id as usize).ok_or(NodeDoesNotExist)
    }

    pub fn get_mut(&mut self, node_id: NodeId) -> Result<&mut Node> {
        self.nodes.get_mut(node_id as usize).ok_or(NodeDoesNotExist)
    }

    pub fn remove_node(&mut self, node_id: NodeId) -> Result<()> {
        let total_nodes = self.nodes.len();
        let last_node = self.nodes.get(total_nodes - 1).ok_or(NodeDoesNotExist).unwrap();
        self.account_node.insert(&last_node.node_pub_key, &node_id);
        self.nodes.swap_remove(node_id.try_into().unwrap());
        Ok(())
    }
}
