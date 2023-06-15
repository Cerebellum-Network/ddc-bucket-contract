//! The public interface to manage Nodes.

use crate::ddc_bucket::node::entity::{NodeInfo, Resource, NodeKey, NodeParams, NodeStatus};
use crate::ddc_bucket::perm::entity::Permission;
use crate::ddc_bucket::{AccountId, Balance, DdcBucket, NodeCreated, Result};
use ink_lang::codegen::{EmitEvent, StaticEnv};
use ink_prelude::vec::Vec;


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
        node_key: NodeKey,
        rent_per_month: Balance,
        node_params: NodeParams,
        capacity: Resource,
        node_tag: NodeStatus,
    ) -> Result<NodeKey> {
        let provider_id = Self::env().caller();

        let node_key = self
            .nodes
            .create(node_key, provider_id, rent_per_month, node_params.clone(), capacity, node_tag)
            .unwrap();

        Self::env().emit_event(NodeCreated {
            node_key,
            provider_id,
            rent_per_month,
            node_params,
        });

        Ok(node_key)
    }

    pub fn message_node_change_tag(&mut self, node_key: NodeKey, new_tag: NodeStatus) -> Result<()> {
        let caller = Self::env().caller();
        let mut node = self.nodes.get(node_key)?;
        node.only_owner(caller)?;
        node.change_tag(new_tag);
        self.nodes.update(node_key, &node)?;
        Ok(())
    }

    pub fn message_node_get(&self, node_key: NodeKey) -> Result<NodeInfo> {
        let node = self.nodes.get(node_key)?;
        Ok(NodeInfo {
            node_key,
            node,
        })
    }

    pub fn message_node_change_params(
        &mut self,
        node_key: NodeKey,
        node_params: NodeParams,
    ) -> Result<()> {
        
        let caller = Self::env().caller();
        let mut node = self.nodes.get(node_key)?;
        node.only_owner(caller)?;
        node.node_params = node_params;
        self.nodes.update(node_key, &node)?;
        Ok(())
    }

    pub fn message_node_list(
        &self,
        offset: u32,
        limit: u32,
        filter_provider_id: Option<AccountId>,
    ) -> (Vec<NodeInfo>, u32) {
        let mut nodes = Vec::with_capacity(limit as usize);
        for idx in offset..offset + limit {
            let node_key = match self.nodes.keys.get(idx as usize) {
                None => break, // No more items, stop.
                Some(node_key) => node_key.clone(),
            };

            let node = self.nodes.nodes.get(node_key).unwrap();

            // Apply the filter if given.
            if let Some(provider_id) = filter_provider_id {
                if provider_id != node.provider_id {
                    continue; // Skip non-matches.
                }
            }
            
            // Include the complete status of matched items.
            let status = NodeInfo {
                node_key,
                node,
            };

            nodes.push(status);
        }

        (nodes, self.nodes.keys.len().try_into().unwrap())
    }

    pub fn message_remove_node(&mut self, node_key: NodeKey) -> Result<()> {
        let caller = Self::env().caller();
        let node = self.nodes.get(node_key)?;
        node.only_owner(caller)?;
        self.nodes.remove(node_key)
    }

}