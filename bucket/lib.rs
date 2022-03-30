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
    use deal::{entity::*, store::*};
    use Error::*;
    use node::{entity::*, store::*};

    pub mod account;
    pub mod flow;
    pub mod schedule;
    pub mod cash;
    pub mod node;
    pub mod bucket;
    pub mod deal;
    pub mod cluster;
    pub mod contract_fee;

    // ---- Global state ----
    #[ink(storage)]
    pub struct DdcBucket {
        buckets: BucketStore,
        deals: DealStore,
        clusters: ClusterStore,
        nodes: NodeStore,
        accounts: AccountStore,
    }

    impl DdcBucket {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                buckets: BucketStore::default(),
                deals: DealStore::default(),
                clusters: ClusterStore::default(),
                nodes: NodeStore::default(),
                accounts: AccountStore::default(),
            }
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
    }

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct DealCreated {
        #[ink(topic)]
        deal_id: DealId,
        #[ink(topic)]
        bucket_id: BucketId,
        #[ink(topic)]
        node_id: NodeId,
    }

    impl DdcBucket {
        #[ink(message, payable)]
        pub fn bucket_create(&mut self, bucket_params: BucketParams, cluster_id: ClusterId) -> BucketId {
            self.message_bucket_create(bucket_params, cluster_id).unwrap()
        }

        #[ink(message, payable)]
        pub fn bucket_alloc_into_cluster(&mut self, bucket_id: BucketId) -> () {
            self.message_bucket_alloc_into_cluster(bucket_id).unwrap()
        }

        #[ink(message)]
        pub fn bucket_settle_payment(&mut self, bucket_id: BucketId) {
            self.message_bucket_settle_payment(bucket_id).unwrap()
        }

        /* Not allowed to reserve because it is not connected to payments yet.
        #[ink(message)]
        pub fn bucket_reserve_resource(&mut self, bucket_id: BucketId, amount: Resource) -> Result<()> {
            self._message_bucket_reserve_resource(bucket_id, amount).unwrap()
        }*/

        #[ink(message)]
        pub fn bucket_list_statuses(&self, offset: u32, limit: u32, filter_owner_id: Option<AccountId>) -> (Vec<BucketStatus>, u32) {
            self.message_bucket_list_statuses(offset, limit, filter_owner_id)
        }

        #[ink(message)]
        pub fn bucket_get(&self, bucket_id: BucketId) -> Result<Bucket> {
            Ok(self.buckets.get(bucket_id)?.clone())
        }

        #[ink(message)]
        pub fn bucket_get_status(&self, bucket_id: BucketId) -> Result<BucketStatus> {
            self.message_bucket_get_status(bucket_id)
        }
    }
    // ---- End Bucket ----


    // ---- Deal ----

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct ProviderWithdraw {
        #[ink(topic)]
        provider_id: AccountId,
        #[ink(topic)]
        deal_id: DealId,
        value: Balance,
    }

    impl DdcBucket {
        #[ink(message)]
        pub fn provider_withdraw(&mut self, deal_id: DealId) -> () {
            self.message_provider_withdraw(deal_id).unwrap()
        }

        #[ink(message)]
        pub fn deal_get_status(&self, deal_id: DealId) -> Result<DealStatus> {
            self.message_deal_get_status(deal_id)
        }
    }
    // ---- End Deal ----


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
        partition_index: PartitionIndex,
    }

    impl DdcBucket {
        #[ink(message, payable)]
        pub fn cluster_create(&mut self, manager: AccountId, partition_count: u32, node_ids: Vec<NodeId>, cluster_params: ClusterParams) -> NodeId {
            self.message_cluster_create(manager, partition_count, node_ids, cluster_params).unwrap()
        }

        #[ink(message)]
        pub fn cluster_reserve_resource(&mut self, cluster_id: ClusterId, amount: Resource) -> () {
            self.message_cluster_reserve_resource(cluster_id, amount).unwrap()
        }

        #[ink(message)]
        pub fn cluster_replace_node(&mut self, cluster_id: ClusterId, partition_i: PartitionIndex, new_node_id: NodeId) -> () {
            self.message_cluster_replace_node(cluster_id, partition_i, new_node_id).unwrap()
        }

        #[ink(message)]
        pub fn cluster_get(&self, cluster_id: ClusterId) -> Result<Cluster> {
            Ok(self.clusters.get(cluster_id)?.clone())
        }

        #[ink(message)]
        pub fn cluster_list(&self, offset: u32, limit: u32, filter_manager_id: Option<AccountId>) -> (Vec<Cluster>, u32) {
            self.clusters.list(offset, limit, filter_manager_id)
        }

        #[ink(message)]
        pub fn cluster_distribute_revenues(&mut self, cluster_id: ClusterId) {
            self.message_cluster_distribute_revenues(cluster_id).unwrap()
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
        pub fn node_create(&mut self, rent_per_month: Balance, node_params: NodeParams, capacity: Resource) -> NodeId {
            self.message_node_create(rent_per_month, node_params, capacity).unwrap()
        }

        #[ink(message)]
        pub fn node_get(&self, node_id: NodeId) -> Result<Node> {
            Ok(self.nodes.get(node_id)?.clone())
        }

        #[ink(message)]
        pub fn node_list(&self, offset: u32, limit: u32, filter_provider_id: Option<AccountId>) -> (Vec<Node>, u32) {
            self.nodes.list(offset, limit, filter_provider_id)
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
        pub fn deposit(&mut self) -> () {
            self.message_deposit().unwrap()
        }
    }
    // ---- End Billing ----


    // ---- Utils ----
    /// One token with 10 decimals.
    pub const TOKEN: Balance = 10_000_000_000;

    #[derive(Debug, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        BucketDoesNotExist,
        DealDoesNotExist,
        ClusterDoesNotExist,
        PartitionDoesNotExist,
        BucketClusterAlreadyConnected,
        BucketClusterNotSetup,
        NodeDoesNotExist,
        FlowDoesNotExist,
        AccountDoesNotExist,
        UnauthorizedProvider,
        UnauthorizedOwner,
        UnauthorizedClusterManager,
        TransferFailed,
        InsufficientBalance,
        InsufficientResources,
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
