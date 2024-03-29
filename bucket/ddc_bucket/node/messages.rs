//! The public interface to manage Nodes.

use crate::ddc_bucket::node::entity::{NodeInfo, NodeKey, NodeParams, Resource};
use crate::ddc_bucket::{
    AccountId, Balance, DdcBucket, NodeCreated, NodeParamsSet, NodeRemoved, Result,
};
use ink_lang::codegen::{EmitEvent, StaticEnv};
use ink_prelude::vec::Vec;

impl DdcBucket {
    pub fn message_node_create(
        &mut self,
        node_key: NodeKey,
        node_params: NodeParams,
        capacity: Resource,
        rent_v_node_per_month: Balance,
    ) -> Result<NodeKey> {
        let caller = Self::env().caller();
        self.nodes.create(
            node_key,
            caller,
            node_params.clone(),
            capacity,
            rent_v_node_per_month,
        )?;

        Self::env().emit_event(NodeCreated {
            node_key,
            provider_id: caller,
            rent_v_node_per_month,
            node_params,
        });

        Ok(node_key)
    }

    pub fn message_node_remove(&mut self, node_key: NodeKey) -> Result<()> {
        let caller = Self::env().caller();
        let node = self.nodes.get(node_key)?;
        node.only_provider(caller)?;
        node.only_without_cluster()?;
        self.nodes.remove(node_key);

        Self::env().emit_event(NodeRemoved { node_key });

        Ok(())
    }

    pub fn message_node_set_params(
        &mut self,
        node_key: NodeKey,
        node_params: NodeParams,
    ) -> Result<()> {
        let caller = Self::env().caller();
        let mut node = self.nodes.get(node_key)?;
        node.only_provider(caller)?;
        node.set_params(node_params.clone())?;
        self.nodes.update(node_key, &node)?;

        Self::env().emit_event(NodeParamsSet {
            node_key,
            node_params,
        });

        Ok(())
    }

    pub fn message_node_get(&self, node_key: NodeKey) -> Result<NodeInfo> {
        let node = self.nodes.get(node_key)?;
        let v_nodes = self.topology.get_v_nodes_by_node(node_key);

        Ok(NodeInfo {
            node_key,
            node,
            v_nodes,
        })
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

            let v_nodes = self.topology.get_v_nodes_by_node(node_key);

            // Include the complete status of matched items.
            let node_info = NodeInfo {
                node_key,
                node,
                v_nodes,
            };

            nodes.push(node_info);
        }

        (nodes, self.nodes.keys.len().try_into().unwrap())
    }
}
