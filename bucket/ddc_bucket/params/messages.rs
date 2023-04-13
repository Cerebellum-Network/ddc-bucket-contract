//! The public interface to change Params.

use crate::ddc_bucket::{DdcBucket, Result};
use crate::ddc_bucket::params::store::ParamsStoreTrait;
use super::store::{Params, ParamsId, BucketParamsStore, ClusterParamsStore, CdnNodeParamsStore, NodeParamsStore};

impl DdcBucket {

    pub fn impl_change_bucket_params(store: &mut BucketParamsStore, params_id: ParamsId, params: Params) -> Result<()> {
        store.change(params_id, params)?;
        Ok(())
    }

    pub fn impl_change_cluster_params(store: &mut ClusterParamsStore, params_id: ParamsId, params: Params) -> Result<()> {
        store.change(params_id, params)?;
        Ok(())
    }

    pub fn impl_change_cdn_node_params(store: &mut CdnNodeParamsStore, params_id: ParamsId, params: Params) -> Result<()> {
        store.change(params_id, params)?;
        Ok(())
    }

    pub fn impl_change_node_params(store: &mut NodeParamsStore, params_id: ParamsId, params: Params) -> Result<()> {
        store.change(params_id, params)?;
        Ok(())
    }

}