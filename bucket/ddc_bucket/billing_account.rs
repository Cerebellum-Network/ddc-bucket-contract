use ink_prelude::{
    string::String,
};
use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};

use crate::ddc_bucket::{AccountId, Balance, Error::*, Result};

use super::cash::{Cash, Payable};
use super::schedule::Schedule;

#[derive(PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct BillingAccount {
    pub deposit: Cash,
    pub payable_locked: Balance,
    pub payable_schedule: Schedule,
}

impl BillingAccount {
    pub fn new() -> BillingAccount {
        BillingAccount {
            deposit: Cash(0),
            payable_locked: 0,
            payable_schedule: Schedule::empty(),
        }
    }

    pub fn deposit(&mut self, cash: Cash) {
        self.deposit.increase(cash);
    }

    pub fn withdraw(&mut self, time_ms: u64, payable: Payable) -> Result<()> {
        if self.get_withdrawable(time_ms) >= payable.peek() {
            self.deposit.pay_unchecked(payable);
            Ok(())
        } else {
            Err(InsufficientBalance)
        }
    }

    pub fn get_withdrawable(&self, time_ms: u64) -> Balance {
        let deposit = self.deposit.peek();
        let consumed = self.payable_locked + self.payable_schedule.value_at_time(time_ms);
        if deposit >= consumed {
            deposit - consumed
        } else {
            0
        }
    }

    pub fn lock_schedule(&mut self, payable_schedule: Schedule) {
        self.payable_locked += self.payable_schedule.take_value_then_add_rate(payable_schedule);
    }

    pub fn schedule_covered_until(&self) -> u64 {
        self.payable_schedule.time_of_value(self.deposit.peek())
    }

    pub fn pay_scheduled(&mut self, now_ms: u64, payable: Payable) -> Result<()> {
        self.unlock_scheduled_amount(now_ms, payable.peek());
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

    fn unlock_scheduled_amount(&mut self, now_ms: u64, unlocked: Balance) {
        self.payable_locked = self.payable_locked
            + self.payable_schedule.take_value_at_time(now_ms)
            - unlocked;
    }
}