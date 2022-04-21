//! The public interface to manage Nodes.

use ink_lang::{EmitEvent, StaticEnv};
use ink_prelude::vec::Vec;

use crate::ddc_bucket::{AccountId, Balance, DdcBucket, NodeCreated, Result};
use crate::ddc_bucket::node::entity::{Node, NodeStatus, Resource};
use crate::ddc_bucket::perm::entity::Permission;
use crate::ddc_bucket::perm::store::PermStore;

use super::entity::{NodeId, NodeParams};

impl DdcBucket {
    pub fn message_node_trust_manager(&mut self, manager: AccountId, is_trusted: bool) -> Result<()> {
        let trust_giver = Self::env().caller();
        let perm = Permission::ManagerTrustedBy(trust_giver);

        if is_trusted {
            self.perms.grant_permission(manager, perm);
            Self::capture_fee_and_refund(PermStore::RECORD_SIZE)?;
        } else {
            self.perms.revoke_permission(manager, perm);
        }

        Ok(())
    }

    pub fn message_node_create(&mut self,
                               rent_per_month: Balance,
                               node_params: NodeParams,
                               capacity: Resource,
    ) -> Result<NodeId> {
        let provider_id = Self::env().caller();

        let node_id = self.nodes.create(provider_id, rent_per_month, capacity);
        let (params_id, recorde_size1) = self.node_params.create(node_params.clone())?;
        assert_eq!(node_id, params_id);

        Self::capture_fee_and_refund(Node::RECORD_SIZE + recorde_size1)?;
        Self::env().emit_event(NodeCreated { node_id, provider_id, rent_per_month, node_params });
        Ok(node_id)
    }

    pub fn message_node_change_params(&mut self, node_id: NodeId, params: NodeParams) -> Result<()>{
        let caller = Self::env().caller();
        let node = self.nodes.get(node_id)?;
        node.only_owner(caller)?;

        Self::impl_change_params(&mut self.node_params, node_id, params)
    }

    pub fn message_node_get(&self, node_id: NodeId) -> Result<NodeStatus> {
        let node = self.nodes.get(node_id)?.clone();
        let params = self.node_params.get(node_id)?.clone();
        Ok(NodeStatus { node_id, node, params })
    }

    pub fn message_node_list(&self, offset: u32, limit: u32, filter_provider_id: Option<AccountId>) -> (Vec<NodeStatus>, u32) {
        let mut nodes = Vec::with_capacity(limit as usize);
        for node_id in offset..offset + limit {
            let node = match self.nodes.0.get(node_id) {
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
            let status = NodeStatus {
                node_id,
                node: node.clone(),
                params: self.node_params.get(node_id).unwrap().clone(),
            };
            nodes.push(status);
        }
        (nodes, self.nodes.0.len())
    }
}
