//! The store to create and access Accounts.
use ink_prelude::vec::Vec;
use ink_storage::{Mapping};
use scale::Encode;

use crate::ddc_bucket::AccountId;

use super::entity::Permission;

pub type TrustedBy = AccountId;

type PermKey = Vec<u8>;

pub const PERMS_STORE_KEY: u32 = openbrush::storage_unique_key!(PermStore);
#[openbrush::upgradeable_storage(PERMS_STORE_KEY)]
#[derive(Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct PermStore {
    pub perms: Mapping<PermKey, bool>,
    _reserved: Option<()>
}


impl PermStore {
    pub fn grant_permission(&mut self, account_id: AccountId, permission: &Permission) {
        let key = (account_id, permission).encode();
        self.perms.insert(key, &true);
    }

    pub fn revoke_permission(&mut self, account_id: AccountId, permission: &Permission) {
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
