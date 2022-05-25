//! The Cere Name System smart contract.

#![cfg_attr(not(feature = "std"), no_std)]
#![feature(proc_macro_hygiene)] // for tests in a separate file
#![deny(unused_must_use, unused_variables)]

use ink_lang as ink;

#[ink::contract]
pub mod cns {
    use ink_prelude::string::String;
    use scale::{Decode, Encode};

    use Error::*;

    pub mod cash;
    pub mod contract_fee;
    pub mod registry;

    // ---- Global state ----
    #[ink(storage)]
    pub struct CNS {}

    impl CNS {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }
    }
    // ---- End global state ----

    // ---- Name Allocation ----

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct AllocateName {
        name: String,
        owner_id: AccountId,
    }

    impl CNS {
        #[ink(message, payable)]
        pub fn claim_name(&mut self, name: String) {
            self.message_claim_name(name).unwrap()
        }

        #[ink(message)]
        pub fn get(&self) -> Result<()> {
            Ok(())
        }
    }
    // ---- End Name Allocation ----


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
