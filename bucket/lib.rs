//! The DDC smart contract implementing cluster-based services, and bucket-based subscriptions.

#![cfg_attr(not(feature = "std"), no_std)]
#![feature(proc_macro_hygiene)] // for tests in a separate file
#![deny(unused_must_use, unused_variables)]

use ink_lang as ink;

#[ink::contract]
pub mod ddc_bucket {
    use ink_prelude::string::ToString;
    use ink_prelude::vec::Vec;
    use scale::{Decode, Encode};

    use account::store::*;
    use bucket::{entity::*, store::*};
    use cash::*;
    use cluster::{entity::*, store::*};
    use node::{entity::*, store::*};
    use params::{store::*};
    use perm::{store::*};
    use committer::{store::*};

    use crate::ddc_bucket::account::entity::Account;
    use crate::ddc_bucket::network_fee::{FeeConfig, NetworkFeeStore};
    use crate::ddc_bucket::perm::entity::Permission;
    use crate::ddc_bucket::committer::store::EraConfig;
    use crate::ddc_bucket::cdn_node::store::CdnNodeStore;
    use crate::ddc_bucket::cdn_cluster::entity::CdnClusterStatus;
    
    use self::cdn_cluster::store::CdnClusterStore;
    use self::cdn_node::entity::CdnNode;

    pub mod account;
    pub mod flow;
    pub mod schedule;
    pub mod cash;
    pub mod cdn_node;
    pub mod node;
    pub mod bucket;
    pub mod cluster;
    pub mod cdn_cluster;
    pub mod contract_fee;
    pub mod network_fee;
    pub mod params;
    pub mod admin;
    pub mod perm;
    pub mod currency;
    pub mod committer;

    // ---- Global state ----
    /// The main DDC smart contract.
    #[ink(storage)]
    pub struct DdcBucket {
        buckets: BucketStore,
        bucket_params: ParamsStore,
        clusters: ClusterStore,
        cdn_clusters: CdnClusterStore,
        cluster_params: ParamsStore,
        cdn_nodes: CdnNodeStore,
        nodes: NodeStore,
        node_params: ParamsStore,
        accounts: AccountStore,
        perms: PermStore,
        network_fee: NetworkFeeStore,
        committer_store: CommitterStore,
    }

    impl DdcBucket {
        /// Create a new contract.
        ///
        /// The caller will be admin of the contract.
        #[ink(constructor)]
        pub fn new() -> Self {
            let operator = Self::env().caller();

            let mut contract = Self {
                buckets: BucketStore::default(),
                bucket_params: ParamsStore::default(),
                clusters: ClusterStore::default(),
                cluster_params: ParamsStore::default(),
                cdn_nodes: CdnNodeStore::default(),
                cdn_clusters: CdnClusterStore:: default(),
                nodes: NodeStore::default(),
                node_params: ParamsStore::default(),
                accounts: AccountStore::default(),
                perms: PermStore::default(),
                network_fee: NetworkFeeStore::default(),
                committer_store: CommitterStore::new(operator),
            };

            // Make the creator of this contract a super-admin.
            let admin_id = Self::env().caller();
            contract.perms.grant_permission(admin_id, &Permission::SuperAdmin);

            // Reserve IDs 0.
            let _ = contract.accounts.create_if_not_exist(AccountId::default());
            let _ = contract.cdn_nodes.create(AccountId::default(), 0);
            let _ = contract.nodes.create(AccountId::default(), 0, 0);
            let _ = contract.node_params.create("".to_string());
            let _ = contract.clusters.create(AccountId::default(), 0, &[]).unwrap();
            let _ = contract.cluster_params.create("".to_string());
            let _ = contract.buckets.create(AccountId::default(), 0);
            let _ = contract.bucket_params.create("".to_string());

            contract
        }
    }
    // ---- End global state ----

    // ---- Bucket ----

    /// A bucket was created. The given account is its first owner and payer of resources.
    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct BucketCreated {
        #[ink(topic)]
        bucket_id: BucketId,
        #[ink(topic)]
        owner_id: AccountId,
    }

