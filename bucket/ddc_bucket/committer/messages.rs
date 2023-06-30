use ink_lang::codegen::StaticEnv;
use ink_prelude::vec::Vec;

use crate::ddc_bucket::{AccountId, DdcBucket, CdnNodeKey, Result, Error::*};
use super::store::{Commit, EraConfig, EraStatus, EraAndTimestamp};


impl DdcBucket {

    pub fn message_set_commit(&mut self, cdn_owner: AccountId, cdn_node_key: CdnNodeKey, commit: Commit) -> Result<()> {
        self.committer.set_commit(cdn_owner, cdn_node_key, commit);
        Ok(())
    }

    pub fn message_get_commit(&self, cdn_owner: AccountId) -> Vec<(CdnNodeKey, Commit)> {
        self.committer.get_commit(cdn_owner)
    }

    pub fn message_get_validated_commit(&self, cdn_node_key: CdnNodeKey) -> EraAndTimestamp {
        self.committer.get_validate_commit(cdn_node_key)
    }

    pub fn message_set_era(&mut self, era_config: EraConfig) -> Result<()> {
        let caller = Self::env().caller();
        
        match self.committer.set_era(caller, era_config) {
            Err(_e) => Err(EraSettingFailed), 
            Ok(_v) => Ok(()),
        }
    }  
  
    pub fn message_get_era(&self) -> EraStatus {
        let timestamp = Self::env().block_timestamp();
        self.committer.get_era(timestamp)
    }

    pub fn message_get_era_settings(&self) -> EraConfig {
        self.committer.get_era_settings()
    }

}