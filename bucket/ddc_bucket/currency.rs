//! The privileged interface for admin tasks.

use ink_storage::traits::{SpreadAllocate, SpreadLayout};
use ink_storage::traits::KeyPtr;

use crate::ddc_bucket::{Balance, TOKEN};

pub type CERE = Balance;
pub type USD = Balance;

const PRECISION: Balance = 10_000_000; 

#[derive(SpreadLayout)]
#[cfg_attr(feature = "std", derive(ink_storage::traits::StorageLayout, Debug))]
pub struct CurrencyConverter { 
    /* how many USD for PRECISION CERE */
    rate: Balance
}


impl SpreadAllocate for CurrencyConverter { 
    fn allocate_spread(_: &mut KeyPtr) -> Self { 
        Self {
            rate: PRECISION
        }
    }
}

impl Default for CurrencyConverter {
    fn default() -> Self {
        Self {
            rate: PRECISION
        }
    }
}

impl CurrencyConverter { // 10_000_000
    pub fn set_usd_per_cere(&mut self, usd_per_cere: USD) {
        self.rate = usd_per_cere * PRECISION / TOKEN;
    }

    pub fn to_cere(&self, usd: USD) -> CERE {
        usd * PRECISION / self.rate
    }

    pub fn to_usd(&self, cere: CERE) -> USD {
        self.rate * cere / PRECISION
    }
}
