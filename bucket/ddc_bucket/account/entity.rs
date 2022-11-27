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
    pub bonded: Cash,
    pub negative: Cash,
    pub unbonded_amount: Cash,
    pub unbonded_timestamp: u64,
    pub payable_schedule: Schedule,
}

impl Account {
    pub const RECORD_SIZE: usize =
        SIZE_PER_RECORD + SIZE_HASHMAP + SIZE_ACCOUNT_ID
            + SIZE_BALANCE + Schedule::RECORD_SIZE;

    pub fn new() -> Account {
        Account {
            deposit: Cash(0),
            bonded: Cash(0),
            negative: Cash(0),
            unbonded_amount: Cash(0),
            unbonded_timestamp: 0,
            payable_schedule: Schedule::empty(),
        }
    }

    pub fn deposit(&mut self, cash: Cash) {
        self.deposit.increase(cash);
    }

    pub fn bond(&mut self, time_ms: u64, conv: &CurrencyConverter, bond_amount: Balance) -> Result<()> {
        let payable = Payable(bond_amount);
        if self.get_withdrawable(time_ms, conv) >= payable.peek() {
            let parsed_payable: u128;
            if self.negative.peek() > 0  && payable.peek() >= self.negative.peek() {
                parsed_payable = payable.peek() - self.negative.peek();
                self.deposit.pay_unchecked(payable);
                self.bonded.increase(Cash(parsed_payable));
                Ok(())
            } else if self.negative.peek() > 0 && payable.peek() < self.negative.peek(){
                Err(InsufficientBalance)
            } else {
                let bonded_amount = Cash(payable.peek());
                self.deposit.pay_unchecked(payable);
                self.bonded.increase(bonded_amount);
                Ok(())
            }
        } else {
            Err(InsufficientBalance)
        }
    }

    pub fn unbond(&mut self, amount_to_unbond: Cash, timestamp: u64) -> Result<()> {
        let remaining_bonded = self.bonded.peek() - self.unbonded_amount.peek();
        if remaining_bonded >= amount_to_unbond.peek() {
            self.bonded.pay_unchecked(Payable(amount_to_unbond.peek()));
            self.unbonded_amount.increase(amount_to_unbond);
            self.unbonded_timestamp = timestamp + MS_PER_WEEK;
            Ok(())
        } else {
            Err(InsufficientBalance)
        }
    }

    pub fn withdraw(&mut self, time_ms: u64, conv: &CurrencyConverter, payable: Payable) -> Result<()> {
        if self.get_withdrawable(time_ms, conv) >= payable.peek() {
            self.deposit.pay_unchecked(payable);
            Ok(())
        } else {
            Err(InsufficientBalance)
        }
    }

    // Add logics when balance is below requested 
    pub fn withdraw_bonded(&mut self, payable: Payable) -> Result<()> {
        let remaining_bonded = self.bonded.peek() - self.unbonded_amount.peek();
        if remaining_bonded >= payable.peek() {
            self.bonded.pay_unchecked(payable);
            Ok(())
        } else {
            let negative_balance = payable.peek() - remaining_bonded;
            self.bonded.pay_unchecked(payable);
            self.negative.increase(Cash(negative_balance));
            Ok(())
        }
    }

    pub fn withdraw_unbonded(&mut self, timestamp: u64) -> Result<()> {
        if timestamp >= self.unbonded_timestamp {
            self.deposit.increase(self.unbonded_amount);
            self.unbonded_amount = Cash(0);
            self.unbonded_timestamp = 0;
            Ok(())
        } else {
            Err(BondingPeriodNotFinished)
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

pub const MS_PER_WEEK: u64 = 7 * 24 * 3600 * 1000;