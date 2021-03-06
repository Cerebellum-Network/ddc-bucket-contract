//! The data structure of Accounts.

use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};

use crate::ddc_bucket::{
    Balance, cash::{Cash, Payable},
    Error::*, Result,
    schedule::Schedule,
};
use crate::ddc_bucket::contract_fee::{SIZE_ACCOUNT_ID, SIZE_BALANCE, SIZE_HASHMAP, SIZE_PER_RECORD};
use crate::ddc_bucket::currency::{USD, CurrencyConverter};

#[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct Account {
    pub deposit: Cash,
    pub payable_schedule: Schedule,
}

impl Account {
    pub const RECORD_SIZE: usize =
        SIZE_PER_RECORD + SIZE_HASHMAP + SIZE_ACCOUNT_ID
            + SIZE_BALANCE + Schedule::RECORD_SIZE;

    pub fn new() -> Account {
        Account {
            deposit: Cash(0),
            payable_schedule: Schedule::empty(),
        }
    }

    pub fn deposit(&mut self, cash: Cash) {
        self.deposit.increase(cash);
    }

    pub fn withdraw(&mut self, time_ms: u64, conv: &CurrencyConverter, payable: Payable) -> Result<()> {
        if self.get_withdrawable(time_ms, conv) >= payable.peek() {
            self.deposit.pay_unchecked(payable);
            Ok(())
        } else {
            Err(InsufficientBalance)
        }
    }

    pub fn get_withdrawable(&self, time_ms: u64, conv: &CurrencyConverter) -> Balance {
        let deposit = self.deposit.peek();
        let consumed_usd = self.payable_schedule.value_at_time(time_ms);
        let consumed = conv.to_cere(consumed_usd);
        if deposit >= consumed {
            deposit - consumed
        } else {
            0
        }
    }

    pub fn lock_schedule(&mut self, payable_schedule: Schedule) {
        self.payable_schedule.add_schedule(payable_schedule);
    }

    pub fn schedule_covered_until(&self, deposit_usd: USD) -> u64 {
        self.payable_schedule.time_of_value(deposit_usd)
    }

    pub fn pay_scheduled(&mut self, payable: Payable, payable_usd: USD) -> Result<()> {
        self.unlock_scheduled_amount(payable_usd);
        self.pay(payable)
    }

    fn pay(&mut self, payable: Payable) -> Result<()> {
        if self.deposit.peek() >= payable.peek() {
            self.deposit.pay_unchecked(payable);
            Ok(())
        } else {
            Err(InsufficientBalance)
        }
    }

    fn unlock_scheduled_amount(&mut self, unlocked: Balance) {
        self.payable_schedule.take_value(unlocked);
    }
}