    /// Some amount of resources of a cluster were allocated to a bucket.
    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct BucketAllocated {
        #[ink(topic)]
        bucket_id: BucketId,
        #[ink(topic)]
        cluster_id: ClusterId,
        resource: Resource,
    }

    /// The due costs of a bucket was settled from the bucket payer to the cluster.
    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct BucketSettlePayment {
        #[ink(topic)]
        bucket_id: BucketId,
        #[ink(topic)]
        cluster_id: ClusterId,
    }

    impl DdcBucket {
        /// Create a new bucket and return its `bucket_id`.
        ///
        /// The caller will be its first owner and payer of resources.
        ///
        /// `bucket_params` is configuration used by clients and nodes. See the [data structure of BucketParams](https://docs.cere.network/ddc/protocols/contract-params-schema)
        ///
        /// The bucket can be connected to a single cluster (currently). Allocate cluster resources with the function `bucket_alloc_into_cluster`
        #[ink(message, payable)]
        pub fn bucket_create(&mut self, bucket_params: BucketParams, cluster_id: ClusterId) -> BucketId {
            self.message_bucket_create(bucket_params, cluster_id).unwrap()
        }

        /// Allocate some resources of a cluster to a bucket.
        ///
        /// The amount of resources is given per vnode (total resources will be `resource` times the number of vnodes).
        #[ink(message)]
        pub fn bucket_alloc_into_cluster(&mut self, bucket_id: BucketId, resource: Resource) -> () {
            self.message_bucket_alloc_into_cluster(bucket_id, resource).unwrap()
        }

        /// Settle the due costs of a bucket from its payer account to the cluster account.
        #[ink(message)]
        pub fn bucket_settle_payment(&mut self, bucket_id: BucketId) {
            self.message_bucket_settle_payment(bucket_id).unwrap()
        }

        /// Change the `bucket_params`, which is configuration used by clients and nodes.
        ///
        /// See the [data structure of BucketParams](https://docs.cere.network/ddc/protocols/contract-params-schema)
        #[ink(message, payable)]
        pub fn bucket_change_params(&mut self, bucket_id: BucketId, params: BucketParams) {
            self.message_bucket_change_params(bucket_id, params).unwrap();
        }

        /// Get the current status of a bucket.
        #[ink(message)]
        pub fn bucket_get(&self, bucket_id: BucketId) -> Result<BucketStatus> {
            self.message_bucket_get(bucket_id)
        }

        /// Iterate through all buckets.
        ///
        /// The algorithm for paging is: start with `offset = 1` and `limit = 20`. The function returns a `(results, max_id)`. Call again with `offset += limit`, until `offset >= max_id`.
        /// The optimal `limit` depends on the size of params.
        ///
        /// The results can be filtered by owner. Note that paging must still be completed fully.
        #[ink(message)]
        pub fn bucket_list(&self, offset: u32, limit: u32, filter_owner_id: Option<AccountId>) -> (Vec<BucketStatus>, u32) {
            self.message_bucket_list(offset, limit, filter_owner_id)
        }

        /// Set availiablity of the bucket
        #[ink(message)]
        pub fn bucket_set_availability(&mut self, bucket_id: BucketId, public_availability: bool) -> () {
            self.message_bucket_set_availability(bucket_id, public_availability).unwrap()
        }

        /// Set max resource cap to be charged by CDN for public bucket
        #[ink(message)]
        pub fn bucket_set_resource_cap(&mut self, bucket_id: BucketId, new_resource_cap: Resource) -> () {
            self.message_bucket_set_resource_cap(bucket_id, new_resource_cap).unwrap()
        }
    }
    // ---- End Bucket ----


    // ---- Cluster ----

    /// A new cluster was created.
    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct ClusterCreated {
        #[ink(topic)]
        cluster_id: ClusterId,
        #[ink(topic)]
        manager: AccountId,
        cluster_params: ClusterParams,
    }

    /// A vnode was re-assigned to new node.
    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct ClusterNodeReplaced {
        #[ink(topic)]
        cluster_id: ClusterId,
        #[ink(topic)]
        node_id: NodeId,
        vnode_index: VNodeIndex,
    }

