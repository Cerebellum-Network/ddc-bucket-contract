//! The store to create and access Accounts.

use ink_prelude::vec::Vec;
use ink_storage::Mapping;
use ink_storage::traits::{SpreadAllocate, SpreadLayout, StorageLayout};
use scale::Encode;

use crate::ddc_bucket::AccountId;

use super::entity::Permission;

pub type TrustedBy = AccountId;


type PermKey = Vec<u8>;

#[derive(SpreadAllocate, SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(StorageLayout, Debug))]
pub struct PermStore(pub Mapping<PermKey, bool>);
// TODO: Switch to Mapping (must upgrade ink first).


impl PermStore {
    pub fn grant_permission(&mut self, account_id: AccountId, permission: &Permission) {
        let key = (account_id, permission).encode();
        self.0.insert(key, &true);
    }

    pub fn revoke_permission(&mut self, account_id: AccountId, permission: &Permission) {
        let key = (account_id, permission).encode();
        self.0.remove(&key);
    }

    pub fn has_permission(&self, account_id: AccountId, permission: Permission) -> bool {
        let key = (account_id, permission).encode();
        if self.0.contains(&key) {
            return true;
        }

        let admin_key = (account_id, Permission::SuperAdmin).encode();
        self.0.contains(&admin_key)
    }
}
