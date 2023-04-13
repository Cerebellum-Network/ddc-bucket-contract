//! The store where to create and access Nodes.
use ink_prelude::vec::Vec;
use crate::ddc_bucket::{AccountId, Balance, Error::*, Result};
use super::entity::{CdnNode, NodeId};

pub const CDN_NODE_STORE_KEY: u32 = openbrush::storage_unique_key!(CdnNodeStore);
#[openbrush::upgradeable_storage(CDN_NODE_STORE_KEY)]
#[derive(Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct CdnNodeStore {
    pub nodes: Vec<CdnNode>,
    _reserved: Option<()>
}

impl CdnNodeStore {
  pub fn create(&mut self,
                provider_id: AccountId,
                undistributed_payment: Balance,
  ) -> NodeId {
      let node_id: NodeId = self.nodes.len().try_into().unwrap();
      let node = CdnNode { provider_id, undistributed_payment };

      self.nodes.push(node);
      node_id
  }

  pub fn get(&self, node_id: NodeId) -> Result<&CdnNode> {
      self.nodes.get(node_id as usize).ok_or(NodeDoesNotExist)
  }

  pub fn get_mut(&mut self, node_id: NodeId) -> Result<&mut CdnNode> {
      self.nodes.get_mut(node_id as usize).ok_or(NodeDoesNotExist)
  }
}
