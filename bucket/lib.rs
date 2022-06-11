//! The DDC smart contract implementing bucket-based services.

#![cfg_attr(not(feature = "std"), no_std)]
#![feature(proc_macro_hygiene)] // for tests in a separate file
#![deny(unused_must_use, unused_variables)]

use ink_lang as ink;

#[ink::contract]
pub mod ddc_bucket {
    use ink_prelude::vec::Vec;
    use scale::{Decode, Encode};

    use account::store::*;
    use bucket::{entity::*, store::*};
    use cash::*;
    use cluster::{entity::*, store::*};
    use Error::*;
    use node::{entity::*, store::*};
    use params::{store::*};
    use perm::{store::*};

    use crate::ddc_bucket::account::entity::Account;
    use crate::ddc_bucket::perm::entity::Permission;

    pub mod account;
    pub mod flow;
    pub mod schedule;
    pub mod cash;
    pub mod node;
    pub mod bucket;
    pub mod cluster;
    pub mod contract_fee;
    pub mod params;
    pub mod admin;
    pub mod perm;
    pub mod currency;

    // ---- Global state ----
    #[ink(storage)]
    pub struct DdcBucket {
        buckets: BucketStore,
        bucket_params: ParamsStore,
        clusters: ClusterStore,
        cluster_params: ParamsStore,
        nodes: NodeStore,
        node_params: ParamsStore,
        accounts: AccountStore,
        perms: PermStore,
    }

    impl DdcBucket {
        #[ink(constructor)]
        pub fn new() -> Self {
            let mut contract = Self {
                buckets: BucketStore::default(),
                bucket_params: ParamsStore::default(),
                clusters: ClusterStore::default(),
                cluster_params: ParamsStore::default(),
                nodes: NodeStore::default(),
                node_params: ParamsStore::default(),
                accounts: AccountStore::default(),
                perms: PermStore::default(),
            };

            // Make the creator of this contract a super-admin.
            let admin_id = Self::env().caller();
            contract.perms.grant_permission(admin_id, &Permission::SuperAdmin);

            // Reserve IDs 0.
            let _ = contract.accounts.create_if_not_exist(AccountId::default());
            let _ = contract.clusters.create(AccountId::default(), 0, &[]).unwrap();
            let _ = contract.cluster_params.create("".to_string());
            let _ = contract.buckets.create(AccountId::default(), 0);
            let _ = contract.bucket_params.create("".to_string());

            contract
        }
    }
    // ---- End global state ----

    // ---- Bucket ----

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct BucketCreated {
        #[ink(topic)]
        bucket_id: BucketId,
        #[ink(topic)]
        owner_id: AccountId,
    }

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct BucketAllocated {
        #[ink(topic)]
        bucket_id: BucketId,
        #[ink(topic)]
        cluster_id: ClusterId,
        resource: Resource,
    }

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct BucketSettlePayment {
        #[ink(topic)]
        bucket_id: BucketId,
        #[ink(topic)]
        cluster_id: ClusterId,
    }

    impl DdcBucket {
        #[ink(message, payable)]
        pub fn bucket_create(&mut self, bucket_params: BucketParams, cluster_id: ClusterId) -> BucketId {
            self.message_bucket_create(bucket_params, cluster_id).unwrap()
        }

        #[ink(message)]
        pub fn bucket_alloc_into_cluster(&mut self, bucket_id: BucketId, resource: Resource) -> () {
            self.message_bucket_alloc_into_cluster(bucket_id, resource).unwrap()
        }

        #[ink(message)]
        pub fn bucket_settle_payment(&mut self, bucket_id: BucketId) {
            self.message_bucket_settle_payment(bucket_id).unwrap()
        }

        #[ink(message, payable)]
        pub fn bucket_change_params(&mut self, bucket_id: BucketId, params: BucketParams) {
            self.message_bucket_change_params(bucket_id, params).unwrap();
        }

        #[ink(message)]
        pub fn bucket_get(&self, bucket_id: BucketId) -> Result<BucketStatus> {
            self.message_bucket_get(bucket_id)
        }

        #[ink(message)]
        pub fn bucket_list(&self, offset: u32, limit: u32, filter_owner_id: Option<AccountId>) -> (Vec<BucketStatus>, u32) {
            self.message_bucket_list(offset, limit, filter_owner_id)
        }
    }
    // ---- End Bucket ----


    // ---- Cluster ----

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct ClusterCreated {
        #[ink(topic)]
        cluster_id: ClusterId,
        #[ink(topic)]
        manager: AccountId,
        cluster_params: ClusterParams,
    }

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct ClusterNodeReplaced {
        #[ink(topic)]
        cluster_id: ClusterId,
        #[ink(topic)]
        node_id: NodeId,
        vnode_index: VNodeIndex,
    }

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct ClusterReserveResource {
        #[ink(topic)]
        cluster_id: ClusterId,
        resource: Resource,
    }

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct ClusterDistributeRevenues {
        #[ink(topic)]
        cluster_id: ClusterId,
        #[ink(topic)]
        provider_id: AccountId,
    }

    impl DdcBucket {
        #[ink(message, payable)]
        pub fn cluster_create(&mut self, manager: AccountId, vnode_count: u32, node_ids: Vec<NodeId>, cluster_params: ClusterParams) -> NodeId {
            self.message_cluster_create(manager, vnode_count, node_ids, cluster_params).unwrap()
        }

        #[ink(message)]
        pub fn cluster_reserve_resource(&mut self, cluster_id: ClusterId, amount: Resource) -> () {
            self.message_cluster_reserve_resource(cluster_id, amount).unwrap()
        }

