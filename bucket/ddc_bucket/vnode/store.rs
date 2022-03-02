use ink_prelude::vec::Vec;
use ink_storage::{
    collections::Vec as InkVec,
    traits,
};

use crate::ddc_bucket::{AccountId, Balance, Error::*, Result};
use super::entity::{VNode, VNodeId, VNodeParams};

#[derive(traits::SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout, Debug))]
pub struct VNodeStore(pub InkVec<VNode>);

impl VNodeStore {
    pub fn create(&mut self, provider_id: AccountId, rent_per_month: Balance, vnode_params: VNodeParams) -> VNodeId {
        let vnode_id = self.0.len();
        let vnode = VNode {
            vnode_id,
            provider_id,
            rent_per_month,
            vnode_params,
        };
        self.0.push(vnode);
        vnode_id
    }

    pub fn get(&self, vnode_id: VNodeId) -> Result<&VNode> {
        self.0.get(vnode_id).ok_or(VNodeDoesNotExist)
    }

    pub fn list(&self, offset: u32, limit: u32, filter_provider_id: Option<AccountId>) -> (Vec<VNode>, u32) {
        let mut vnodes = Vec::with_capacity(limit as usize);
        for vnode_id in offset..offset + limit {
            let vnode = match self.0.get(vnode_id) {
                None => break, // No more items, stop.
                Some(vnode) => vnode,
            };
            // Apply the filter if given.
            if let Some(provider_id) = filter_provider_id {
                if provider_id != vnode.provider_id {
                    continue; // Skip non-matches.
                }
            }
            vnodes.push(vnode.clone());
        }
        (vnodes, self.0.len())
    }
}
