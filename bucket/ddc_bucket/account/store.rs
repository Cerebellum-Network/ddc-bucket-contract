//! The store to create and access Accounts.

use ink_storage::{collections::{HashMap, hashmap::Entry::*}, traits, Lazy};

use crate::ddc_bucket::{
    AccountId, Balance, cash::Cash, Error::*,
    Result,
    schedule::Schedule,
};
use crate::ddc_bucket::currency::CurrencyConverter;
use crate::ddc_bucket::flow::Flow;

use super::entity::Account;

#[derive(traits::SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout, Debug))]
pub struct AccountStore(
    pub HashMap<AccountId, Account>,
    pub Lazy<CurrencyConverter>,
);

impl AccountStore {
    /// Create a record for the given account if it does not exist yet.
    /// Does not return extra contract storage used, due to blockchain changes.
    pub fn create_if_not_exist(&mut self, account_id: AccountId) {
        match self.0.entry(account_id) {
            Vacant(e) => {
                e.insert(Account::new());
                ()
            }
            Occupied(_) => (),
        }
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

    /// Increase the rate of the given flow starting from the given time.
    /// Lock the payment flow from the deposit of the payer account.
    pub fn increase_flow(&mut self, start_ms: u64, extra_rate: Balance, flow: &mut Flow) -> Result<()> {
        let extra_schedule = Schedule::new(start_ms, extra_rate);
        flow.schedule.add_schedule(extra_schedule.clone());

        let from_account = self.get_mut(&flow.from)?;
        from_account.lock_schedule(extra_schedule);

        Ok(())
    }

    pub fn settle_flow(&mut self, now_ms: u64, flow: &mut Flow) -> Result<Cash> {
        let flowed_usd = flow.schedule.take_value_at_time(now_ms);
        let flowed_cere = self.1.to_cere(flowed_usd);
        let (payable, cash) = Cash::borrow_payable_cash(flowed_cere);

        let account = self.get_mut(&flow.from)?;
        account.pay_scheduled(payable, flowed_usd)?;
        Ok(cash)
    }

    pub fn flow_covered_until(&self, flow: &Flow) -> Result<u64> {
        let account = self.get(&flow.from)?;
        let deposit_cere = account.deposit.peek();
        let deposit_usd = self.1.to_usd(deposit_cere);
        Ok(account.schedule_covered_until(deposit_usd))
    }
}
