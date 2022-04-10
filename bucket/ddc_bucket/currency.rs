//! The privileged interface for admin tasks.

use ink_lang::StaticEnv;
use ink_storage::{Lazy, traits};

use crate::ddc_bucket::{AccountId, Balance, Cash, DdcBucket, Error::UnauthorizedAdmin, Result, TOKEN};

pub type CERE = Balance;
pub type USD = Balance;


#[derive(traits::SpreadLayout)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout, Debug))]
pub struct CurrencyConverter(Balance /* how many USD for PRECISION CERE */);

const PRECISION: Balance = 10_000;

impl Default for CurrencyConverter {
    fn default() -> Self {
        Self(PRECISION)
    }
}

impl CurrencyConverter {
    pub fn set_usd_per_cere(&mut self, usd_per_cere: USD) {
        self.0 = usd_per_cere * PRECISION / TOKEN;
    }

    pub fn to_cere(&self, usd: USD) -> CERE {
        usd * PRECISION / self.0
    }

    pub fn to_usd(&self, cere: CERE) -> USD {
        self.0 * cere / PRECISION
    }
}