    /// Some resources were reserved for the cluster from the nodes.
    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct ClusterReserveResource {
        #[ink(topic)]
        cluster_id: ClusterId,
        resource: Resource,
    }

    /// The share of revenues of a cluster for a provider was distributed.
    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct ClusterDistributeRevenues {
        #[ink(topic)]
        cluster_id: ClusterId,
        #[ink(topic)]
        provider_id: AccountId,
    }

    impl DdcBucket {
        /// Create a new cluster and return its `cluster_id`.
        ///
        /// The caller will be its first manager.
        ///
        /// The cluster is split in a number of vnodes. The vnodes are assigned to the given physical nodes in a round-robin. Only nodes of providers that trust the cluster manager can be used (see `node_trust_manager`). The assignment can be changed with the function `cluster_replace_node`.
        ///
        /// `cluster_params` is configuration used by clients and nodes. In particular, this describes the semantics of vnodes. See the [data structure of ClusterParams](https://docs.cere.network/ddc/protocols/contract-params-schema)
        #[ink(message, payable)]
        pub fn cluster_create(&mut self, _unused: AccountId, vnode_count: u32, node_ids: Vec<NodeId>, cluster_params: ClusterParams) -> ClusterId {
            self.message_cluster_create(vnode_count, node_ids, cluster_params).unwrap()
        }

        /// As manager, reserve more resources for the cluster from the free capacity of nodes.
        ///
        /// The amount of resources is given per vnode (total resources will be `resource` times the number of vnodes).
        #[ink(message)]
        pub fn cluster_reserve_resource(&mut self, cluster_id: ClusterId, amount: Resource) -> () {
            self.message_cluster_reserve_resource(cluster_id, amount).unwrap()
        }

        /// As manager, re-assign a vnode to another physical node.
        ///
        /// The cluster manager can only use nodes of providers that trust him (see `node_trust_manager`), or any nodes if he is also SuperAdmin.
        #[ink(message)]
        pub fn cluster_replace_node(&mut self, cluster_id: ClusterId, vnode_i: VNodeIndex, new_node_id: NodeId) -> () {
            self.message_cluster_replace_node(cluster_id, vnode_i, new_node_id).unwrap()
        }

        /// Trigger the distribution of revenues from the cluster to the providers.
        #[ink(message)]
        pub fn cluster_distribute_revenues(&mut self, cluster_id: ClusterId) {
            self.message_cluster_distribute_revenues(cluster_id).unwrap()
        }

        /// Change the `cluster_params`, which is configuration used by clients and nodes.
        ///
        /// See the [data structure of ClusterParams](https://docs.cere.network/ddc/protocols/contract-params-schema)
        #[ink(message, payable)]
        pub fn cluster_change_params(&mut self, cluster_id: ClusterId, params: ClusterParams) {
            self.message_cluster_change_params(cluster_id, params).unwrap();
        }

        /// Get the current status of a cluster.
        #[ink(message)]
        pub fn cluster_get(&self, cluster_id: ClusterId) -> Result<ClusterStatus> {
            self.message_cluster_get(cluster_id)
        }

        /// Iterate through all clusters.
        ///
        /// The algorithm for paging is: start with `offset = 1` and `limit = 20`. The function returns a `(results, max_id)`. Call again with `offset += limit`, until `offset >= max_id`.
        /// The optimal `limit` depends on the size of params.
        ///
        /// The results can be filtered by manager. Note that paging must still be completed fully.
        #[ink(message)]
        pub fn cluster_list(&self, offset: u32, limit: u32, filter_manager_id: Option<AccountId>) -> (Vec<ClusterStatus>, u32) {
            self.message_cluster_list(offset, limit, filter_manager_id)
        }
    }
    // ---- End Cluster ----

    // ---- CDN Cluster ----

    /// A new cluster was created.
    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct CdnClusterCreated {
        #[ink(topic)]
        cluster_id: ClusterId,
        #[ink(topic)]
        manager: AccountId,
    }

