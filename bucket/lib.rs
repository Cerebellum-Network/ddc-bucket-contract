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
    use flow::entity::*;
    use schedule::*;
    use vnode::{entity::*, store::*};

    pub mod billing;
    pub mod account;
    pub mod flow;
    pub mod schedule;
    pub mod cash;
    pub mod vnode;
    pub mod bucket;
    pub mod deal;
    pub mod cluster;

    // ---- Global state ----
    #[ink(storage)]
    pub struct DdcBucket {
        buckets: BucketStore,
        deals: DealStore,
        clusters: ClusterStore,
        vnodes: VNodeStore,
        accounts: AccountStore,
    }

    impl DdcBucket {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                buckets: BucketStore::default(),
                deals: DealStore::default(),
                clusters: ClusterStore::default(),
                vnodes: VNodeStore::default(),
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
        vnode_id: VNodeId,
    }

    impl DdcBucket {
        #[ink(message)]
        pub fn bucket_create(&mut self, bucket_params: BucketParams) -> Result<BucketId> {
            self.message_bucket_create(bucket_params)
        }

        #[ink(message, payable)]
        pub fn bucket_alloc_into_cluster(&mut self, bucket_id: BucketId, cluster_id: ClusterId) -> Result<()> {
            self.message_bucket_alloc_into_cluster(bucket_id, cluster_id)
        }

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
        pub fn provider_withdraw(&mut self, deal_id: DealId) -> Result<()> {
            self.message_provider_withdraw(deal_id)
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
        cluster_params: ClusterParams,
    }

    impl DdcBucket {
        #[ink(message)]
        pub fn cluster_create(&mut self, cluster_params: ClusterParams) -> Result<VNodeId> {
            self.message_cluster_create(cluster_params)
        }

        #[ink(message)]
        pub fn cluster_get(&self, cluster_id: ClusterId) -> Result<Cluster> {
            Ok(self.clusters.get(cluster_id)?.clone())
        }

        #[ink(message)]
        pub fn cluster_list(&self, offset: u32, limit: u32) -> (Vec<Cluster>, u32) {
            self.clusters.list(offset, limit)
        }
    }
    // ---- End Cluster ----


    // ---- VNode ----

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct VNodeCreated {
        #[ink(topic)]
        vnode_id: VNodeId,
        #[ink(topic)]
        provider_id: AccountId,
        rent_per_month: Balance,
        vnode_params: VNodeParams,
    }

    impl DdcBucket {
        #[ink(message)]
        pub fn vnode_create(&mut self, cluster_id: ClusterId, rent_per_month: Balance, vnode_params: VNodeParams) -> Result<VNodeId> {
            self.message_vnode_create(cluster_id, rent_per_month, vnode_params)
        }

        #[ink(message)]
        pub fn vnode_get(&self, vnode_id: VNodeId) -> Result<VNode> {
            Ok(self.vnodes.get(vnode_id)?.clone())
        }

        #[ink(message)]
        pub fn vnode_list(&self, offset: u32, limit: u32, filter_provider_id: Option<AccountId>) -> (Vec<VNode>, u32) {
            self.vnodes.list(offset, limit, filter_provider_id)
        }
    }
    // ---- End VNode ----


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
        pub fn deposit(&mut self) -> Result<()> {
            self.message_deposit()
        }
    }
    // ---- End Billing ----


    // ---- Utils ----
    #[derive(Debug, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        BucketDoesNotExist,
        DealDoesNotExist,
        ClusterDoesNotExist,
        BucketClusterAlreadyConnected,
        VNodeDoesNotExist,
        FlowDoesNotExist,
        AccountDoesNotExist,
        UnauthorizedProvider,
        UnauthorizedOwner,
        TransferFailed,
        InsufficientBalance,
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
