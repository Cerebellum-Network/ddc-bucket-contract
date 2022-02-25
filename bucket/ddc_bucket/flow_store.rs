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
use crate::ddc_bucket::schedule::Schedule;

use super::billing_flow::{BillingFlow, FlowId};

#[derive(SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(StorageLayout, Debug))]
pub struct FlowStore(pub InkVec<BillingFlow>);

impl FlowStore {
    pub fn create(&mut self, from: AccountId, schedule: Schedule) -> FlowId {
        let flow = BillingFlow {
            from,
            schedule,
        };
        let flow_id = self.0.len();
        self.0.push(flow);
        flow_id
    }

    pub fn get(&self, flow_id: FlowId) -> Result<&BillingFlow> {
        self.0.get(flow_id).ok_or(FlowDoesNotExist)
    }

    pub fn get_mut(&mut self, flow_id: FlowId) -> Result<&mut BillingFlow> {
        self.0.get_mut(flow_id).ok_or(FlowDoesNotExist)
    }
}