        #[ink(message)]
        pub fn cluster_replace_node(&mut self, cluster_id: ClusterId, vnode_i: VNodeIndex, new_node_id: NodeId) -> () {
            self.message_cluster_replace_node(cluster_id, vnode_i, new_node_id).unwrap()
        }

        #[ink(message)]
        pub fn cluster_distribute_revenues(&mut self, cluster_id: ClusterId) {
            self.message_cluster_distribute_revenues(cluster_id).unwrap()
        }

        #[ink(message, payable)]
        pub fn cluster_change_params(&mut self, cluster_id: ClusterId, params: ClusterParams) {
            self.message_cluster_change_params(cluster_id, params).unwrap();
        }

        #[ink(message)]
        pub fn cluster_get(&self, cluster_id: ClusterId) -> Result<ClusterStatus> {
            self.message_cluster_get(cluster_id)
        }

        #[ink(message)]
        pub fn cluster_list(&self, offset: u32, limit: u32, filter_manager_id: Option<AccountId>) -> (Vec<ClusterStatus>, u32) {
            self.message_cluster_list(offset, limit, filter_manager_id)
        }
    }
    // ---- End Cluster ----


    // ---- Node ----

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct NodeCreated {
        #[ink(topic)]
        node_id: NodeId,
        #[ink(topic)]
        provider_id: AccountId,
        rent_per_month: Balance,
        node_params: NodeParams,
    }

    impl DdcBucket {
        #[ink(message, payable)]
        pub fn node_trust_manager(&mut self, manager: AccountId) {
            self.message_node_trust_manager(manager, true).unwrap();
        }

        #[ink(message)]
        pub fn node_distrust_manager(&mut self, manager: AccountId) {
            self.message_node_trust_manager(manager, false).unwrap();
        }

        #[ink(message, payable)]
        pub fn node_create(&mut self, rent_per_month: Balance, node_params: NodeParams, capacity: Resource) -> NodeId {
            self.message_node_create(rent_per_month, node_params, capacity).unwrap()
        }

        #[ink(message, payable)]
        pub fn node_change_params(&mut self, node_id: NodeId, params: NodeParams) {
            self.message_node_change_params(node_id, params).unwrap();
        }

        #[ink(message)]
        pub fn node_get(&self, node_id: NodeId) -> Result<NodeStatus> {
            self.message_node_get(node_id)
        }

        #[ink(message)]
        pub fn node_list(&self, offset: u32, limit: u32, filter_provider_id: Option<AccountId>) -> (Vec<NodeStatus>, u32) {
            self.message_node_list(offset, limit, filter_provider_id)
        }
    }
    // ---- End Node ----


    // ---- Billing ----

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct Deposit {
        #[ink(topic)]
        account_id: AccountId,
        value: Balance,
    }

    impl DdcBucket {
        #[ink(message, payable)]
        pub fn account_deposit(&mut self) -> () {
            self.message_account_deposit().unwrap()
        }

        #[ink(message)]
        pub fn account_get(&self, account_id: AccountId) -> Result<Account> {
            Ok(self.accounts.get(&account_id)?.clone())
        }

        #[ink(message)]
        pub fn account_get_usd_per_cere(&self) -> Balance {
            self.message_account_get_usd_per_cere()
        }

        #[ink(message)]
        pub fn account_set_usd_per_cere(&mut self, usd_per_cere: Balance) {
            self.message_account_set_usd_per_cere(usd_per_cere);
        }
    }
    // ---- End Billing ----


    // ---- Permissions ----
    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct GrantPermission {
        #[ink(topic)]
        account_id: AccountId,
        permission: Permission,
    }

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct RevokePermission {
        #[ink(topic)]
        account_id: AccountId,
        permission: Permission,
    }

    impl DdcBucket {
        #[ink(message)]
        pub fn has_permission(&self, grantee: AccountId, permission: Permission) -> bool {
            self.perms.has_permission(grantee, permission)
        }
    }
    // ---- End Permissions ----


    // ---- Admin ----
    impl DdcBucket {
        #[ink(message, payable)]
        pub fn admin_grant_permission(&mut self, grantee: AccountId, permission: Permission) {
            self.message_admin_grant_permission(grantee, permission, true).unwrap();
        }

        #[ink(message)]
        pub fn admin_revoke_permission(&mut self, grantee: AccountId, permission: Permission) {
            self.message_admin_grant_permission(grantee, permission, false).unwrap();
        }

        #[ink(message)]
        pub fn admin_withdraw(&mut self, amount: Balance) {
            self.message_admin_withdraw(amount).unwrap();
        }
    }
    // ---- End Admin ----


    // ---- Utils ----
    /// One token with 10 decimals.
    pub const TOKEN: Balance = 10_000_000_000;

    #[derive(Debug, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        BucketDoesNotExist,
        ClusterDoesNotExist,
        TooManyVNodes,
        ParamsTooBig,
        VNodeDoesNotExist,
        BucketClusterAlreadyConnected,
        BucketClusterNotSetup,
        NodeDoesNotExist,
        FlowDoesNotExist,
        AccountDoesNotExist,
        ParamsDoesNotExist,
        UnauthorizedProvider,
        UnauthorizedOwner,
        UnauthorizedClusterManager,
        ClusterManagerIsNotTrusted,
        TransferFailed,
        InsufficientBalance,
        InsufficientResources,
        Unauthorized,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl From<Error> for ink_env::Error {
        fn from(_: Error) -> Self {
            ink_env::Error::Unknown
        }
    }
    // ---- End Utils ----

    #[cfg(test)]
    mod tests;
}
