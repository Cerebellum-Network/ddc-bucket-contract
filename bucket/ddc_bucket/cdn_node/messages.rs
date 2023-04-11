//! The public interface to manage Nodes.

use ink::codegen::StaticEnv;
use ink_prelude::vec::Vec;

use crate::ddc_bucket::{AccountId, DdcBucket, Result};
use crate::ddc_bucket::params::store::Params;
use crate::ddc_bucket::perm::entity::Permission;

use super::entity::{NodeId, CdnNodeStatus};

impl DdcBucket {
    pub fn message_cdn_node_trust_manager(&mut self, manager: AccountId, is_trusted: bool) -> Result<()> {
        let trust_giver = Self::env().caller();
        let permission = Permission::ManagerTrustedBy(trust_giver);
        self.impl_grant_permission(manager, permission, is_trusted)
    }

    pub fn message_cdn_node_create(&mut self, node_params: Params) -> Result<NodeId> {
        let provider_id = Self::env().caller();

        let node_id = self.cdn_nodes.create(provider_id, 0);
        let params_id = self.cdn_node_params.create(node_params.clone())?;
        assert_eq!(node_id, params_id);

        Ok(node_id)
    }

    pub fn message_cdn_node_change_params(&mut self, node_id: NodeId, params: Params) -> Result<()> {
        let caller = Self::env().caller();
        let node = self.cdn_nodes.get(node_id)?;
        node.only_owner(caller)?;

        Self::impl_change_params(&mut self.cdn_node_params, node_id, params)
    }

    pub fn message_cdn_node_get(&self, node_id: NodeId) -> Result<CdnNodeStatus> {
        let node = self.cdn_nodes.get(node_id)?.clone();
        let params = self.cdn_node_params.get(node_id)?.clone();
        Ok(CdnNodeStatus { node_id, node, params })
    }

    pub fn message_cdn_node_list(&self, offset: u32, limit: u32, filter_provider_id: Option<AccountId>) -> (Vec<CdnNodeStatus>, u32) {
        let mut cdn_nodes = Vec::with_capacity(limit as usize);
        for node_id in offset..offset + limit {
            let node = match self.cdn_nodes.0.get(node_id as usize) {
                None => break, // No more items, stop.
                Some(node) => node,
            };
            // Apply the filter if given.
            if let Some(provider_id) = filter_provider_id {
                if provider_id != node.provider_id {
                    continue; // Skip non-matches.
                }
            }

            // Include the complete status of matched items.
            let status = CdnNodeStatus {
                node_id,
                node: node.clone(),
                params: self.cdn_node_params.get(node_id).unwrap().clone(),
            };
            
            cdn_nodes.push(status);
        }
        (cdn_nodes, self.cdn_nodes.0.len().try_into().unwrap())
    }
}
