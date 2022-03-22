use ink_lang::{EmitEvent, StaticEnv};

use crate::ddc_bucket::{Balance, DdcBucket, Result, VNodeCreated};

use super::entity::{VNodeId, VNodeParams};

impl DdcBucket {
    pub fn message_vnode_create(&mut self, rent_per_month: Balance, vnode_params: VNodeParams) -> Result<VNodeId> {
        let provider_id = Self::env().caller();
        let (vnode_id, record_size) = self.vnodes.create(provider_id, rent_per_month, vnode_params.clone());

        Self::capture_fee_and_refund(record_size)?;
        Self::env().emit_event(VNodeCreated { vnode_id, provider_id, rent_per_month, vnode_params });
        Ok(vnode_id)
    }
}
