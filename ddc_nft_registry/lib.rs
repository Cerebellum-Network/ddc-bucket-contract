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
    use ink_storage::traits::SpreadAllocate;

    pub mod cash;
    pub mod contract_fee;
    pub mod attachment;

    // ---- Global state ----
    #[ink(storage)]
    #[derive(Default, SpreadAllocate)]
    pub struct DdcNftRegistry {
        attachments: AttachmentStore,
    }

    impl DdcNftRegistry {
        #[ink(constructor)]
        pub fn new() -> Self {
            ink_lang::utils::initialize_contract(|_| {})
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
        /// Report and attach an asset ID to an NFT ID.
        ///
        /// All attachments are recorded as events.
        /// There is absolutely no validation, any account can "attach" some asset ID.
        /// Events should be filtered by reporter_id, or by analyzing the proof (not specified here).
        ///
        /// The latest attachment is also recorded in contract storage.
        /// The latest asset ID can be queried from get_by_nft_id.
        /// The first reporter for an NFT ID can also update the asset ID.
        #[ink(message, payable)]
        pub fn attach(&mut self, nft_id: String, asset_id: String, proof: String) {
            self.message_attach(nft_id, asset_id, proof).unwrap()
        }

        /// Report the attachment of an asset ID to an NFT ID.
        ///
        /// This is recorded only as a contract event.
        /// This can *not* be queried from get_by_nft_id.
        ///
        /// There is absolutely no validation, any account can "report" some asset ID.
        /// Events should be filtered by reporter_id, or by analyzing the proof (not specified here).
        #[ink(message, payable)]
        pub fn report(&mut self, nft_id: String, asset_id: String, proof: String) {
            self.message_report(nft_id, asset_id, proof).unwrap()
        }

        #[ink(message)]
        pub fn get_by_nft_id(&mut self, nft_id: String) -> AttachmentStatus {
            self.message_get_by_nft_id(nft_id).unwrap()
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
