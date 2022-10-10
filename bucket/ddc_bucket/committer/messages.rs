use ink_lang::{StaticEnv};
use ink_env::Error;

use crate::ddc_bucket::{AccountId, DdcBucket};
use super::store::{Commit, EraConfig};

pub type Result<T> = core::result::Result<T, Error>;

impl DdcBucket {
    pub fn message_get_commit(&self, node: AccountId) -> Commit {
        self.committer_store.get_commit(node)
    }

    pub fn message_set_commit(&mut self, node: AccountId, commit: Commit) {
        self.committer_store.set_commit(node, commit);
    }

    pub fn message_set_era(&mut self, era_config: EraConfig) -> Result<()> {
        let caller = Self::env().caller();
        
        match self.committer_store.set_era(caller, era_config) {
          Err(_e) => panic!("Setting erra failed"), 
          Ok(_v) => Ok(()),
        }
    }
  
    pub fn message_get_era(&self) -> u64 {
        let timestamp = Self::env().block_timestamp();
        self.committer_store.get_era(timestamp)
    }

    pub fn message_get_era_settings(&self) -> () {
        self.committer_store.get_era_settings();
    }
}