use ink_lang::{EmitEvent, StaticEnv};

use crate::ddc_bucket::{Balance, ClusterId, DdcBucket, Result, VNodeCreated};

use super::entity::{VNodeId, VNodeParams};

impl DdcBucket {
    pub fn message_vnode_create(&mut self, cluster_id: ClusterId, rent_per_month: Balance, service_params: VNodeParams) -> Result<VNodeId> {
        let provider_id = Self::env().caller();
        let vnode_id = self.vnodes.create(provider_id, rent_per_month, service_params.clone());

        self.clusters.add_vnode(cluster_id, vnode_id)?;

        Self::env().emit_event(VNodeCreated { vnode_id, provider_id, rent_per_month, vnode_params: service_params });
        Ok(vnode_id)
    }
}
