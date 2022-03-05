use ink_storage::{
    collections::Vec as InkVec,
    traits,
};

use crate::ddc_bucket::{
    AccountId, Error::*, Result,
    schedule::Schedule,
};

use super::entity::{Flow, FlowId};

#[derive(traits::SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout, Debug))]
pub struct FlowStore(pub InkVec<Flow>);

impl FlowStore {
    pub fn create(&mut self, from: AccountId, schedule: Schedule) -> FlowId {
        let flow = Flow {
            from,
            schedule,
        };
        let flow_id = self.0.len();
        self.0.push(flow);
        flow_id
    }

    pub fn get(&self, flow_id: FlowId) -> Result<&Flow> {
        self.0.get(flow_id).ok_or(FlowDoesNotExist)
    }

    pub fn get_mut(&mut self, flow_id: FlowId) -> Result<&mut Flow> {
        self.0.get_mut(flow_id).ok_or(FlowDoesNotExist)
    }
}
