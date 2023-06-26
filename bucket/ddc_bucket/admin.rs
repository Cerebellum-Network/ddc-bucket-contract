//! The privileged interface for admin tasks.

use crate::ddc_bucket::perm::entity::Permission;
use crate::ddc_bucket::{AccountId, Balance, Cash, DdcBucket, PermissionGranted, PermissionRevoked, Result};
use ink_lang::codegen::{EmitEvent, StaticEnv};

impl DdcBucket {

    pub fn message_admin_grant_permission(
        &mut self,
        grantee: AccountId,
        permission: Permission,
    ) -> Result<()> {
        self.only_with_permission(Permission::SuperAdmin)?;
        self.impl_grant_permission(grantee, permission)?;
        
        Self::env().emit_event(PermissionGranted { 
            account_id: grantee,
            permission: permission
        });

        Ok(())
    }

    pub fn message_admin_revoke_permission(
        &mut self,
        grantee: AccountId,
        permission: Permission,
    ) -> Result<()> {
        self.only_with_permission(Permission::SuperAdmin)?;
        self.impl_revoke_permission(grantee, permission)?;
        
        Self::env().emit_event(PermissionRevoked { 
            account_id: grantee,
            permission: permission
        });

        Ok(())
    }

    pub fn message_admin_withdraw(&mut self, amount: Balance) -> Result<()> {
        let admin = self.only_with_permission(Permission::SuperAdmin)?;
        Self::send_cash(admin, Cash(amount))
    }

}
