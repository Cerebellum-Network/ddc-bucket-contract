//! Cash and Payable represent money and debt.
//!
//! These data structures facilitate the correctness of money-related calculations using the Rust type system.
use scale::{Decode, Encode};
use crate::ddc_nft_registry::{Balance, InsufficientBalance, Result};

// TODO: remove Clone.
/// Cash represents some value that was taken from someone, and that must be credited to someone.
#[must_use]
#[derive(Encode, Decode, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout, Debug))]
pub struct Cash(pub Balance);

/// Payable represents some value that was credited to someone, and that must be paid by someone.
/// Payable must be covered by Cash at all times to guarantee the balance of the contract.
#[must_use]
#[derive(Encode, Decode, PartialEq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout, Debug))]
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

    pub fn pay(&mut self, payable: Payable) -> Result<()> {
        if self.peek() >= payable.peek() {
            self.pay_unchecked(payable);
            Ok(())
        } else {
            Err(InsufficientBalance)
        }
    }

    pub fn pay_unchecked(&mut self, payable: Payable) {
        self.0 -= payable.consume();
    }
}

impl Payable {
    pub fn new(amount: Balance) -> Self {
        Self(amount)
    }

    #[must_use]
    pub fn consume(self) -> Balance { self.0 }

    pub fn peek(&self) -> Balance { self.0 }
}

// Implement TypeInfo with a field "value" to work with polkadot.js.
#[cfg(feature = "std")]
impl ::scale_info::TypeInfo for Cash {
    type Identity = Self;
    fn type_info() -> ::scale_info::Type {
        ::scale_info::Type::builder()
            .path(::scale_info::Path::new(
                "Cash",
                "ddc_bucket::ddc_bucket::cash",
            ))
            .type_params([])
            .composite(
                ::scale_info::build::Fields::named()
                    .field(|f| f.ty::<Balance>().name("value").type_name("Balance"))
              )
    }
}
