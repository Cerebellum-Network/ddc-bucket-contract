//! The store where to create and access Nodes.

use ink_prelude::string::String;
use ink_prelude::vec::Vec;
use ink_storage::traits::{SpreadAllocate, SpreadLayout, StorageLayout};

use crate::ddc_bucket::{Error::ParamsDoesNotExist, Result};
use crate::ddc_bucket::Error::ParamsTooBig;

pub type ParamsId = u32;
pub type Params = String;

pub const PARAMS_MAX_LEN: usize = 100_000;

#[derive(SpreadAllocate, SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(StorageLayout, Debug))]
pub struct ParamsStore(pub Vec<Params>);

impl ParamsStore {
    pub fn create(&mut self, params: Params) -> Result<ParamsId> {
        if params.len() > PARAMS_MAX_LEN {
            return Err(ParamsTooBig);
        }
        let params_id: ParamsId = self.0.len().try_into().unwrap();
        self.0.push(params);
        Ok(params_id)
    }

    pub fn change(&mut self, params_id: ParamsId, params: Params) -> Result<usize> {
        let current = self.0.get_mut(params_id as usize).ok_or(ParamsDoesNotExist)?;

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
        self.0.get(params_id as usize).ok_or(ParamsDoesNotExist)
    }
}
