//! The smart contract of the Cere Name System.

#![cfg_attr(not(feature = "std"), no_std)]
#![feature(proc_macro_hygiene)] // for tests in a separate file
#![deny(unused_must_use, unused_variables)]

use ink_lang as ink;

#[ink::contract]
pub mod cns {
    use ink_prelude::string::String;
    use scale::{Decode, Encode};

    use Error::*;
    use names::{entity::*, store::*};

    pub mod names;
    pub mod cash;

    // ---- Global state ----
    #[ink(storage)]
    pub struct CNS {
        name_store: NameStore,
    }

    impl CNS {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                name_store: Default::default(),
            }
        }
    }
    // ---- End global state ----

    // ---- Name Allocation ----

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct AllocateName {
        #[ink(topic)]
        name: String,
        #[ink(topic)]
        owner_id: AccountId,
    }

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct SetPayload {
        #[ink(topic)]
        name: String,
        payload: String,
    }

    impl CNS {
        #[ink(message, payable)]
        pub fn claim_name(&mut self, name: String) {
            self.message_claim_name(name).unwrap()
        }

        #[ink(message)]
        pub fn set_payload(&mut self, name: String, payload: String) {
            self.message_set_payload(name, payload).unwrap()
        }

        #[ink(message)]
        pub fn get_by_name(&self, name: String) -> Result<Record> {
            Ok(self.name_store.get(&name)?.clone())
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
        NameDoesNotExist,
        NameAlreadyTaken,
        NameTooLong,
        NameMustStartWithALetter,
        PayloadTooLong,
        Unauthorized,
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
