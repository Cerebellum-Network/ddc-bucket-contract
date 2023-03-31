//! The public interface for permission management.

use ink_lang::codegen::{EmitEvent, StaticEnv};

use crate::ddc_bucket::{AccountId, DdcBucket, GrantPermission, Result, RevokePermission};
use crate::ddc_bucket::Error::Unauthorized;
use crate::ddc_bucket::perm::entity::Permission;

impl DdcBucket {
    pub fn impl_grant_permission(&mut self, account_id: AccountId, permission: Permission, is_granted: bool) -> Result<()> {
        if is_granted {
            self.perms.grant_permission(account_id, &permission);
            Self::env().emit_event(GrantPermission { account_id, permission });
            Ok(())
        } else {
            self.perms.revoke_permission(account_id, &permission);
            Self::env().emit_event(RevokePermission { account_id, permission });
            Ok(())
        }
    }

    pub fn only_with_permission(&self, permission: Permission) -> Result<AccountId> {
        let caller = Self::env().caller();
        if self.perms.has_permission(caller, permission) {
            Ok(caller)
        } else {
            Err(Unauthorized)
        }
    }
}
