use ink_lang::StaticEnv;

use crate::ddc_bucket::{DdcBucket, Result, ServiceId};

use super::entity::{DealId, DealParams, DealStatus};

impl DdcBucket {
    pub fn message_deal_get_status(&self, deal_id: DealId) -> Result<DealStatus> {
        let deal = self.deals.get(deal_id)?;
        let estimated_rent_end_ms = self.billing_flow_covered_until(deal.flow_id)?;

        Ok(DealStatus {
            service_id: deal.service_id,
            estimated_rent_end_ms,
            deal_params: deal.deal_params.clone(),
        })
    }

    pub fn deal_create(&mut self, service_id: ServiceId, deal_params: DealParams) -> Result<DealId> {
        let payer_id = Self::env().caller();

        // Start the payment flow for a deal.
        let rent_per_month = self.services.get(service_id)?.rent_per_month;
        let flow_id = self.billing_start_flow(payer_id, rent_per_month)?;
        let deal_id = self.deals.create(service_id, flow_id, deal_params);

        Ok(deal_id)
    }
}