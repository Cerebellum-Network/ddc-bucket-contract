use ink_prelude::{
    vec, vec::Vec,
};
use ink_storage::{
    collections::{HashMap, hashmap::Entry::*},
    collections::Stash,
    collections::Vec as InkVec,
    traits::{PackedLayout, SpreadLayout, StorageLayout},
};

use crate::ddc_bucket::{AccountId, Balance, Error::*, Result};
use crate::ddc_bucket::billing_flow::FlowId;
use crate::ddc_bucket::deal::DealParams;
use crate::ddc_bucket::schedule::Schedule;

use super::deal::{Deal, DealId};
use super::service::ServiceId;

#[derive(SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(StorageLayout, Debug))]
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
