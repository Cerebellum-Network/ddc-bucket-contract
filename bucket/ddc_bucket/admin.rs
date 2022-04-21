//! The privileged interface for admin tasks.

use ink_lang::{EmitEvent, StaticEnv};

use crate::ddc_bucket::{AccountId, Balance, Cash, DdcBucket, GrantPermission, Result, RevokePermission};
use crate::ddc_bucket::perm::entity::Permission;
use crate::ddc_bucket::perm::store::PermStore;

impl DdcBucket {
    pub fn message_admin_grant_permission(&mut self, grantee: AccountId, permission: Permission, is_granted: bool) -> Result<()> {
        self.only_with_permission(Permission::SuperAdmin)?;

        if is_granted {
            self.perms.grant_permission(grantee, &permission);
            Self::env().emit_event(GrantPermission { account_id: grantee, permission });
            Self::capture_fee_and_refund(PermStore::RECORD_SIZE)?;
        } else {
            self.perms.revoke_permission(grantee, &permission);
            Self::env().emit_event(RevokePermission { account_id: grantee, permission });
        }

        Ok(())
    }

    pub fn message_admin_withdraw(&mut self, amount: Balance) -> Result<()> {
        let admin = self.only_with_permission(Permission::SuperAdmin)?;
        Self::send_cash(admin, Cash(amount))
    }
}