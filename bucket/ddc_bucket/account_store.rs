use ink_prelude::{
    vec, vec::Vec,
};
use ink_storage::{
    collections::{HashMap, hashmap::Entry::*},
    collections::Stash,
    collections::Vec as InkVec,
    traits::{PackedLayout, SpreadLayout, StorageLayout},
};

use crate::ddc_bucket::{AccountId, Balance, Error::*, Result};

use super::billing_account::BillingAccount;
use super::cash::Cash;

#[derive(SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(StorageLayout, Debug))]
pub struct AccountStore(pub HashMap<AccountId, BillingAccount>);

impl AccountStore {
    pub fn deposit(&mut self, to: AccountId, cash: Cash) {
        match self.0.entry(to) {
            Vacant(e) => {
                let mut account = BillingAccount::new();
                account.deposit(cash);
                e.insert(account);
            }
            Occupied(mut e) => {
                let account = e.get_mut();
                account.deposit(cash);
            }
        };
    }

    pub fn balance(&self, account_id: &AccountId) -> Balance {
        match self.0.get(account_id) {
            None => 0,
            Some(account) => account.deposit.peek(),
        }
    }

    pub fn get(&self, account_id: &AccountId) -> Result<&BillingAccount> {
        self.0.get(account_id).ok_or(AccountDoesNotExist)
    }

    pub fn get_mut(&mut self, account_id: &AccountId) -> Result<&mut BillingAccount> {
        self.0.get_mut(account_id).ok_or(AccountDoesNotExist)
    }
}
