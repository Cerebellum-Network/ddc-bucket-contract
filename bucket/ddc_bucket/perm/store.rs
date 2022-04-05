//! The store to create and access Accounts.

use ink_storage::{
    collections::HashMap,
    traits,
};

use crate::ddc_bucket::AccountId;

pub type TrustedBy = AccountId;

#[derive(traits::SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout, Debug))]
pub struct PermStore(pub HashMap<(AccountId, TrustedBy), bool>);

impl PermStore {
    pub fn grant_perm(&mut self, account_id: AccountId, perm: TrustedBy) {
        let key = (account_id, perm);
        self.0.insert(key, true);
    }

    pub fn revoke_perm(&mut self, account_id: AccountId, perm: TrustedBy) {
        let key = (account_id, perm);
        self.0.take(&key);
    }

    pub fn has_perm(&self, account_id: AccountId, perm: TrustedBy) -> bool {
        let key = (account_id, perm);
        self.0.contains_key(&key)
    }
}
