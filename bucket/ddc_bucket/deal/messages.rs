use ink_lang::{EmitEvent, StaticEnv};

use crate::ddc_bucket::{DdcBucket, ProviderWithdraw, Result, ServiceId};

use super::entity::{DealId, DealParams, DealStatus};

impl DdcBucket {
    pub fn message_provider_withdraw(&mut self, deal_id: DealId) -> Result<()> {
        let caller = Self::env().caller();

        let (flow_id, service_id) = {
            let deal = self.deals.get(deal_id)?;
            (deal.flow_id, deal.service_id)
        };

        // Find where to distribute the revenues.
        let revenue_account_id = {
            let service = self.services.get(service_id)?;
            // Authorize only the service owner to trigger the distribution.
            service.only_owner(caller)?;
            service.revenue_account_id()
        };

        let cash = self.billing_settle_flow(flow_id)?;

        Self::env().emit_event(ProviderWithdraw { provider_id: revenue_account_id, deal_id, value: cash.peek() });

        Self::send_cash(revenue_account_id, cash)
    }

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