    /// The share of revenues of a cluster for a provider was distributed.
    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct CdnClusterDistributeRevenues {
        #[ink(topic)]
        cluster_id: ClusterId,
        #[ink(topic)]
        provider_id: AccountId,
    }

    impl DdcBucket {
        /// Create a new cluster and return its `cluster_id`.
        ///
        /// The caller will be its first manager.
        ///
        /// The cluster is split in a number of vnodes. The vnodes are assigned to the given physical nodes in a round-robin. Only nodes of providers that trust the cluster manager can be used (see `node_trust_manager`). The assignment can be changed with the function `cluster_replace_node`.
        ///
        /// `cluster_params` is configuration used by clients and nodes. In particular, this describes the semantics of vnodes. See the [data structure of ClusterParams](https://docs.cere.network/ddc/protocols/contract-params-schema)
        #[ink(message, payable)]
        pub fn cdn_cluster_create(&mut self, _unused: AccountId, node_ids: Vec<NodeId>) -> ClusterId {
            self.message_cdn_cluster_create(node_ids).unwrap()
        }

        /// Set rate for streaming (price per gb)
        #[ink(message, payable)]
        pub fn cdn_set_rate(&mut self,  cluster_id: ClusterId, usd_per_gb: u128) -> () {
            self.message_cdn_set_rate(cluster_id, usd_per_gb).unwrap()
        }

        /// As manager, reserve more resources for the cluster from the free capacity of nodes.
        ///
        /// The amount of resources is given per vnode (total resources will be `resource` times the number of vnodes).
        #[ink(message)]
        pub fn cdn_cluster_put_revenue(&mut self, cluster_id: ClusterId, aggregates_accounts: Vec<(AccountId, u128)>, aggregates_nodes: Vec<(u32, u128)>) -> () {
            self.message_cdn_cluster_put_revenue(cluster_id, aggregates_accounts, aggregates_nodes).unwrap()
        }

        /// Trigger the distribution of revenues from the cluster to the providers.
        #[ink(message)]
        pub fn cdn_cluster_distribute_revenues(&mut self, cluster_id: ClusterId) {
            self.message_cdn_cluster_distribute_revenues(cluster_id).unwrap()
        }

        /// Get the current status of a cluster.
        #[ink(message)]
        pub fn cdn_cluster_get(&self, cluster_id: ClusterId) -> Result<CdnClusterStatus> {
            self.message_cdn_cluster_get(cluster_id)
        }

        /// Iterate through all clusters.
        ///
        /// The algorithm for paging is: start with `offset = 1` and `limit = 20`. The function returns a `(results, max_id)`. Call again with `offset += limit`, until `offset >= max_id`.
        /// The optimal `limit` depends on the size of params.
        ///
        /// The results can be filtered by manager. Note that paging must still be completed fully.
        #[ink(message)]
        pub fn cdn_cluster_list(&self, offset: u32, limit: u32, filter_manager_id: Option<AccountId>) -> (Vec<CdnClusterStatus>, u32) {
            self.message_cdn_cluster_list(offset, limit, filter_manager_id)
        }
    }
    // ---- End CDN Cluster ----

    // ---- Committer ----

    impl DdcBucket {
        #[ink(message)]
        pub fn get_commit(&self, node: AccountId) -> () {
            self.message_get_commit(node);
        }

        #[ink(message)]
        pub fn set_commit(&mut self, node: AccountId, commit: Commit) {
            self.message_set_commit(node, commit);
        }

        #[ink(message)]
        pub fn set_era(&mut self, era_config: EraConfig) -> () {
            self.message_set_era(era_config).unwrap();
        }
    
        #[ink(message)]
        pub fn get_era(&self) -> () {
            self.message_get_era();
        }

        #[ink(message)]
        pub fn get_era_settings(&self) -> () {
            self.message_get_era_settings();
        }

        #[ink(message)]
        pub fn new_era(&mut self) -> () {
            self.message_new_era().unwrap();
        }
    }
    // ---- End Committer ----

    // ---- CDN Node ----

