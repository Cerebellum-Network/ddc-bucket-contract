//! The public interface to manage Nodes.

use ink_lang::{EmitEvent, StaticEnv};
use ink_prelude::vec::Vec;

use crate::ddc_bucket::{AccountId, Balance, DdcBucket, NodeCreated, Result};
use crate::ddc_bucket::node::entity::{Node, NodeStatus, Resource};
use crate::ddc_bucket::perm::entity::Permission;

use super::entity::{NodeId, CdnNode};

impl DdcBucket {
    pub fn message_cdn_node_trust_manager(&mut self, manager: AccountId, is_trusted: bool) -> Result<()> {
        let trust_giver = Self::env().caller();
        let permission = Permission::ManagerTrustedBy(trust_giver);
        self.impl_grant_permission(manager, permission, is_trusted)
    }

    pub fn message_cdn_node_create(&mut self,
                               undistributed_payment: Balance,
    ) -> Result<NodeId> {
        let provider_id = Self::env().caller();

        let node_id = self.cdn_nodes.create(provider_id, undistributed_payment);

        Self::capture_fee_and_refund(Node::RECORD_SIZE)?;
        Ok(node_id)
    }

    pub fn message_cdn_node_get(&self, node_id: NodeId) -> Result<CdnNode> {
        let node = self.cdn_nodes.get(node_id)?.clone();
        Ok(node)
    }

    pub fn message_cdn_node_list(&self, offset: u32, limit: u32, filter_provider_id: Option<AccountId>) -> Vec<CdnNode> {
        let mut cdn_nodes = Vec::with_capacity(limit as usize);
        for node_id in offset..offset + limit {
            let node = match self.cdn_nodes.0.get(node_id) {
                None => break, // No more items, stop.
                Some(node) => node,
            };
            // Apply the filter if given.
            if let Some(provider_id) = filter_provider_id {
                if provider_id != node.provider_id {
                    continue; // Skip non-matches.
                }
            }
            
            cdn_nodes.push(node.clone());
        }
        cdn_nodes
    }
}
