use ink_lang::{EmitEvent, StaticEnv};

use crate::ddc_bucket::{ClusterCreated, DdcBucket, Result};

use super::entity::{ClusterId, ClusterParams};

impl DdcBucket {
    pub fn message_cluster_create(&mut self, cluster_params: ClusterParams) -> Result<ClusterId> {
        let (cluster_id, record_size) = self.clusters.create(cluster_params.clone());
        Self::capture_fee_and_refund(record_size)?;
        Self::env().emit_event(ClusterCreated { cluster_id, cluster_params });
        Ok(cluster_id)
    }
}
