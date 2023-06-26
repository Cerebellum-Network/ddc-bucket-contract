//! The public interface for permission management.

use ink_lang::codegen::{EmitEvent, StaticEnv};

use crate::ddc_bucket::{AccountId, DdcBucket, PermissionGranted, Result, PermissionRevoked};
use crate::ddc_bucket::Error::Unauthorized;
use crate::ddc_bucket::perm::entity::Permission;

impl DdcBucket {

    pub fn grant_permission(&mut self, account_id: AccountId, permission: Permission) -> Result<()> {
        self.perms.grant_permission(account_id, &permission);
        Self::env().emit_event(PermissionGranted { 
            account_id, 
            permission 
        });
        Ok(())
    }

    pub fn revoke_permission(&mut self, account_id: AccountId, permission: Permission) -> Result<()> {
        self.perms.revoke_permission(account_id, &permission);
        Self::env().emit_event(PermissionRevoked { 
            account_id, 
            permission 
        });
        Ok(())
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
