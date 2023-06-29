//! The store to create and access Accounts.

use ink_prelude::vec::Vec;
use ink_storage::Mapping;
use ink_storage::traits::{SpreadAllocate, SpreadLayout};
use scale::Encode;

use crate::ddc_bucket::AccountId;

use super::entity::Permission;

pub type TrustedBy = AccountId;


type PermKey = Vec<u8>;

#[derive(SpreadAllocate, SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(ink_storage::traits::StorageLayout, Debug))]
pub struct PermStore {
    pub perms: Mapping<PermKey, bool>
}


impl PermStore {
    pub fn grant_permission(&mut self, account_id: AccountId, permission: Permission) {
        let key = (account_id, permission).encode();
        self.perms.insert(key, &true);
    }

    pub fn revoke_permission(&mut self, account_id: AccountId, permission: Permission) {
        let key = (account_id, permission).encode();
        self.perms.remove(&key);
    }

    pub fn has_permission(&self, account_id: AccountId, permission: Permission) -> bool {
        let key = (account_id, permission).encode();
        if self.perms.contains(&key) {
            return true;
        }

        let admin_key = (account_id, Permission::SuperAdmin).encode();
        self.perms.contains(&admin_key)
    }
}
