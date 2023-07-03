//! The public interface to manage Nodes.

use crate::ddc_bucket::{
    AccountId, Balance, CdnNodeCreated, CdnNodeParamsSet, CdnNodeRemoved, DdcBucket, Result,
};
use ink_lang::codegen::{EmitEvent, StaticEnv};
use ink_prelude::vec::Vec;

use super::entity::{CdnNodeInfo, CdnNodeKey, CdnNodeParams};

impl DdcBucket {
    pub fn message_cdn_node_create(
        &mut self,
        cdn_node_key: CdnNodeKey,
        cdn_node_params: CdnNodeParams,
    ) -> Result<CdnNodeKey> {
        let caller = Self::env().caller();
        let undistributed_payment: Balance = 0;
        self.cdn_nodes.create(
            cdn_node_key,
            caller,
            cdn_node_params.clone(),
            undistributed_payment,
        )?;

        Self::env().emit_event(CdnNodeCreated {
            cdn_node_key,
            provider_id: caller,
            cdn_node_params,
            undistributed_payment,
        });

        Ok(cdn_node_key)
    }

    pub fn message_remove_cdn_node(&mut self, cdn_node_key: CdnNodeKey) -> Result<()> {
        let caller = Self::env().caller();
        let cdn_node = self.cdn_nodes.get(cdn_node_key)?;
        cdn_node.only_provider(caller)?;
        cdn_node.only_without_cluster()?;
        self.cdn_nodes.remove(cdn_node_key);

        Self::env().emit_event(CdnNodeRemoved { cdn_node_key });

        Ok(())
    }

    pub fn message_cdn_node_set_params(
        &mut self,
        cdn_node_key: CdnNodeKey,
        cdn_node_params: CdnNodeParams,
    ) -> Result<()> {
        let caller = Self::env().caller();
        let mut cdn_node = self.cdn_nodes.get(cdn_node_key)?;
        cdn_node.only_provider(caller)?;
        cdn_node.set_params(cdn_node_params.clone())?;
        self.cdn_nodes.update(cdn_node_key, &cdn_node)?;

        Self::env().emit_event(CdnNodeParamsSet {
            cdn_node_key,
            cdn_node_params,
        });

        Ok(())
    }

    pub fn message_cdn_node_get(&self, cdn_node_key: CdnNodeKey) -> Result<CdnNodeInfo> {
        let cdn_node = self.cdn_nodes.get(cdn_node_key)?;
        Ok(CdnNodeInfo {
            cdn_node_key,
            cdn_node,
        })
    }

    pub fn message_cdn_node_list(
        &self,
        offset: u32,
        limit: u32,
        filter_provider_id: Option<AccountId>,
    ) -> (Vec<CdnNodeInfo>, u32) {
        let mut cdn_nodes = Vec::with_capacity(limit as usize);
        for idx in offset..offset + limit {
            let cdn_node_key = match self.cdn_nodes.keys.get(idx as usize) {
                None => break, // No more items, stop.
                Some(cdn_node_key) => cdn_node_key.clone(),
            };

            let cdn_node = self.cdn_nodes.cdn_nodes.get(cdn_node_key).unwrap();
            // Apply the filter if given.
            if let Some(provider_id) = filter_provider_id {
                if provider_id != cdn_node.provider_id {
                    continue; // Skip non-matches.
                }
            }

            // Include the complete status of matched items.
            let status = CdnNodeInfo {
                cdn_node_key,
                cdn_node,
            };

            cdn_nodes.push(status);
        }

        (cdn_nodes, self.cdn_nodes.keys.len().try_into().unwrap())
    }
}
