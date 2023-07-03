//! The store where to create and access Nodes.

use ink_storage::traits::{SpreadAllocate, SpreadLayout};
use ink_prelude::vec::Vec;
use ink_storage::Mapping;
use crate::ddc_bucket::{AccountId, Balance, Error::*, Result};
use super::entity::{Node, NodeKey, NodeParams, Resource};

// https://use.ink/datastructures/storage-layout#packed-vs-non-packed-layout
// There is a buffer with only limited capacity (around 16KB in the default configuration) available.
pub const MAX_NODES_LEN_IN_VEC: usize = 400;

#[derive(SpreadAllocate, SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(ink_storage::traits::StorageLayout, Debug))]
pub struct NodeStore {
    pub nodes: Mapping<NodeKey, Node>,
    // todo: remove this vector as it can store an arbitrary number of elements and easily exceed 16KB limit
    pub keys: Vec<NodeKey> 
}

impl NodeStore {
    pub fn create(
        &mut self,
        node_key: AccountId,
        provider_id: AccountId,
        node_params: NodeParams,
        capacity: Resource,
        rent_v_node_per_month: Balance,
    ) -> Result<NodeKey> {

        if self.nodes.contains(&node_key) {
           return Err(NodeAlreadyExists);
        } 

        if self.keys.len() + 1 > MAX_NODES_LEN_IN_VEC {
            return Err(NodesSizeExceedsLimit);
        }

        let node = Node::new(
            provider_id,
            node_params,
            capacity,
            rent_v_node_per_month
        )?;

        self.nodes.insert(node_key, &node);
        self.keys.push(node_key);
        Ok(node_key)
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

    pub fn remove(&mut self, node_key: NodeKey) {
        self.nodes.remove(node_key);
        if let Some(pos) = self.keys.iter().position(|x| *x == node_key) {
            self.keys.remove(pos);
        };
    }
    
}
