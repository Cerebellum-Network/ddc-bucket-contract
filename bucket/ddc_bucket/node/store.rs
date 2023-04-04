//! The store where to create and access Nodes.

use ink_storage::{collections::Vec as InkVec, traits};

use crate::ddc_bucket::node::entity::Resource;
use crate::ddc_bucket::{AccountId, Balance, Error::*, Result};
use ink_storage::collections::HashMap;

use super::entity::{Node, NodeId, NodeTag};

#[derive(traits::SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout, Debug))]
pub struct NodeStore {
    pub store: HashMap<AccountId, NodeId>,
    pub keys: InkVec<Node>,
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
        let node_id = self.keys.len();

        let node = Node {
            provider_id,
            rent_per_month,
            free_resource: capacity,
            node_tag,
        };

        let exists = self.store.contains_key(&pubkey);
        if exists {
            return Err(NodeAlreadyExists);
        }

        self.keys.push(node);
        self.store.insert(pubkey, node_id);

        Ok(node_id)
    }

    pub fn get_by_pub_key(&self, pubkey: AccountId) -> Result<&NodeId> {
        self.store.get(&pubkey).ok_or(NodeDoesNotExist)
    }

    pub fn get(&self, node_id: NodeId) -> Result<&Node> {
        self.keys.get(node_id).ok_or(NodeDoesNotExist)
    }

    pub fn get_mut(&mut self, node_id: NodeId) -> Result<&mut Node> {
        self.keys.get_mut(node_id).ok_or(NodeDoesNotExist)
    }
}
