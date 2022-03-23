use ink_storage::{
    collections::Vec as InkVec,
    traits,
};

use crate::ddc_bucket::{
    Error::*,
    Result,
    node::entity::NodeId,
};
use crate::ddc_bucket::flow::Flow;

use super::entity::{Deal, DealId};

#[derive(traits::SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout, Debug))]
pub struct DealStore(pub InkVec<Deal>);

impl DealStore {
    pub fn create(&mut self, node_id: NodeId, flow: Flow) -> DealId {
        let deal = Deal {
            node_id,
            flow,
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
