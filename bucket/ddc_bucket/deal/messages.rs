use ink_lang::{EmitEvent, StaticEnv};

use crate::ddc_bucket::{DdcBucket, ProviderWithdraw, Result, NodeId};

use super::entity::{DealId, DealStatus};

impl DdcBucket {
    pub fn message_provider_withdraw(&mut self, deal_id: DealId) -> Result<()> {
        let caller = Self::env().caller();

        let deal = self.deals.get_mut(deal_id)?;

        // Find where to distribute the revenues.
        let revenue_account_id = {
            let node = self.nodes.get(deal.node_id)?;
            // Authorize only the node owner to trigger the distribution.
            node.only_owner(caller)?;
            node.revenue_account_id()
        };

        let now_ms = Self::env().block_timestamp();
        let cash = self.accounts.settle_flow(now_ms, &mut deal.flow)?;

        Self::env().emit_event(ProviderWithdraw { provider_id: revenue_account_id, deal_id, value: cash.peek() });

        Self::send_cash(revenue_account_id, cash)
    }

    pub fn message_deal_get_status(&self, deal_id: DealId) -> Result<DealStatus> {
        let deal = self.deals.get(deal_id)?;
        let estimated_rent_end_ms = self.accounts.flow_covered_until(&deal.flow)?;

        Ok(DealStatus {
            node_id: deal.node_id,
            estimated_rent_end_ms,
        })
    }

    pub fn deal_create(&mut self, node_id: NodeId) -> Result<DealId> {
        let payer_id = Self::env().caller();

        // Start the payment flow for a deal.
        let rent_per_month = self.nodes.get(node_id)?.rent_per_month;
        let start_ms = Self::env().block_timestamp();
        let flow = self.accounts.start_flow(start_ms, payer_id, rent_per_month)?;
        let deal_id = self.deals.create(node_id, flow);

        Ok(deal_id)
    }
}