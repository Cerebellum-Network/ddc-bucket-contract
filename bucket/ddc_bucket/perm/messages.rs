//! The public interface for permission management.

use ink_lang::StaticEnv;

use crate::ddc_bucket::{
    AccountId, DdcBucket, Result,
};
use crate::ddc_bucket::Error::Unauthorized;
use crate::ddc_bucket::perm::entity::Permission;

impl DdcBucket {
    pub fn only_with_permission(&self, permission: Permission) -> Result<AccountId> {
        let caller = Self::env().caller();
        if self.perms.has_permission(caller, permission) {
            Ok(caller)
        } else {
            Err(Unauthorized)
        }
    }
}
