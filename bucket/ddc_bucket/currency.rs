//! The privileged interface for admin tasks.

use ink_lang::StaticEnv;
use ink_storage::Lazy;

use crate::ddc_bucket::{
    AccountId, Balance, Cash,
    DdcBucket, Error::UnauthorizedAdmin,
    Result,
};

pub type CurrencyStore = Lazy<Balance>;

impl DdcBucket {
    pub fn message_currency_get_conversion_rate(&self) -> Balance {
        *self.currency
    }

    pub fn message_currency_set_conversion_rate(&mut self, rate: Balance) {
        self.only_admin().unwrap();
        *self.currency = rate;
    }
}