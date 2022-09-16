use ink_lang::{StaticEnv};
use ink_env::Error;

use crate::ddc_bucket::{AccountId, DdcBucket};
use super::store::{Confirmation, EraConfig, Settlement};

pub type Result<T> = core::result::Result<T, Error>;

impl DdcBucket {
    pub fn message_get_commit(&self, node: AccountId) -> Confirmation {
        self.committer_store.get_commit(node)
    }

    pub fn message_set_commit(&mut self, node: AccountId, confirmation: Confirmation) {
        self.committer_store.set_commit(node, confirmation);
    }

    pub fn message_get_settlement(&self, node: AccountId) -> Settlement {
        self.committer_store.get_settlement(node)
    }

    pub fn message_validate_settlement(&mut self, node: AccountId, settlement: Settlement) -> Result<()> {
        let caller = Self::env().caller();

        match self.committer_store.validate_settlement(caller, node, settlement) {
            Err(_e) => panic!("Settlement validation failed"), 
            Ok(_v) => Ok(()),
        }
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

    pub fn message_get_era_settings(&self) -> EraConfig {
        self.committer_store.get_era_settings()
    }

    pub fn message_new_era(&mut self) -> Result<()> {
        let timestamp = Self::env().block_timestamp();
        match self.committer_store.new_era(timestamp) {
          Err(_e) => panic!("Triggering new era failed"), 
          Ok(_v) => Ok(()),
        }
    }
}