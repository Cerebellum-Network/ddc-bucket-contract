//! The privileged interface for admin tasks.

use crate::ddc_bucket::perm::entity::Permission;
use crate::ddc_bucket::{AccountId, Balance, Cash, DdcBucket, Result};

impl DdcBucket {
    pub fn message_admin_grant_permission(
        &mut self,
        grantee: AccountId,
        permission: Permission,
        is_granted: bool,
    ) -> Result<()> {
        self.only_with_permission(Permission::SuperAdmin)?;
        self.impl_grant_permission(grantee, permission)
    }

    pub fn message_admin_withdraw(&mut self, amount: Balance) -> Result<()> {
        let admin = self.only_with_permission(Permission::SuperAdmin)?;
        Self::send_cash(admin, Cash(amount))
    }
}
