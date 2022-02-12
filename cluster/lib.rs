#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod cluster {
    use ink_storage::{
        collections::Stash,
        traits::{PackedLayout, SpreadLayout},
    };
    use scale::{Decode, Encode};

    pub type ResourceId = u32;

    #[derive(Copy, Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
    pub struct Resource {}

    #[ink(storage)]
    pub struct Cluster {
        price: Balance,
        location: String,
        resources: Stash<Resource>,
    }

    impl Cluster {
        // ---- Owner Interface ----
        #[ink(constructor)]
        pub fn default() -> Self {
            Self {
                price: 0,
                location: "".to_string(),
                resources: Default::default(),
            }
        }

        #[ink(message)]
        pub fn set_location(&mut self, location: String) -> Result<()> {
            self.location = location;
            Ok(())
        }

        #[ink(message)]
        pub fn set_price(&mut self, price: Balance) -> Result<()> {
            self.price = price;
            Ok(())
        }

        // ---- Registry Interface ----

        #[ink(message)]
        pub fn get_price(&self) -> Result<Balance> { Ok(self.price) }

        #[ink(message)]
        pub fn create_resource(&mut self) -> Result<ResourceId> {
            let resource = Resource {};
            let resource_id = self.resources.put(resource);
            Ok(resource_id)
        }

        // ---- Node Interface ----
        #[ink(message)]
        pub fn is_accepted(&self, _resource_id: ResourceId) -> Result<bool> {
            Ok(true)
        }

        // ---- App Interface ----
        #[ink(message)]
        pub fn get_location(&self) -> Result<String> {
            Ok(self.location.clone())
        }
    }

    // ---- Utils ----
    #[derive(Debug, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        ResourceDoesNotExist,
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
