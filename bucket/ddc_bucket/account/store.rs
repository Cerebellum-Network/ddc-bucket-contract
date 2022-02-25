use ink_storage::{
    collections::{HashMap, hashmap::Entry::*},
    traits,
};

use crate::ddc_bucket::{
    AccountId, Balance, cash::Cash, Error::*,
    Result,
};

use super::entity::Account;

#[derive(traits::SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout, Debug))]
pub struct AccountStore(pub HashMap<AccountId, Account>);

impl AccountStore {
    pub fn deposit(&mut self, to: AccountId, cash: Cash) {
        match self.0.entry(to) {
            Vacant(e) => {
                let mut account = Account::new();
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

    pub fn get(&self, account_id: &AccountId) -> Result<&Account> {
        self.0.get(account_id).ok_or(AccountDoesNotExist)
    }

    pub fn get_mut(&mut self, account_id: &AccountId) -> Result<&mut Account> {
        self.0.get_mut(account_id).ok_or(AccountDoesNotExist)
    }
}