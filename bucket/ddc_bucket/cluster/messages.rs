use ink_lang::{EmitEvent, StaticEnv};

use crate::ddc_bucket::{ClusterCreated, DdcBucket, Result};

use super::entity::{ClusterId, ClusterParams};

impl DdcBucket {
    pub fn message_cluster_create(&mut self, cluster_params: ClusterParams) -> Result<ClusterId> {
        let cluster_id = self.clusters.create(cluster_params.clone());
        Self::env().emit_event(ClusterCreated { cluster_id, cluster_params });
        Ok(cluster_id)
    }
}
