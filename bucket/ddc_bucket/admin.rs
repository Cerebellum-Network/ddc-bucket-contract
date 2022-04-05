//! The privileged interface for admin tasks.

use ink_lang::StaticEnv;

use crate::ddc_bucket::{
    AccountId, Balance, Cash,
    DdcBucket, Error::UnauthorizedAdmin,
    Result,
};

impl DdcBucket {
    pub fn message_admin_withdraw(&mut self, amount: Balance) {
        let admin = self.only_admin().unwrap();
        Self::send_cash(admin, Cash(amount)).unwrap();
    }

    pub fn message_admin_change(&mut self, new_admin: AccountId) {
        let _ = self.only_admin().unwrap();
        *self.admin_id = new_admin;
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