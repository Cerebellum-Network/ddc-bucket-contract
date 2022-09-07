use crate::ddc_bucket::{AccountId, Hash};

use more_asserts;

use ink_storage::{
    collections::{
        HashMap as StorageHashMap
    },
    traits,
};

#[derive(Debug, PartialEq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    /// The caller is not the authorised operator of the smart contract
    UnauthorizedOperator
}

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Copy, Clone, traits::PackedLayout, traits::SpreadLayout, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
pub struct Confirmation {
    hash: Hash,
    total: u128
} 

#[derive(Copy, Clone, traits::SpreadLayout, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(::scale_info::TypeInfo, traits::StorageLayout))]
pub struct EraConfig {
    start: u64,
    interval: u64
}  

#[derive(traits::SpreadLayout)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout))]
pub struct CommitterStore {
    operator_id: AccountId,
    commits: StorageHashMap<AccountId, Confirmation>,
    era_settings: EraConfig
}

impl CommitterStore {
    pub fn new(operator_id: AccountId) -> CommitterStore {
        CommitterStore {
            operator_id,
            commits: Default::default(),
            era_settings: EraConfig {
                start: 0,
                interval: 0
            }
        }
    }

    pub fn get_commit(&self, node: AccountId) -> Confirmation {
        *self.commits.get(&node).unwrap()
    }

    /// The node can set the latest commit with this function
    /// check the sender !!!!
    pub fn set_commit(&mut self, node: AccountId, confirmation: Confirmation) {
        self.commits.insert(node, confirmation);
    }

    // Akin to modifier
    pub fn only_owner(&self, operator_id: AccountId) -> Result<()> {
        if self.operator_id == operator_id { Ok(()) } else { Err(Error::UnauthorizedOperator) }
    }

    // Set the new value for the era config
    pub fn set_era(&mut self, caller: AccountId, era_config: EraConfig) -> Result<()> {
        self.only_owner(caller)?;
        self.era_settings = era_config;
        Ok(())
    }

    // Get the current era phase. 0 represents commit phase, 1 validation phase, 2 payout phase
    pub fn get_era(&self, timestamp: u64) -> u64 {
        let era_start = self.era_settings.start;
        let interval = self.era_settings.interval;

        if timestamp < (era_start + interval / 3) {
            return 0;
        } else if timestamp < (era_start + interval * 2 / 3) {
            return 1;
        } else {
            return 2;
        }
    }

    // Get the current era phase
    pub fn get_era_settings(&self) -> EraConfig {
        return self.era_settings;
    }

    // Move to next era
    pub fn new_era(&mut self, timestamp: u64) -> Result<()> {
        more_asserts::assert_ge!(timestamp, self.era_settings.start + self.era_settings.interval);
        self.era_settings.start = self.era_settings.start + self.era_settings.interval;
        Ok(())
    }
}
