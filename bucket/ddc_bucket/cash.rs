use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};

use crate::ddc_bucket::Balance;

/// Cash represents some value that was taken from someone, and that must be credited to someone.
#[must_use]
#[derive(PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct Cash(pub Balance);

/// Payable represents some value that was credited to someone, and that must be paid by someone.
/// Payable must be covered by Cash at all times to guarantee the balance of the contract.
#[must_use]
#[derive(PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct Payable(pub Balance);

impl Cash {
    pub fn borrow_payable_cash(amount: Balance) -> (Payable, Cash) {
        (Payable(amount), Cash(amount))
    }

    #[must_use]
    pub fn consume(self) -> Balance { self.0 }

    pub fn peek(&self) -> Balance { self.0 }

    pub fn increase(&mut self, cash: Cash) {
        self.0 += cash.consume();
    }

    pub fn pay_unchecked(&mut self, payable: Payable) {
        self.0 -= payable.consume();
    }
}

impl Payable {
    #[must_use]
    pub fn consume(self) -> Balance { self.0 }

    pub fn peek(&self) -> Balance { self.0 }
}