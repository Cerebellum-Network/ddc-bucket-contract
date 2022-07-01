//! The DDC smart contract implementing bucket-based services.

#![cfg_attr(not(feature = "std"), no_std)]
#![feature(proc_macro_hygiene)] // for tests in a separate file
#![deny(unused_must_use, unused_variables)]

use ink_lang as ink;

#[ink::contract]
pub mod ddc_nft_registry {
    use ink_prelude::string::String;
    use scale::{Decode, Encode};

    use Error::*;
    use crate::ddc_nft_registry::attachment::entity::AttachmentStatus;
    use crate::ddc_nft_registry::attachment::store::AttachmentStore;

    pub mod cash;
    pub mod contract_fee;
    pub mod attachment;

    // ---- Global state ----
    #[ink(storage)]
    pub struct DdcNftRegistry {
        attachments: AttachmentStore
    }

    impl DdcNftRegistry {
        #[ink(constructor)]
        pub fn new() -> Self {
            let contract = Self {
                attachments: AttachmentStore::default(),
            };
            contract
        }
    }
    // ---- End global state ----

    // ---- Bucket ----

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct Attach {
        reporter_id: AccountId,
        nft_id: String,
        asset_id: String,
        proof: String,
    }

    impl DdcNftRegistry {
        #[ink(message, payable)]
        pub fn attach(&mut self, nft_id: String, asset_id: String, proof: String) {
            self.message_attach(nft_id, asset_id, proof).unwrap()
        }

        #[ink(message)]
        pub fn get_by_nft_id(&mut self, nft_id: String) -> AttachmentStatus {
            self.message_get_by_nft_id(nft_id).unwrap()
        }

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
        AttachmentDoesNotExist,
        UnauthorizedUpdate,
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
