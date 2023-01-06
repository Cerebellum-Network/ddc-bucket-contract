//! The Schedule data structure implements a value that increases over time.

use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};

use crate::ddc_bucket::Balance;

#[must_use]
#[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct Schedule {
    rate: Balance,
    offset: Balance,
}

impl Schedule {
    pub fn new(start_ms: u64, rate: Balance) -> Schedule {
        let offset = rate * start_ms as Balance / MS_PER_MONTH;
        Schedule { rate, offset }
    }

    pub fn empty() -> Schedule { Schedule::new(0, 0) }

    pub fn value_at_time(&self, time_ms: u64) -> Balance {
        let absolute = self.rate * time_ms as Balance / MS_PER_MONTH;
        assert!(absolute >= self.offset);
        absolute - self.offset
    }

    pub fn time_of_value(&self, value: Balance) -> u64 {
        if self.rate == 0 { return u64::MAX; }

        let absolute = self.offset + value;
        let time = absolute * MS_PER_MONTH / self.rate;
        time as u64
    }

    pub fn add_schedule(&mut self, to_add: Schedule) {
        self.offset += to_add.offset;
        self.rate += to_add.rate;
    }

    pub fn take_value(&mut self, value: Balance) {
        self.offset += value;
    }

    #[must_use]
    pub fn take_value_at_time(&mut self, now_ms: u64) -> Balance {
        let value = self.value_at_time(now_ms);
        self.offset += value;
        value
    }
}

pub const MS_PER_MONTH: u128 = 31 * 24 * 3600 * 1000;