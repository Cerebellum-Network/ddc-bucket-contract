//! The DDC smart contract implementing bucket-based services.

#![cfg_attr(not(feature = "std"), no_std)]
#![feature(proc_macro_hygiene)] // for tests in a separate file
#![deny(unused_must_use, unused_variables)]

use ink_lang as ink;

#[ink::contract]
pub mod ddc_nft_registry {
    use scale::{Decode, Encode};

    use Error::*;

    pub mod cash;
    pub mod contract_fee;

    // ---- Global state ----
    #[ink(storage)]
    pub struct DdcNftRegistry {}

    impl DdcNftRegistry {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }
    }
    // ---- End global state ----

    // ---- Bucket ----

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct Attach {
        #[ink(topic)]
        account_id: AccountId,
    }

    impl DdcNftRegistry {
        #[ink(message, payable)]
        pub fn attach(&mut self) {}

        #[ink(message)]
        pub fn get(&self) -> Result<()> {
            Ok(())
        }
    }
    // ---- End Bucket ----


    // ---- Utils ----
    /// One token with 10 decimals.
    pub const TOKEN: Balance = 10_000_000_000;

    #[derive(Debug, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InsufficientBalance,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl From<Error> for ink_env::Error {
        fn from(_: Error) -> Self {
            ink_env::Error::Unknown
        }
    }
    // ---- End Utils ----

    #[cfg(test)]
    mod tests;
}
