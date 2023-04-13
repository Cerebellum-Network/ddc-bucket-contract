//! The store where to create and access Nodes.
use ink_prelude::string::String;
use ink_prelude::vec::Vec;

use crate::ddc_bucket::{Error::ParamsDoesNotExist, Result};
use crate::ddc_bucket::Error::ParamsTooBig;

pub type ParamsId = u32;
pub type Params = String;

pub const PARAMS_MAX_LEN: usize = 100_000;

pub const BUCKET_PARAMS_STORE_KEY: u32 = openbrush::storage_unique_key!(BucketParamsStore);
#[openbrush::upgradeable_storage(BUCKET_PARAMS_STORE_KEY)]
#[derive(Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct BucketParamsStore {
    pub params: Vec<Params>,
    _reserved: Option<()>
}


pub const CLUSTER_PARAMS_STORE_KEY: u32 = openbrush::storage_unique_key!(ClusterParamsStore);
#[openbrush::upgradeable_storage(CLUSTER_PARAMS_STORE_KEY)]
#[derive(Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ClusterParamsStore {
    pub params: Vec<Params>,
    _reserved: Option<()>
}


pub const CDN_NODE_PARAMS_STORE_KEY: u32 = openbrush::storage_unique_key!(CdnNodeParamsStore);
#[openbrush::upgradeable_storage(CDN_NODE_PARAMS_STORE_KEY)]
#[derive(Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct CdnNodeParamsStore {
    pub params: Vec<Params>,
    _reserved: Option<()>
}


pub const NODE_PARAMS_STORE_KEY: u32 = openbrush::storage_unique_key!(NodeParamsStore);
#[openbrush::upgradeable_storage(NODE_PARAMS_STORE_KEY)]
#[derive(Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct NodeParamsStore {
    pub params: Vec<Params>,
    _reserved: Option<()>
}


pub trait ParamsStoreTrait {
    fn get_params_mut(&mut self) -> &mut Vec<Params>;

    fn get_params(&self) -> &Vec<Params>;

    fn create(&mut self, params: Params) -> Result<ParamsId> {
        if params.len() > PARAMS_MAX_LEN {
            return Err(ParamsTooBig);
        }
        let params_id: ParamsId = self.get_params().len().try_into().unwrap();
        self.get_params_mut().push(params);
        Ok(params_id)
    }

    fn change(&mut self, params_id: ParamsId, params: Params) -> Result<usize> {
        let current = self.get_params_mut().get_mut(params_id as usize).ok_or(ParamsDoesNotExist)?;

        if params.len() > PARAMS_MAX_LEN {
            return Err(ParamsTooBig);
        }
        let record_size = if params.len() > current.len() {
            params.len() - current.len()
        } else { 0 };

        *current = params;
        Ok(record_size)
    }

    fn get(&self, params_id: ParamsId) -> Result<&Params> {
        self.get_params().get(params_id as usize).ok_or(ParamsDoesNotExist)
    }
}


impl ParamsStoreTrait for BucketParamsStore {
    fn get_params_mut(&mut self) -> &mut Vec<Params> {
        &mut self.params
    }

    fn get_params(&self) -> &Vec<Params> {
        &self.params
    }
}

impl ParamsStoreTrait for ClusterParamsStore {
    fn get_params_mut(&mut self) -> &mut Vec<Params> {
        &mut self.params
    }

    fn get_params(&self) -> &Vec<Params> {
        &self.params
    }
}

impl ParamsStoreTrait for CdnNodeParamsStore {
    fn get_params_mut(&mut self) -> &mut Vec<Params> {
        &mut self.params
    }

    fn get_params(&self) -> &Vec<Params> {
        &self.params
    }
}

impl ParamsStoreTrait for NodeParamsStore {
    fn get_params_mut(&mut self) -> &mut Vec<Params> {
        &mut self.params
    }

    fn get_params(&self) -> &Vec<Params> {
        &self.params
    }
}