    /// A node was created. The given account is its owner and recipient of revenues.
    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct CdnNodeCreated {
        #[ink(topic)]
        node_id: NodeId,
        #[ink(topic)]
        provider_id: AccountId,
        undistributed_payment: Balance,
    }

    impl DdcBucket {
        /// As node provider, authorize a cluster manager to use his nodes.
        #[ink(message, payable)]
        pub fn cdn_node_trust_manager(&mut self, manager: AccountId) {
            self.message_cdn_node_trust_manager(manager, true).unwrap();
        }

        /// As node provider, revoke the authorization of a cluster manager to use his nodes.
        #[ink(message)]
        pub fn cdn_node_distrust_manager(&mut self, manager: AccountId) {
            self.message_cdn_node_trust_manager(manager, false).unwrap();
        }

        /// Create a new node and return its `node_id`.
        ///
        /// The caller will be its owner.
        ///
        /// `node_params` is configuration used by clients and nodes. In particular, this contains the URL to the service. See the [data structure of NodeParams](https://docs.cere.network/ddc/protocols/contract-params-schema)
        #[ink(message, payable)]
        pub fn cdn_node_create(&mut self, undistributed_payment: Balance) -> NodeId {
            self.message_cdn_node_create(undistributed_payment).unwrap()
        }

        /// Get the current state of the cdn node
        #[ink(message)]
        pub fn cdn_node_get(&self, node_id: NodeId) -> Result<CdnNode> {
            self.message_cdn_node_get(node_id)
        }

        /// Iterate through all nodes.
        ///
        /// The algorithm for paging is: start with `offset = 1` and `limit = 20`. The function returns a `(results, max_id)`. Call again with `offset += limit`, until `offset >= max_id`.
        /// The optimal `limit` depends on the size of params.
        ///
        /// The results can be filtered by owner. Note that paging must still be completed fully.
        #[ink(message)]
        pub fn cdn_node_list(&self, offset: u32, limit: u32, filter_provider_id: Option<AccountId>) -> Vec<CdnNode> {
            self.message_cdn_node_list(offset, limit, filter_provider_id)
        }
    }
    // ---- End CDN Node ----

    // ---- Node ----

    /// A node was created. The given account is its owner and recipient of revenues.
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
        /// As node provider, authorize a cluster manager to use his nodes.
        #[ink(message, payable)]
        pub fn node_trust_manager(&mut self, manager: AccountId) {
            self.message_node_trust_manager(manager, true).unwrap();
        }

        /// As node provider, revoke the authorization of a cluster manager to use his nodes.
        #[ink(message)]
        pub fn node_distrust_manager(&mut self, manager: AccountId) {
            self.message_node_trust_manager(manager, false).unwrap();
        }

        /// Create a new node and return its `node_id`.
        ///
        /// The caller will be its owner.
        ///
        /// `node_params` is configuration used by clients and nodes. In particular, this contains the URL to the service. See the [data structure of NodeParams](https://docs.cere.network/ddc/protocols/contract-params-schema)
        #[ink(message, payable)]
        pub fn node_create(&mut self, rent_per_month: Balance, node_params: NodeParams, capacity: Resource) -> NodeId {
            self.message_node_create(rent_per_month, node_params, capacity).unwrap()
        }

        /// Change the `node_params`, which is configuration used by clients and nodes.
        ///
        /// See the [data structure of NodeParams](https://docs.cere.network/ddc/protocols/contract-params-schema)
        #[ink(message, payable)]
        pub fn node_change_params(&mut self, node_id: NodeId, params: NodeParams) {
            self.message_node_change_params(node_id, params).unwrap();
        }

        /// Get the current status of a node.
        #[ink(message)]
        pub fn node_get(&self, node_id: NodeId) -> Result<NodeStatus> {
            self.message_node_get(node_id)
        }

