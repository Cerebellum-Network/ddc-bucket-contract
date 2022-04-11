//! The privileged interface for admin tasks.

use crate::ddc_bucket::{
    AccountId, Balance, Cash,
    DdcBucket,
    Result,
};
use crate::ddc_bucket::perm::entity::Perm;
use crate::ddc_bucket::perm::store::PermStore;

impl DdcBucket {
    pub fn message_admin_grant_perm(&mut self, grantee: AccountId, perm: Perm, is_granted: bool) -> Result<()> {
        self.only_with_perm(Perm::SuperAdmin)?;

        if is_granted {
            self.perms.grant_perm(grantee, perm);
            Self::capture_fee_and_refund(PermStore::RECORD_SIZE)?;
        } else {
            self.perms.revoke_perm(grantee, perm);
        }

        Ok(())
    }

    pub fn message_admin_withdraw(&mut self, amount: Balance) -> Result<()> {
        let admin = self.only_with_perm(Perm::SuperAdmin)?;
        Self::send_cash(admin, Cash(amount))
    }
}