//! The public interface for permission management.

use ink_lang::StaticEnv;

use crate::ddc_bucket::{
    AccountId, DdcBucket, Result,
};
use crate::ddc_bucket::Error::Unauthorized;
use crate::ddc_bucket::perm::entity::Perm;

impl DdcBucket {
    pub fn only_with_perm(&self, perm: Perm) -> Result<AccountId> {
        let caller = Self::env().caller();
        if self.perms.has_perm(caller, perm) {
            Ok(caller)
        } else {
            Err(Unauthorized)
        }
    }
}
