//! The privileged interface for admin tasks.

use crate::ddc_bucket::{
    AccountId, Balance, Cash,
    DdcBucket,
    Result,
};
use crate::ddc_bucket::perm::entity::Perm;
use crate::ddc_bucket::perm::store::PermStore;

impl DdcBucket {
    pub fn message_admin_grant_perm(&mut self, trustee: AccountId, perm: Perm) -> Result<()> {
        self.only_with_perm(Perm::SuperAdmin)?;
        self.perms.grant_perm(trustee, perm);

        Self::capture_fee_and_refund(PermStore::RECORD_SIZE)?;
        Ok(())
    }

    pub fn message_admin_revoke_perm(&mut self, trustee: AccountId, perm: Perm) -> Result<()> {
        self.only_with_perm(Perm::SuperAdmin)?;
        self.perms.revoke_perm(trustee, perm);
        Ok(())
    }

    pub fn message_admin_withdraw(&mut self, amount: Balance) -> Result<()> {
        let admin = self.only_with_perm(Perm::SuperAdmin)?;
        Self::send_cash(admin, Cash(amount))
    }
}