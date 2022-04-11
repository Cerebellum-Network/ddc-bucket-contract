//! The store to create and access Accounts.

use ink_env::hash::{Blake2x256, HashOutput};
use ink_prelude::vec::Vec;
use ink_storage::{
    collections::HashMap,
    traits::{PackedLayout, SpreadLayout, StorageLayout},
};
use scale::{Decode, Encode};

use crate::ddc_bucket::AccountId;
use crate::ddc_bucket::contract_fee::{SIZE_ACCOUNT_ID, SIZE_HASHMAP, SIZE_PER_RECORD};

use super::entity::Perm;

pub type TrustedBy = AccountId;


type PermKey = (AccountId, Vec<u8>);

#[derive(SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(StorageLayout, Debug))]
pub struct PermStore(pub HashMap<PermKey, bool>);
// TODO: Switch to Mapping (must upgrade ink first).


impl PermStore {
    pub const RECORD_SIZE: usize =
        SIZE_PER_RECORD + SIZE_HASHMAP + SIZE_ACCOUNT_ID + SIZE_ACCOUNT_ID + 1;

    pub fn grant_perm(&mut self, account_id: AccountId, perm: Perm) {
        let key = (account_id, perm.encode());
        self.0.insert(key, true);
    }

    pub fn revoke_perm(&mut self, account_id: AccountId, perm: Perm) {
        let key = (account_id, perm.encode());
        self.0.take(&key);
    }

    pub fn has_perm(&self, account_id: AccountId, perm: Perm) -> bool {
        let key = (account_id, perm.encode());
        self.0.contains_key(&key)
    }
}
