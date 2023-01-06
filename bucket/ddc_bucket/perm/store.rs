//! The store to create and access Accounts.

use ink_prelude::vec::Vec;
use ink_storage::{
    collections::HashMap,
    traits,
};
use scale::Encode;

use crate::ddc_bucket::AccountId;

use super::entity::Permission;

pub type TrustedBy = AccountId;


type PermKey = Vec<u8>;

#[derive(traits::SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout, Debug))]
pub struct PermStore(pub HashMap<PermKey, bool>);
// TODO: Switch to Mapping (must upgrade ink first).


impl PermStore {
    pub fn grant_permission(&mut self, account_id: AccountId, permission: &Permission) {
        let key = (account_id, permission).encode();
        self.0.insert(key, true);
    }

    pub fn revoke_permission(&mut self, account_id: AccountId, permission: &Permission) {
        let key = (account_id, permission).encode();
        self.0.take(&key);
    }

    pub fn has_permission(&self, account_id: AccountId, permission: Permission) -> bool {
        let key = (account_id, permission).encode();
        if self.0.contains_key(&key) {
            return true;
        }

        let admin_key = (account_id, Permission::SuperAdmin).encode();
        self.0.contains_key(&admin_key)
    }
}
