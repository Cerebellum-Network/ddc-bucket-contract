//! The public interface for permission management.

use ink_lang::StaticEnv;

use crate::ddc_bucket::{
    AccountId, DdcBucket, Result,
};
use crate::ddc_bucket::perm::store::PermStore;

impl DdcBucket {
    pub fn message_perm_trust(&mut self, trustee: AccountId) -> Result<()> {
        let trust_giver = Self::env().caller();
        self.perms.grant_perm(trustee, trust_giver);

        Self::capture_fee_and_refund(PermStore::RECORD_SIZE)?;
        Ok(())
    }
}
