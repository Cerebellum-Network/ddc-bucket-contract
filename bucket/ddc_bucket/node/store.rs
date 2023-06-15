//! The store where to create and access Nodes.

use ink_storage::traits::{SpreadAllocate, SpreadLayout, StorageLayout};
use ink_prelude::vec::Vec;
use ink_storage::Mapping;

// use crate::ddc_bucket::node::entity::Resource;
use crate::ddc_bucket::{AccountId, Balance, Error::*, Result};

use super::entity::{Node, NodeStatus, NodeKey, NodeParams, Resource};

#[derive(SpreadAllocate, SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(StorageLayout, Debug))]
pub struct NodeStore {
    pub nodes: Mapping<NodeKey, Node>,
    // This vector is temporal and must be replaced with an offchain indexer
    pub keys: Vec<NodeKey> 
}

impl NodeStore {
    pub fn create(
        &mut self,
        node_key: AccountId,
        provider_id: AccountId,
        rent_per_month: Balance,
        node_params: NodeParams,
        capacity: Resource,
        node_tag: NodeStatus,
    ) -> Result<NodeKey> {
        
        let node = Node {
            provider_id,
            rent_per_month,
            free_resource: capacity,
            node_tag,
            node_params
        };

        if self.nodes.contains(&node_key) {
            Err(NodeAlreadyExists)
        } else {
            self.nodes.insert(node_key, &node);
            self.keys.push(node_key);
            Ok(node_key)
        }
    }

    pub fn get(&self, node_key: NodeKey) -> Result<Node> {
        self.nodes.get(node_key).ok_or(NodeDoesNotExist)
    }

    pub fn update(&mut self, node_key: NodeKey, node: &Node) -> Result<()> {
        if !self.nodes.contains(&node_key) {
            Err(NodeDoesNotExist)
        } else {
            self.nodes.insert(node_key, node);
            Ok(())
        }
    }

    pub fn remove(&mut self, node_key: NodeKey) -> Result<()> {
        self.nodes.remove(node_key);
        if let Some(pos) = self.keys.iter().position(|x| *x == node_key) {
            self.keys.remove(pos);
        };
        Ok(())
    }
}
