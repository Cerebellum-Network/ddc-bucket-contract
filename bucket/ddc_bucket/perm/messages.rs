//! The public interface for permission management.

use ink_lang::codegen::{StaticEnv};

use crate::ddc_bucket::{AccountId, DdcBucket, Result, Error::* };
use crate::ddc_bucket::perm::entity::Permission;

impl DdcBucket {

    pub fn grant_permission(&mut self, account_id: AccountId, permission: Permission) -> Result<()> {
        self.perms.grant_permission(account_id, &permission);
        Ok(())
    }

    pub fn revoke_permission(&mut self, account_id: AccountId, permission: Permission) -> Result<()> {
        self.perms.revoke_permission(account_id, &permission);
        Ok(())
    }

    pub fn only_with_permission(&self, permission: Permission) -> Result<AccountId> {
        let caller = Self::env().caller();
        self.perms.has_permission(caller, permission)
            .then(|| caller)
            .ok_or(Unauthorized)
    }

    pub fn only_trusted_cluster_manager(&self, provider_id: AccountId) -> Result<AccountId>  {
        let caller = Self::env().caller();
        let perm = Permission::ClusterManagerTrustedBy(provider_id);
        self.perms.has_permission(caller, perm)
            .then(|| caller)
            .ok_or(OnlyTrustedClusterManager)
    }
    
}
