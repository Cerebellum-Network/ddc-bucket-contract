#![cfg_attr(not(feature = "std"), no_std)]
#![feature(proc_macro_hygiene)] // for tests in a separate file

use ink_lang as ink;

pub use self::payments::{
    Payments,
    PaymentsRef,
};

#[ink::contract]
mod payments {
    use ink_storage::{
        collections::Stash,
        traits::{PackedLayout, SpreadLayout},
    };
    use scale::{Decode, Encode};

    pub type TabId = u32;

    #[derive(Copy, Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
    pub struct Tab {}

    #[derive(Copy, Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
    pub struct TabStatus {}

    #[ink(storage)]
    pub struct Payments {
        tabs: Stash<Tab>,
    }

    impl Payments {
        #[ink(constructor)]
        pub fn default() -> Self {
            Self { tabs: Default::default() }
        }

        #[ink(message)]
        pub fn deposit(&mut self) -> Result<()> {
            Ok(())
        }

        #[ink(message)]
        pub fn withdraw(&mut self, _amount: Balance) -> Result<()> {
            Ok(())
        }

        #[ink(message)]
        pub fn create_tab(&mut self, _to: AccountId) -> Result<TabId> {
            let tab = Tab {};
            let tab_id = self.tabs.put(tab);
            Ok(tab_id)
        }

        #[ink(message)]
        pub fn increase_flow(&mut self, _from: AccountId, _into: TabId, _flow: Balance) -> Result<()> {
            Ok(())
        }

        #[ink(message)]
        pub fn decrease_flow(&mut self, _from: AccountId, _into: TabId, _flow: Balance) -> Result<()> {
            Ok(())
        }

        #[ink(message)]
        pub fn get_tab_status(&mut self, _tab_id: TabId) -> Result<TabStatus> {
            Ok(TabStatus {})
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
