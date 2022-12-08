use ink_lang::{StaticEnv};
use ink_env::Error;
use ink_prelude::vec::Vec;

use crate::ddc_bucket::{AccountId, DdcBucket, NodeId};
use super::store::{Commit, EraConfig, EraStatus, EraAndTimestamp};

pub type Result<T> = core::result::Result<T, Error>;

impl DdcBucket {
    pub fn message_set_commit(&mut self, cdn_owner: AccountId, node_id: NodeId, commit: Commit) {
        self.committer_store.set_commit(cdn_owner, node_id, commit);
    }

    pub fn message_get_commit(&self, cdn_owner: AccountId) -> Vec<(NodeId, Commit)> {
        self.committer_store.get_commit(cdn_owner)
    }

    pub fn message_set_validated_commit(&mut self, node: AccountId, era: u64) -> Result<()> {
        match self.committer_store.set_validated_commit(node, era) {
            Err(_e) => panic!("Setting validated commit failed"), 
            Ok(_v) => Ok(()),
        }
    }

    pub fn message_get_validated_commit(&self, node: AccountId) -> EraAndTimestamp {
        self.committer_store.get_validate_commit(node)
    }

    pub fn message_set_era(&mut self, era_config: EraConfig) -> Result<()> {
        let caller = Self::env().caller();
        
        match self.committer_store.set_era(caller, era_config) {
            Err(_e) => panic!("Setting era failed"), 
            Ok(_v) => Ok(()),
        }
    }  
  
    pub fn message_get_era(&self) -> EraStatus {
        let timestamp = Self::env().block_timestamp();
        self.committer_store.get_era(timestamp)
    }

    pub fn message_get_era_settings(&self) -> EraConfig {
        self.committer_store.get_era_settings()
    }
}