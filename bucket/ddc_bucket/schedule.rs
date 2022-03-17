use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};

use crate::ddc_bucket::Balance;
use crate::ddc_bucket::contract_fee::SIZE_BALANCE;

#[must_use]
#[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct Schedule {
    rate: Balance,
    start_ms: u64,
}

impl Schedule {
    pub const RECORD_SIZE: usize = SIZE_BALANCE + 8;

    pub fn new(start_ms: u64, rate: Balance) -> Schedule {
        Schedule { rate, start_ms }
    }

    pub fn empty() -> Schedule { Schedule::new(0, 0) }

    pub fn value_at_time(&self, time_ms: u64) -> Balance {
        assert!(time_ms >= self.start_ms);
        let period_ms = (time_ms - self.start_ms) as u128;
        period_ms * self.rate / MS_PER_MONTH
    }

    pub fn time_of_value(&self, value: Balance) -> u64 {
        if self.rate == 0 { return u64::MAX; }
        let duration_ms = value * MS_PER_MONTH / self.rate;
        self.start_ms + duration_ms as u64
    }

    #[must_use]
    pub fn take_value_at_time(&mut self, now_ms: u64) -> Balance {
        let value = self.value_at_time(now_ms);
        self.start_ms = now_ms;
        value
    }

    #[must_use]
    pub fn take_value_then_add_rate(&mut self, to_add: Schedule) -> Balance {
        let accumulated = self.take_value_at_time(to_add.start_ms);
        self.rate += to_add.rate;
        accumulated
    }
}

pub const MS_PER_MONTH: u128 = 31 * 24 * 3600 * 1000;