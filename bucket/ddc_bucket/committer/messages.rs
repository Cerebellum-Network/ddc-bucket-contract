use ink_lang::{StaticEnv};
use ink_env::Error;
use ink_prelude::vec::Vec;

use crate::ddc_bucket::{AccountId, DdcBucket};
use super::store::{Commit, EraConfig};

pub type Result<T> = core::result::Result<T, Error>;

impl DdcBucket {
    pub fn message_get_commit(&self, node: AccountId) -> Commit {
        self.committer_store.get_commit(node)
    }

    pub fn message_set_commit(&mut self, node: AccountId, commit: Commit, logs: Vec<(AccountId, AccountId, u128, u64)>) {
        self.committer_store.set_commit(node, commit, logs);
    }

    pub fn message_set_era(&mut self, era_config: EraConfig) -> Result<()> {
        let caller = Self::env().caller();
        
        match self.committer_store.set_era(caller, era_config) {
          Err(_e) => panic!("Setting erra failed"), 
          Ok(_v) => Ok(()),
        }
    }
  
    pub fn message_get_era(&self) -> () {
        let timestamp = Self::env().block_timestamp(); 
        self.committer_store.get_era(timestamp);
    }

    pub fn message_get_era_settings(&self) -> () {
        self.committer_store.get_era_settings();
    }

    pub fn message_new_era(&mut self) -> Result<()> {
        let timestamp = Self::env().block_timestamp();
        match self.committer_store.new_era(timestamp) {
          Err(_e) => panic!("Triggering new era failed"), 
          Ok(_v) => Ok(()),
        }
    }
}