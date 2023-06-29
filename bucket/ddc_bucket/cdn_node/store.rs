//! The store where to create and access Nodes.

use ink_storage::traits::{SpreadAllocate, SpreadLayout};
use ink_storage::Mapping;
use ink_prelude::vec::Vec;

use crate::ddc_bucket::{AccountId, Balance, Error::*, Result};
use super::entity::{CdnNode, CdnNodeKey, CdnNodeParams};


#[derive(SpreadAllocate, SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(ink_storage::traits::StorageLayout, Debug))]
pub struct CdnNodeStore {
  pub cdn_nodes: Mapping<CdnNodeKey, CdnNode>,
  // This vector is temporal and must be replaced with an offchain indexer
  pub keys: Vec<CdnNodeKey> 
}

impl CdnNodeStore {
    pub fn create(
      &mut self,
      cdn_node_key: CdnNodeKey,
      provider_id: AccountId,
      cdn_node_params: CdnNodeParams,
      undistributed_payment: Balance,
    ) -> Result<CdnNodeKey> {

        if self.cdn_nodes.contains(&cdn_node_key) {
            Err(CdnNodeAlreadyExists)
        } else {
            let cdn_node = CdnNode::new(
                provider_id,
                cdn_node_params,
                undistributed_payment
            )?;
            self.cdn_nodes.insert(&cdn_node_key, &cdn_node);
            self.keys.push(cdn_node_key);
            Ok(cdn_node_key)
        }
        
    }

  pub fn get(&self, cdn_node_key: CdnNodeKey) -> Result<CdnNode> {
      self.cdn_nodes.get(cdn_node_key).ok_or(CdnNodeDoesNotExist)
  }

  pub fn update(&mut self, cdn_node_key: CdnNodeKey, cdn_node: &CdnNode) -> Result<()> {
      if !self.cdn_nodes.contains(&cdn_node_key) {
          Err(CdnNodeDoesNotExist)
      } else {
          self.cdn_nodes.insert(cdn_node_key, cdn_node);
          Ok(())
      }
  }

  pub fn remove(&mut self, cdn_node_key: CdnNodeKey) {
      self.cdn_nodes.remove(cdn_node_key);
      if let Some(pos) = self.keys.iter().position(|x| *x == cdn_node_key) {
          self.keys.remove(pos);
      };
  }

}