        /// Iterate through all nodes.
        ///
        /// The algorithm for paging is: start with `offset = 1` and `limit = 20`. The function returns a `(results, max_id)`. Call again with `offset += limit`, until `offset >= max_id`.
        /// The optimal `limit` depends on the size of params.
        ///
        /// The results can be filtered by owner. Note that paging must still be completed fully.
        #[ink(message)]
        pub fn node_list(&self, offset: u32, limit: u32, filter_provider_id: Option<AccountId>) -> (Vec<NodeStatus>, u32) {
            self.message_node_list(offset, limit, filter_provider_id)
        }
    }
    // ---- End Node ----


    // ---- Billing ----

    /// Tokens were deposited on an account.
    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct Deposit {
        #[ink(topic)]
        account_id: AccountId,
        value: Balance,
    }

    impl DdcBucket {
        /// As user, deposit tokens on the account of the caller from the transaction value. This deposit
        /// can be used to pay for the services to buckets of the account.
        #[ink(message, payable)]
        pub fn account_deposit(&mut self) -> () {
            self.message_account_deposit().unwrap()
        }

        /// As user, bond some amount of tokens from the withdrawable balance. These funds will be used to 
        /// pay for CDN nodes
        #[ink(message, payable)]
        pub fn account_bond(&mut self, payable: Payable) -> () {
            self.message_account_bond(payable).unwrap()
        }

        /// As user, unbond a specified amount of tokens
        #[ink(message, payable)]
        pub fn account_unbond(&mut self, amount_to_unbond: Cash) -> () {
            self.message_account_unbond(amount_to_unbond).unwrap()
        }

        /// As user, move the unbonded tokens back to withdrawable balance state
        #[ink(message, payable)]
        pub fn account_withdraw_unbonded(&mut self) -> () {
            self.message_account_withdraw_unbonded().unwrap()
        }
        
        /// Get the current status of an account.
        #[ink(message)]
        pub fn account_get(&self, account_id: AccountId) -> Result<Account> {
            Ok(self.accounts.get(&account_id)?.clone())
        }

        /// Get the current conversion rate between the native currency and an external currency (USD).
        #[ink(message)]
        pub fn account_get_usd_per_cere(&self) -> Balance {
            self.message_account_get_usd_per_cere()
        }

        /// As price oracle, set the current conversion rate between the native currency and an external currency (USD).
        ///
        /// This requires the permission SetExchangeRate or SuperAdmin.
        #[ink(message)]
        pub fn account_set_usd_per_cere(&mut self, usd_per_cere: Balance) {
            self.message_account_set_usd_per_cere(usd_per_cere);
        }
    }
    // ---- End Billing ----


    // ---- Permissions ----
    /// A permission was granted to the account.
    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct GrantPermission {
        #[ink(topic)]
        account_id: AccountId,
        permission: Permission,
    }

    /// A permission was revoked from the account.
    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct RevokePermission {
        #[ink(topic)]
        account_id: AccountId,
        permission: Permission,
    }

    impl DdcBucket {
        /// Check whether the given account has the given permission currently,
        /// or the SuperAdmin permission.
        #[ink(message)]
        pub fn has_permission(&self, grantee: AccountId, permission: Permission) -> bool {
            self.perms.has_permission(grantee, permission)
        }
    }
    // ---- End Permissions ----


    // ---- Admin ----
    impl DdcBucket {
        /// As SuperAdmin, grant any permission to any account.
        #[ink(message, payable)]
        pub fn admin_grant_permission(&mut self, grantee: AccountId, permission: Permission) {
            self.message_admin_grant_permission(grantee, permission, true).unwrap();
        }

        /// As SuperAdmin, revoke any permission to any account.
        #[ink(message)]
        pub fn admin_revoke_permission(&mut self, grantee: AccountId, permission: Permission) {
            self.message_admin_grant_permission(grantee, permission, false).unwrap();
        }

        /// As SuperAdmin, withdraw the funds held in custody in this contract.
        ///
        /// This is a temporary measure to allow migrating the funds to a new version of the contract.
        #[ink(message)]
        pub fn admin_withdraw(&mut self, amount: Balance) {
            self.message_admin_withdraw(amount).unwrap();
        }

        /// As SuperAdmin, set the network and cluster fee configuration.
        #[ink(message)]
        pub fn admin_set_fee_config(&mut self, config: FeeConfig) {
            self.message_admin_set_fee_config(config).unwrap();
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
        BondingPeriodNotFinished,
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
