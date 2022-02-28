use ink_lang::{EmitEvent, StaticEnv};

use crate::ddc_bucket::{Balance, ClusterId, DdcBucket, Result, ServiceCreated};

use super::entity::{ServiceId, ServiceParams};

impl DdcBucket {
    pub fn message_service_create(&mut self, cluster_id: ClusterId, rent_per_month: Balance, service_params: ServiceParams) -> Result<ServiceId> {
        let provider_id = Self::env().caller();
        let service_id = self.services.create(provider_id, rent_per_month, service_params.clone());

        self.clusters.add_service(cluster_id, service_id)?;

        Self::env().emit_event(ServiceCreated { service_id, provider_id, rent_per_month, service_params });
        Ok(service_id)
    }
}
