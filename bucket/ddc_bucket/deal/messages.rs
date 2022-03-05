use ink_lang::{EmitEvent, StaticEnv};

use crate::ddc_bucket::{DdcBucket, ProviderWithdraw, Result, VNodeId};

use super::entity::{DealId, DealStatus};

impl DdcBucket {
    pub fn message_provider_withdraw(&mut self, deal_id: DealId) -> Result<()> {
        let caller = Self::env().caller();

        let deal = self.deals.get_mut(deal_id)?;

        // Find where to distribute the revenues.
        let revenue_account_id = {
            let vnode = self.vnodes.get(deal.vnode_id)?;
            // Authorize only the vnode owner to trigger the distribution.
            vnode.only_owner(caller)?;
            vnode.revenue_account_id()
        };

        let cash = Self::billing_settle_flow(&mut self.accounts, &mut deal.flow)?;

        Self::env().emit_event(ProviderWithdraw { provider_id: revenue_account_id, deal_id, value: cash.peek() });

        Self::send_cash(revenue_account_id, cash)
    }

    pub fn message_deal_get_status(&self, deal_id: DealId) -> Result<DealStatus> {
        let deal = self.deals.get(deal_id)?;
        let estimated_rent_end_ms = self.billing_flow_covered_until(&deal.flow)?;

        Ok(DealStatus {
            vnode_id: deal.vnode_id,
            estimated_rent_end_ms,
        })
    }

    pub fn deal_create(&mut self, vnode_id: VNodeId) -> Result<DealId> {
        let payer_id = Self::env().caller();

        // Start the payment flow for a deal.
        let rent_per_month = self.vnodes.get(vnode_id)?.rent_per_month;
        let flow = Self::billing_start_flow(&mut self.accounts, payer_id, rent_per_month)?;
        let deal_id = self.deals.create(vnode_id, flow);

        Ok(deal_id)
    }
}