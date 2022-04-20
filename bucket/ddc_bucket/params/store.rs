//! The store where to create and access Nodes.

use ink_prelude::string::String;
use ink_storage::{
    collections::Vec as InkVec,
    traits,
};

use crate::ddc_bucket::{Error::ParamsDoesNotExist, Result};
use crate::ddc_bucket::contract_fee::SIZE_VEC;
use crate::ddc_bucket::Error::ParamsTooBig;

pub type ParamsId = u32;
pub type Params = String;

pub const PARAMS_MAX_LEN: usize = 100_000;

#[derive(traits::SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout, Debug))]
pub struct ParamsStore(pub InkVec<Params>);

impl ParamsStore {
    pub fn create(&mut self, params: Params) -> Result<(ParamsId, usize)> {
        if params.len() > PARAMS_MAX_LEN {
            return Err(ParamsTooBig);
        }
        let record_size = SIZE_VEC + params.len();
        let params_id = self.0.len();
        self.0.push(params);
        Ok((params_id, record_size))
    }

    pub fn change(&mut self, params_id: ParamsId, params: Params) -> Result<usize> {
        let current = self.0.get_mut(params_id).ok_or(ParamsDoesNotExist)?;

        if params.len() > PARAMS_MAX_LEN {
            return Err(ParamsTooBig);
        }
        let record_size = if params.len() > current.len() {
            params.len() - current.len()
        } else { 0 };

        *current = params;
        Ok(record_size)
    }

    pub fn get(&self, params_id: ParamsId) -> Result<&Params> {
        self.0.get(params_id).ok_or(ParamsDoesNotExist)
    }
}
