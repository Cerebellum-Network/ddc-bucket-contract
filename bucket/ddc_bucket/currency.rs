//! The privileged interface for admin tasks.

use ink_storage::traits::{SpreadAllocate, SpreadLayout, PackedLayout};
use scale::{Decode, Encode};
use crate::ddc_bucket::{Balance, TOKEN};

pub const PRECISION: Balance = 10_000_000;

pub type CERE = Balance;
pub type USD = Balance;

#[derive(Default, Clone, PartialEq, Encode, Decode, SpreadAllocate, PackedLayout, SpreadLayout)]
#[cfg_attr(feature = "std", derive(ink_storage::traits::StorageLayout, Debug, scale_info::TypeInfo))]
pub struct CurrencyConverter { 
    /* how many USD for PRECISION CERE */
    rate: Balance
}

impl CurrencyConverter {

    pub fn new() -> Self {
        Self {
            rate: PRECISION
        }
    }

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
