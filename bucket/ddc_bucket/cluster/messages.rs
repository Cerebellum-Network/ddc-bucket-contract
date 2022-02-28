use ink_lang::{EmitEvent, StaticEnv};

use crate::ddc_bucket::{Balance, DdcBucket, Result, ClusterCreated};

use super::entity::{ClusterId, ClusterParams};

impl DdcBucket {
    pub fn message_cluster_create(&mut self, rent_per_month: Balance, cluster_params: ClusterParams) -> Result<ClusterId> {
        let cluster_id = self.clusters.create(rent_per_month, cluster_params.clone());
        Self::env().emit_event(ClusterCreated { cluster_id, rent_per_month, cluster_params });
        Ok(cluster_id)
    }
}
