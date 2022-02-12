#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod payments {
    #[ink(storage)]
    pub struct Payments {}

    impl Payments {
        #[ink(constructor)]
        pub fn default() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn get(&self) -> bool {
            true
        }
    }
}

#[cfg(test)]
mod tests;
