//! The public interface to change Params.

use crate::ddc_bucket::{DdcBucket, Result};

use super::store::{Params, ParamsId, ParamsStore};

impl DdcBucket {
    pub fn impl_change_params(store: &mut ParamsStore, params_id: ParamsId, params: Params) -> Result<()> {
        // TODO: permission.
        let record_size = store.change(params_id, params)?;
        Self::capture_fee_and_refund(record_size)
    }
}