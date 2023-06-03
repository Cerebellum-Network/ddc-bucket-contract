//! The store where to create and access Nodes.

use ink_storage::traits::{SpreadAllocate, SpreadLayout, StorageLayout};
use ink_prelude::vec::Vec as InkVec;
use ink_storage::Mapping;

use crate::ddc_bucket::{AccountId, Balance, Error::*, Result};

use super::entity::{CdnNode, NodeId};

pub type NodeKey = AccountId;

#[derive(SpreadAllocate, SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(StorageLayout, Debug))]
pub struct CdnNodeStore {
    pub pub_key_to_node: Mapping<AccountId, NodeId>,
    pub cdn_nodes: InkVec<CdnNode>
}

impl CdnNodeStore {
  pub fn create(
    &mut self,
    provider_id: AccountId,
    undistributed_payment: Balance,
    pubkey: AccountId,
  ) -> Result<NodeId> {
      let node_id: NodeId = self.cdn_nodes.len().try_into().unwrap();
      let node = CdnNode { provider_id, undistributed_payment, node_pub_key: pubkey };

      let exists = self.pub_key_to_node.contains(&pubkey);
      if exists {
          return Err(NodeAlreadyExists);
      }

      self.cdn_nodes.push(node);
      self.pub_key_to_node.insert(&pubkey, &node_id);
      Ok(node_id)
  }

  pub fn get_by_pub_key(&self, pubkey: AccountId) -> Result<NodeId> {
    self.pub_key_to_node.get(&pubkey).ok_or(NodeDoesNotExist)
}

  pub fn get(&self, node_id: NodeId) -> Result<&CdnNode> {
      self.cdn_nodes.get(node_id as usize).ok_or(NodeDoesNotExist)
  }

  pub fn get_mut(&mut self, node_id: NodeId) -> Result<&mut CdnNode> {
      self.cdn_nodes.get_mut(node_id as usize).ok_or(NodeDoesNotExist)
  }

  pub fn remove_node(&mut self, node_id: NodeId) -> Result<()> {
    let total_nodes = self.cdn_nodes.len();
    let last_node = self.cdn_nodes.get(total_nodes - 1).ok_or(NodeDoesNotExist).unwrap();
    self.pub_key_to_node.insert(&last_node.node_pub_key, &node_id);
    self.cdn_nodes.swap_remove(node_id.try_into().unwrap());
    Ok(())
}
}
