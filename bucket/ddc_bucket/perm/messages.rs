//! The public interface for permission management.

use ink_lang::StaticEnv;

use crate::ddc_bucket::{
    AccountId, DdcBucket,
};

impl DdcBucket {
    pub fn message_perm_trust(&mut self, trustee: AccountId) {
        let trust_giver = Self::env().caller();
        self.perms.grant_perm(trustee, trust_giver);

        // TODO: Event.

        // TODO: Capture storage fee.
    }
}
