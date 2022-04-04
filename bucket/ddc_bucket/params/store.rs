//! The store where to create and access Nodes.

use ink_prelude::string::String;
use ink_storage::{
    collections::Vec as InkVec,
    traits,
};

use crate::ddc_bucket::contract_fee::SIZE_VEC;

pub type ParamsId = u32;
pub type Params = String;

#[derive(traits::SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout, Debug))]
pub struct ParamsStore(pub InkVec<Params>);

impl ParamsStore {
    pub fn create(&mut self, params: Params) -> (ParamsId, usize) {
        let record_size = SIZE_VEC + params.len();
        let params_id = self.0.len();
        self.0.push(params);
        (params_id, record_size)
    }
}
