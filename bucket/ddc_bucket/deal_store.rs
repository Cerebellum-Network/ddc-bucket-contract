use ink_storage::{
    collections::Vec as InkVec,
    traits,
};

use crate::ddc_bucket::{
    billing_flow::FlowId, deal::{Deal, DealId, DealParams},
    Error::*,
    Result,
    service::entity::ServiceId,
};

#[derive(traits::SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout, Debug))]
pub struct DealStore(pub InkVec<Deal>);

impl DealStore {
    pub fn create(&mut self, service_id: ServiceId, flow_id: FlowId, deal_params: DealParams) -> DealId {
        let deal = Deal {
            service_id,
            flow_id,
            deal_params,
        };
        let deal_id = self.0.len();
        self.0.push(deal);
        deal_id
    }

    pub fn get(&self, deal_id: DealId) -> Result<&Deal> {
        self.0.get(deal_id).ok_or(DealDoesNotExist)
    }

    pub fn get_mut(&mut self, deal_id: DealId) -> Result<&mut Deal> {
        self.0.get_mut(deal_id).ok_or(DealDoesNotExist)
    }
}
