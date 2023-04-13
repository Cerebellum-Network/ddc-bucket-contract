//! The store to create and access Accounts.

use ink_storage::Mapping;

use crate::ddc_bucket::{
    AccountId, Balance, cash::Cash, Error::*,
    Result,
    schedule::Schedule,
};
use crate::ddc_bucket::currency::CurrencyConverter;
use crate::ddc_bucket::flow::Flow;
use ink_prelude::vec::Vec;
use super::entity::Account;


pub const ACCOUNT_STORE_KEY: u32 = openbrush::storage_unique_key!(AccountStore);
#[openbrush::upgradeable_storage(ACCOUNT_STORE_KEY)]
#[derive(Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct AccountStore {
    pub accounts: Mapping<AccountId, Account>,
    pub curr_converter: CurrencyConverter,
    // todo: remove this vector as it can store an arbitrary number of elements and easily exceed 16KB limit
    pub account_ids: Vec<AccountId>,
    _reserved: Option<()>
}

impl AccountStore {
    /// Create a record for the given account if it does not exist yet.
    /// Does not return extra contract storage used, due to blockchain changes.
    pub fn create_if_not_exist(&mut self, account_id: AccountId) {
        if !self.accounts.contains(account_id) {
            let acc = Account::new();
            self.accounts.insert(account_id, &acc);
            self.account_ids.push(account_id);
        };
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
        self.accounts.insert(account_id, account);
    }

    /// Increase the rate of the given flow starting from the given time.
    /// Lock the payment flow from the deposit of the payer account.
    pub fn increase_flow(&mut self, start_ms: u64, extra_rate: Balance, flow: &mut Flow) -> Result<()> {
        let extra_schedule = Schedule::new(start_ms, extra_rate);
        flow.schedule.add_schedule(extra_schedule.clone());

        let mut from_account = self.get(&flow.from)?;
        from_account.lock_schedule(extra_schedule);
        self.save(&flow.from, &from_account);

        Ok(())
    }

    pub fn settle_flow(&mut self, now_ms: u64, flow: &mut Flow) -> Result<Cash> {
        let flowed_usd = flow.schedule.take_value_at_time(now_ms);
        let flowed_cere = self.curr_converter.to_cere(flowed_usd);
        let (payable, cash) = Cash::borrow_payable_cash(flowed_cere);

        let mut account = self.get(&flow.from)?;
        account.pay_scheduled(payable, flowed_usd)?;
        self.save(&flow.from, &account);

        Ok(cash)
    }

    pub fn flow_covered_until(&self, flow: &Flow) -> Result<u64> {
        let account = self.get(&flow.from)?;
        let deposit_cere = account.deposit.peek();
        let deposit_usd = self.curr_converter.to_usd(deposit_cere);
        Ok(account.schedule_covered_until(deposit_usd))
    }
}
