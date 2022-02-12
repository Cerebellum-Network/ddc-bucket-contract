#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod cluster {
    use ink_storage::{
        collections::HashMap,
        collections::Stash,
        traits::{PackedLayout, SpreadLayout},
    };
    use scale::{Decode, Encode};

    /// Defines the storage of your contract.
                /// Add new fields to the below struct in order
                /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct Cluster {}

    impl Cluster {
        #[ink(constructor)]
        pub fn default() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn get(&self) -> Result<()> {
            Ok(())
        }
    }

    // ---- Utils ----
    #[derive(Debug, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {}

    pub type Result<T> = core::result::Result<T, Error>;
}

#[cfg(test)]
mod tests;
