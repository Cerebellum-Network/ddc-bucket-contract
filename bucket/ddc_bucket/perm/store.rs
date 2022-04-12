//! The store to create and access Accounts.

use ink_prelude::vec::Vec;
use ink_storage::{
    collections::HashMap,
    traits::{SpreadLayout, StorageLayout},
};
use scale::Encode;

use crate::ddc_bucket::AccountId;
use crate::ddc_bucket::contract_fee::{SIZE_ACCOUNT_ID, SIZE_HASHMAP, SIZE_PER_RECORD, SIZE_VEC};

use super::entity::Permission;

pub type TrustedBy = AccountId;


type PermKey = Vec<u8>;

#[derive(SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(StorageLayout, Debug))]
pub struct PermStore(pub HashMap<PermKey, bool>);
// TODO: Switch to Mapping (must upgrade ink first).


impl PermStore {
    pub const RECORD_SIZE: usize =
        SIZE_PER_RECORD + SIZE_HASHMAP + SIZE_VEC // Overhead.
            + SIZE_ACCOUNT_ID // Grantee.
            + 1 + SIZE_ACCOUNT_ID // The permission enum and its largest parameter.
            + 1; // Boolean value.

    pub fn grant_permission(&mut self, account_id: AccountId, permission: Permission) {
        let key = (account_id, permission).encode();
        self.0.insert(key, true);
    }

    pub fn revoke_permission(&mut self, account_id: AccountId, permission: Permission) {
        let key = (account_id, permission).encode();
        self.0.take(&key);
    }

    pub fn has_permission(&self, account_id: AccountId, permission: Permission) -> bool {
        let key = (account_id, permission).encode();
        self.0.contains_key(&key)
    }
}
