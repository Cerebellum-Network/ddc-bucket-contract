//! The public interface for permission management.

use ink_lang::StaticEnv;

use crate::ddc_bucket::{
    AccountId, DdcBucket, Result,
};
use crate::ddc_bucket::Error::Unauthorized;
use crate::ddc_bucket::perm::entity::Perm;
use crate::ddc_bucket::perm::store::PermStore;

impl DdcBucket {
    pub fn message_perm_trust(&mut self, trustee: AccountId) -> Result<()> {
        let trust_giver = Self::env().caller();
        let perm = Perm::TrustedBy(trust_giver);
        self.perms.grant_perm(trustee, perm);

        Self::capture_fee_and_refund(PermStore::RECORD_SIZE)?;
        Ok(())
    }

    pub fn message_perm_has_trust(&self, trustee: AccountId, trust_giver: AccountId) -> bool {
        let perm = Perm::TrustedBy(trust_giver);
        self.perms.has_perm(trustee, perm)
    }

    pub fn only_with_perm(&self, perm: Perm) -> Result<AccountId> {
        let caller = Self::env().caller();
        if self.perms.has_perm(caller, perm) {
            Ok(caller)
        } else {
            Err(Unauthorized)
        }
    }
}
