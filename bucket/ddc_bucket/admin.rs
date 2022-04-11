//! The privileged interface for admin tasks.

use ink_lang::StaticEnv;

use crate::ddc_bucket::{
    AccountId, Balance, Cash,
    DdcBucket, Error::UnauthorizedAdmin,
    Result,
};
use crate::ddc_bucket::perm::entity::Perm;
use crate::ddc_bucket::perm::store::PermStore;

impl DdcBucket {
    pub fn message_admin_withdraw(&mut self, amount: Balance) {
        let admin = self.only_admin().unwrap();
        Self::send_cash(admin, Cash(amount)).unwrap();
    }

    pub fn message_admin_change(&mut self, new_admin: AccountId) -> Result<()> {
        self.only_admin()?;
        *self.admin_id = new_admin;
        Ok(())
    }

    pub fn message_admin_grant(&mut self, trustee: AccountId, perm: Perm) -> Result<()> {
        let _ = self.only_admin()?;
        self.perms.grant_perm(trustee, perm);

        Self::capture_fee_and_refund(PermStore::RECORD_SIZE)?;
        Ok(())
    }

    pub fn only_admin(&self) -> Result<AccountId> {
        let admin = Self::env().caller();
        if admin == *self.admin_id {
            Ok(admin)
        } else {
            Err(UnauthorizedAdmin)
        }
    }
}