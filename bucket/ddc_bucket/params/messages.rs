//! The public interface to change Params.

use crate::ddc_bucket::{DdcBucket, Result};

use super::store::{Params, ParamsId, ParamsStore};

impl DdcBucket {
    pub fn impl_change_params(store: &mut ParamsStore, params_id: ParamsId, params: Params) -> Result<()> {
        store.change(params_id, params)?;
        Ok(())
    }
}