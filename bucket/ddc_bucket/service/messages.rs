use ink_lang::{EmitEvent, StaticEnv};

use crate::ddc_bucket::{AccountId, Balance, Cash, DdcBucket, Deposit, FlowId, InsufficientBalance, Payable, Result, Schedule, ServiceCreated};

use super::entity::{ServiceId, ServiceParams};

impl DdcBucket {
    pub fn message_service_create(&mut self, rent_per_month: Balance, service_params: ServiceParams) -> Result<ServiceId> {
        let provider_id = Self::env().caller();
        let service_id = self.services.create(provider_id, rent_per_month, service_params.clone());
        Self::env().emit_event(ServiceCreated { service_id, provider_id, rent_per_month, service_params });
        Ok(service_id)
    }
}
