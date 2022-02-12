#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod payments {
    use scale::{Decode, Encode};

    pub type Time = u64;

    #[ink(storage)]
    pub struct Payments {}

    impl Payments {
        #[ink(constructor)]
        pub fn default() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn deposit(&mut self) -> Result<()> {
            Ok(())
        }

        #[ink(message)]
        pub fn withdraw(&mut self, amount: Balance) -> Result<()> {
            Ok(())
        }

        #[ink(message)]
        pub fn increase_flow(&mut self, from: AccountId, to: AccountId, flow: Balance) -> Result<()> {
            Ok(())
        }

        #[ink(message)]
        pub fn decrease_flow(&mut self, from: AccountId, to: AccountId, flow: Balance) -> Result<()> {
            Ok(())
        }

        #[ink(message)]
        pub fn get_flow_end_date(&mut self, from: AccountId, to: AccountId) -> Result<Time> {
            Ok(0)
        }
    }

    // ---- Utils ----
    #[derive(Debug, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        TransferFailed,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl From<Error> for ink_env::Error {
        fn from(_: Error) -> Self {
            ink_env::Error::Unknown
        }
    }
}

#[cfg(test)]
mod tests;
