//! The store to create and access Accounts.

use ink_storage::{
    collections::{HashMap, hashmap::Entry::*},
    traits,
};

use crate::ddc_bucket::{
    AccountId, Balance, cash::Cash, Error::*,
    Result,
    schedule::Schedule,
};
use crate::ddc_bucket::flow::Flow;

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

    /// Create a record for the given account if it does not exist yet.
    /// Return the extra contract storage used.
    #[must_use]
    pub fn create_if_not_exist(&mut self, account_id: AccountId) -> usize {
        match self.0.entry(account_id) {
            Vacant(e) => {
                e.insert(Account::new());
                Account::RECORD_SIZE
            }
            Occupied(_) => 0,
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

    pub fn start_flow(&mut self, start_ms: u64, from: AccountId, rate: Balance) -> Result<Flow> {
        let cash_schedule = Schedule::new(start_ms, rate);
        let payable_schedule = cash_schedule.clone();

        let from_account = self.get_mut(&from)?;
        from_account.lock_schedule(payable_schedule);

        let flow = Flow {
            from,
            schedule: cash_schedule,
        };
        Ok(flow)
    }

    pub fn settle_flow(&mut self, now_ms: u64, flow: &mut Flow) -> Result<Cash> {
        let flowed_amount = flow.schedule.take_value_at_time(now_ms);
        let (payable, cash) = Cash::borrow_payable_cash(flowed_amount);

        let account = self.get_mut(&flow.from)?;
        account.pay_scheduled(now_ms, payable)?;
        Ok(cash)
    }

    pub fn flow_covered_until(&self, flow: &Flow) -> Result<u64> {
        let account = self.get(&flow.from)?;
        Ok(account.schedule_covered_until())
    }
}
