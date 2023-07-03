//! The store to create and access Accounts.

use super::entity::Account;
use crate::ddc_bucket::flow::Flow;
use crate::ddc_bucket::{
    cash::Cash, currency::CurrencyConverter, schedule::Schedule, AccountId, Balance, Error::*,
    Result,
};
use ink_prelude::vec::Vec;
use ink_storage::traits::{SpreadAllocate, SpreadLayout};
use ink_storage::Mapping;

// https://use.ink/datastructures/storage-layout#packed-vs-non-packed-layout
// There is a buffer with only limited capacity (around 16KB in the default configuration) available.
pub const MAX_ACCOUNTS_LEN_IN_VEC: usize = 400;

#[derive(Default, SpreadLayout, SpreadAllocate)]
#[cfg_attr(feature = "std", derive(ink_storage::traits::StorageLayout, Debug))]
pub struct AccountStore {
    pub accounts: Mapping<AccountId, Account>,
    // todo: remove this vector as it can store an arbitrary number of elements and easily exceed 16KB limit
    pub accounts_keys: Vec<AccountId>,
}

impl AccountStore {
    /// Create a record for the given account if it does not exist yet.
    /// Does not return extra contract storage used, due to blockchain changes.
    pub fn create_if_not_exist(&mut self, account_id: AccountId) -> Result<()> {
        if !self.accounts.contains(account_id) {
            if self.accounts_keys.len() + 1 > MAX_ACCOUNTS_LEN_IN_VEC {
                return Err(AccountsSizeExceedsLimit);
            }
            let acc = Account::new();
            self.accounts.insert(account_id, &acc);
            self.accounts_keys.push(account_id);
        };

        Ok(())
    }

    pub fn balance(&self, account_id: &AccountId) -> Balance {
        match self.accounts.get(account_id) {
            None => 0,
            Some(account) => account.deposit.peek(),
        }
    }

    pub fn get(&self, account_id: &AccountId) -> Result<Account> {
        self.accounts.get(account_id).ok_or(AccountDoesNotExist)
    }

    pub fn save(&mut self, account_id: &AccountId, account: &Account) {
        self.accounts.insert(account_id, account)
    }

    /// Increase the rate of the given flow starting from the given time.
    /// Lock the payment flow from the deposit of the payer account.
    pub fn increase_flow(
        &mut self,
        start_ms: u64,
        extra_rate: Balance,
        flow: &mut Flow,
    ) -> Result<()> {
        let extra_schedule = Schedule::new(start_ms, extra_rate);
        flow.schedule.add_schedule(extra_schedule.clone());

        let mut from_account = self.get(&flow.from)?;
        from_account.lock_schedule(extra_schedule);
        self.save(&flow.from, &from_account);

        Ok(())
    }

    pub fn settle_flow(
        &mut self,
        now_ms: u64,
        flow: &mut Flow,
        curr_converter: &CurrencyConverter,
    ) -> Result<Cash> {
        let flowed_usd = flow.schedule.take_value_at_time(now_ms);
        let flowed_cere = curr_converter.to_cere(flowed_usd);
        let (payable, cash) = Cash::borrow_payable_cash(flowed_cere);

        let mut account = self.get(&flow.from)?;
        account.pay_scheduled(payable, flowed_usd)?;
        self.save(&flow.from, &account);

        Ok(cash)
    }

    pub fn flow_covered_until(
        &self,
        flow: &Flow,
        curr_converter: &CurrencyConverter,
    ) -> Result<u64> {
        let account = self.get(&flow.from)?;
        let deposit_cere = account.deposit.peek();
        let deposit_usd = curr_converter.to_usd(deposit_cere);
        Ok(account.schedule_covered_until(deposit_usd))
    }
}
