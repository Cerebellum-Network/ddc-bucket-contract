use crate::ddc_bucket::{AccountId, Hash, CdnNodeKey};

use ink_storage::traits::{SpreadAllocate, SpreadLayout, PackedLayout};
use ink_prelude::vec::Vec;
use ink_storage::Mapping;

#[derive(Debug, PartialEq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    /// The caller is not the authorised operator of the smart contract
    UnauthorizedOperator
}

/// Within the concept of era we would like to return specific phase to interested agents
#[derive(Debug, PartialEq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Phase {
    Commit,
    Valiadation,
    Payout
}

#[derive(Debug, PartialEq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct  EraStatus {
    current_era: u64,
    current_phase: Phase,
    previous_era: u64,
    prev_era_from_timestamp: u64,
    prev_era_to_timestamp: u64,
}

#[derive(Copy, Clone, SpreadAllocate, PackedLayout, SpreadLayout, scale::Encode, scale::Decode, Debug)]
#[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
pub struct Commit {
    hash: Hash,
    total_logs: u128,
    from_timestamp: u64,
    to_timestamp: u64,
} 

#[derive(Default, Copy, Clone, SpreadAllocate, SpreadLayout, scale::Encode, scale::Decode, Debug)]
#[cfg_attr(feature = "std", derive(::scale_info::TypeInfo, ink_storage::traits::StorageLayout))]
pub struct EraConfig {
    start: u64,
    interval: u64,
    commit_duration: u64,
    validation_duration:u64
}  

#[derive(Default, SpreadAllocate, SpreadLayout, Debug)]
#[cfg_attr(feature = "std", derive(ink_storage::traits::StorageLayout))]
pub struct CommitterStore {
    operator_id: AccountId,
    commits: Mapping<AccountId, Vec<(CdnNodeKey, Commit)>>,
    validated_commits: Mapping<CdnNodeKey, EraAndTimestamp>,
    era_settings: EraConfig
}

pub type Result<T> = core::result::Result<T, Error>;

pub type EraAndTimestamp = (u64, u64);

impl CommitterStore {
    pub fn init(&mut self, operator_id: AccountId) {
        self.operator_id = operator_id;
    }

    /// The node can set the latest commit with this function
    /// check the sender !!!!
    pub fn set_commit(&mut self, cdn_owner: AccountId, cdn_node_key: CdnNodeKey, commit: Commit) {
        if !self.commits.contains(&cdn_owner) {
            let empty_vec = Vec::<(CdnNodeKey, Commit)>::new();
            self.commits.insert(cdn_owner, &empty_vec);
        }

        let mut account_commits = self.commits.get(&cdn_owner).unwrap();
        let index = account_commits.iter().position(|(node, _)| *node == cdn_node_key).unwrap_or(usize::MAX);
        if index != usize::MAX {
            account_commits.remove(index);
        }
        account_commits.push((cdn_node_key, commit));
        self.commits.insert(&cdn_owner, &account_commits);
    }

    pub fn get_commit(&self, cdn_owner: AccountId) -> Vec<(CdnNodeKey, Commit)> {
        self.commits.get(&cdn_owner).unwrap_or(Vec::<(CdnNodeKey, Commit)>::new()).iter().cloned().collect()
    }

    // Set the last validated commit per CDN node
    pub fn set_validated_commit(&mut self, cdn_node_key: CdnNodeKey, era: u64) -> Result<()> {
        let prev_era_to_timestamp = self.era_settings.start + self.era_settings.interval * (era + 1);
        self.validated_commits.insert(&cdn_node_key, &(era, prev_era_to_timestamp));
        Ok(())
    }

    // Get the last era & timestamp validated per CDN node
    pub fn get_validate_commit(&self, cdn_node_key: CdnNodeKey) -> EraAndTimestamp {
        self.validated_commits.get(&cdn_node_key).unwrap_or((0,0))
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
    pub fn get_era(&self, timestamp: u64) -> EraStatus {
        let era_start = self.era_settings.start;
        let interval = self.era_settings.interval;
        let elapsed_time_within_interval = (timestamp - era_start) % interval;
        
        let current_phase = if elapsed_time_within_interval < self.era_settings.commit_duration {
            Phase::Commit
        } else if elapsed_time_within_interval < self.era_settings.validation_duration + self.era_settings.commit_duration {
            Phase::Valiadation
        } else {
            Phase::Payout
        };

        let current_era = (timestamp - era_start) / interval;
        let previous_era = current_era - 1;
        let prev_era_from_timestamp = era_start + interval * previous_era;
        let prev_era_to_timestamp = era_start + interval * current_era;

        EraStatus { 
            current_era, 
            current_phase, 
            previous_era, 
            prev_era_from_timestamp, 
            prev_era_to_timestamp
        }
    }

    // Get the current era phase
    pub fn get_era_settings(&self) -> EraConfig {
        return self.era_settings;
    }
}
