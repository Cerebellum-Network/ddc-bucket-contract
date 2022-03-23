use ink_lang::{EmitEvent, StaticEnv};

use crate::ddc_bucket::{Balance, DdcBucket, Result, NodeCreated};

use super::entity::{NodeId, NodeParams};

impl DdcBucket {
    pub fn message_node_create(&mut self, rent_per_month: Balance, node_params: NodeParams) -> Result<NodeId> {
        let provider_id = Self::env().caller();
        let (node_id, record_size) = self.nodes.create(provider_id, rent_per_month, node_params.clone());

        Self::capture_fee_and_refund(record_size)?;
        Self::env().emit_event(NodeCreated { node_id: node_id, provider_id, rent_per_month, node_params: node_params });
        Ok(node_id)
    }
}