//! The public interface to manage Nodes.

use crate::ddc_bucket::node::entity::{NodeStatus, Resource};
use crate::ddc_bucket::perm::entity::Permission;
use crate::ddc_bucket::{AccountId, Balance, DdcBucket, NodeCreated, Result};
use ink_lang::{EmitEvent, StaticEnv};
use ink_prelude::vec::Vec;

use super::entity::{NodeId, NodeParams, NodeTag};

impl DdcBucket {
    pub fn message_node_trust_manager(
        &mut self,
        manager: AccountId,
        is_trusted: bool,
    ) -> Result<()> {
        let trust_giver = Self::env().caller();
        let permission = Permission::ManagerTrustedBy(trust_giver);
        self.impl_grant_permission(manager, permission, is_trusted)
    }

    pub fn message_node_create(
        &mut self,
        rent_per_month: Balance,
        node_params: NodeParams,
        capacity: Resource,
        node_tag: NodeTag,
        pubkey: AccountId,
    ) -> Result<NodeId> {
        let provider_id = Self::env().caller();

        let node_id = self
            .nodes
            .create(provider_id, rent_per_month, capacity, node_tag, pubkey)
            .unwrap();

        let params_id = self.node_params.create(node_params.clone())?;
        assert_eq!(node_id, params_id);

        Self::env().emit_event(NodeCreated {
            node_id,
            provider_id,
            rent_per_month,
            node_params,
        });
        Ok(node_id)
    }

    pub fn message_node_change_tag(&mut self, node_id: NodeId, new_tag: NodeTag) -> Result<()> {
        let caller = Self::env().caller();
        let node = self.nodes.get_mut(node_id)?;

        node.only_owner(caller)?;
        node.change_tag(new_tag);
        Ok(())
    }

    pub fn message_node_get(&self, node_id: NodeId) -> Result<NodeStatus> {
        let node = self.nodes.get(node_id)?.clone();
        let params = self.node_params.get(node_id)?.clone();
        Ok(NodeStatus {
            node_id,
            node,
            params,
        })
    }

    pub fn message_node_change_params(
        &mut self,
        node_id: NodeId,
        params: NodeParams,
    ) -> Result<()> {
        let caller = Self::env().caller();
        let node = self.nodes.get(node_id)?;
        node.only_owner(caller)?;

        Self::impl_change_params(&mut self.node_params, node_id, params)
    }

    pub fn message_node_list(
        &self,
        offset: u32,
        limit: u32,
        filter_provider_id: Option<AccountId>,
    ) -> (Vec<NodeStatus>, u32) {
        let mut nodes = Vec::with_capacity(limit as usize);
        for node_id in offset..offset + limit {
            let node = match self.nodes.nodes.get(node_id) {
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
        (nodes, self.nodes.nodes.len())
    }

    pub fn message_node_get_by_pub_key(&self, pubkey: AccountId) -> Result<NodeStatus> {
        let node_id = self.nodes.get_by_pub_key(pubkey).unwrap();
        let node = self.nodes.get(*node_id)?.clone();
        let params = self.node_params.get(*node_id)?.clone();
        Ok(NodeStatus {
            node_id: *node_id,
            node,
            params,
        })
    }
}
