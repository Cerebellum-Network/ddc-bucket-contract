//! The privileged interface for admin tasks.

use crate::ddc_bucket::perm::entity::Permission;
use crate::ddc_bucket::{AccountId, Balance, Cash, DdcBucket, NodeKey, 
    NodeOwnershipTransferred, CdnNodeOwnershipTransferred, PermissionGranted, PermissionRevoked, Result,
    Error::*
};
use ink_lang::codegen::{EmitEvent, StaticEnv};

impl DdcBucket {


    pub fn message_admin_grant_permission(
        &mut self,
        grantee: AccountId,
        permission: Permission,
    ) -> Result<()> {
        self.only_with_permission(Permission::SuperAdmin)
            .map_err(|_| OnlySuperAdmin)?;

        self.grant_permission(grantee, permission)?;
        
        Self::env().emit_event(PermissionGranted { 
            account_id: grantee,
            permission
        });

        Ok(())
    }


    pub fn message_admin_revoke_permission(
        &mut self,
        grantee: AccountId,
        permission: Permission,
    ) -> Result<()> {
        self.only_with_permission(Permission::SuperAdmin)
            .map_err(|_| OnlySuperAdmin)?;

        self.revoke_permission(grantee, permission)?;
        
        Self::env().emit_event(PermissionRevoked { 
            account_id: grantee,
            permission
        });

        Ok(())
    }


    pub fn message_admin_transfer_node_ownership(
        &mut self,
        node_key: NodeKey, 
        new_owner: AccountId
    ) -> Result<()> {
        let admin = self.only_with_permission(Permission::SuperAdmin)
            .map_err(|_| OnlySuperAdmin)?;

        let mut node = self.nodes.get(node_key)?;
        // allow node ownership transfer only if the current owner is the admin
        node.only_provider(admin).map_err(|_| NodeProviderIsNotSuperAdmin)?;

        node.provider_id = new_owner;
        self.nodes.update(node_key, &node)?;

        Self::env().emit_event(NodeOwnershipTransferred { 
            account_id: new_owner,
            node_key
        });

        Ok(())
    }


    pub fn message_admin_transfer_cdn_node_ownership(
        &mut self,
        cdn_node_key: NodeKey, 
        new_owner: AccountId
    ) -> Result<()> {
        let admin = self.only_with_permission(Permission::SuperAdmin)
            .map_err(|_| OnlySuperAdmin)?;

        let mut cdn_node = self.cdn_nodes.get(cdn_node_key)?;
        // allow node ownership transfer only if the current owner is the admin
        cdn_node.only_provider(admin).map_err(|_| CdnNodeOwnerIsNotSuperAdmin)?;

        cdn_node.provider_id = new_owner;
        self.cdn_nodes.update(cdn_node_key, &cdn_node)?;

        Self::env().emit_event(CdnNodeOwnershipTransferred { 
            account_id: new_owner,
            cdn_node_key
        });

        Ok(())
    }


    pub fn message_admin_withdraw(&mut self, amount: Balance) -> Result<()> {
        let admin = self.only_with_permission(Permission::SuperAdmin)
            .map_err(|_| OnlySuperAdmin)?;

        Self::send_cash(admin, Cash(amount))
    }

}
