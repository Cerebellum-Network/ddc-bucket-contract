//! The store where to create and access Nodes.
use ink_prelude::vec::Vec;
use crate::ddc_bucket::{AccountId, Balance, Error::*, Result};
use super::entity::{CdnNode, NodeId};

#[ink::storage_item]
#[derive(Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct CdnNodeStore(pub Vec<CdnNode>);

impl CdnNodeStore {
  pub fn create(&mut self,
                provider_id: AccountId,
                undistributed_payment: Balance,
  ) -> NodeId {
      let node_id: NodeId = self.0.len().try_into().unwrap();
      let node = CdnNode { provider_id, undistributed_payment };

      self.0.push(node);
      node_id
  }

  pub fn get(&self, node_id: NodeId) -> Result<&CdnNode> {
      self.0.get(node_id as usize).ok_or(NodeDoesNotExist)
  }

  pub fn get_mut(&mut self, node_id: NodeId) -> Result<&mut CdnNode> {
      self.0.get_mut(node_id as usize).ok_or(NodeDoesNotExist)
  }
}
