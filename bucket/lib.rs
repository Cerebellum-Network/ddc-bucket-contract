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
    
    use self::buckets_perms::store::BucketsPermsStore;
    use self::cdn_cluster::store::CdnClusterStore;
    use self::cdn_node::entity::CdnNodeStatus;
    use self::protocol::store::ProtocolStore;

    pub mod account;
    pub mod flow;
    pub mod schedule;
    pub mod cash;
    pub mod cdn_node;
    pub mod node;
    pub mod bucket;
    pub mod cluster;
    pub mod cdn_cluster;
    pub mod network_fee;
    pub mod params;
    pub mod admin;
    pub mod perm;
    pub mod currency;
    pub mod committer;
    pub mod buckets_perms;
    pub mod protocol;

    // ---- Global state ----
    /// The main DDC smart contract.
    #[ink(storage)]
    pub struct DdcBucket {
        buckets: BucketStore,
        buckets_perms: BucketsPermsStore,
        bucket_params: ParamsStore,
        clusters: ClusterStore,
        cdn_clusters: CdnClusterStore,
        cluster_params: ParamsStore,
        cdn_nodes: CdnNodeStore,
        cdn_node_params: ParamsStore,
        nodes: NodeStore,
        node_params: ParamsStore,
        accounts: AccountStore,
        perms: PermStore,
        network_fee: NetworkFeeStore,
        committer_store: CommitterStore,
        protocol_store: ProtocolStore,
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
                buckets_perms: BucketsPermsStore::default(),
                bucket_params: ParamsStore::default(),
                clusters: ClusterStore::default(),
                cluster_params: ParamsStore::default(),
                cdn_nodes: CdnNodeStore::default(),
                cdn_node_params: ParamsStore::default(),
                cdn_clusters: CdnClusterStore:: default(),
                nodes: NodeStore::default(),
                node_params: ParamsStore::default(),
                accounts: AccountStore::default(),
                perms: PermStore::default(),
                network_fee: NetworkFeeStore::default(),
                committer_store: CommitterStore::new(operator),
                protocol_store: ProtocolStore::new(operator, DEFAULT_BASIS_POINTS),
            };

            // Make the creator of this contract a super-admin.
            let admin_id = Self::env().caller();
            contract.perms.grant_permission(admin_id, &Permission::SuperAdmin);

            // Reserve IDs 0.
            let _ = contract.accounts.create_if_not_exist(AccountId::default());
            let _ = contract.cdn_nodes.create(AccountId::default(), 0);
            let _ = contract.cdn_node_params.create("".to_string());
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
        /// **Description**:
        ///     Create a new bucket.
        ///
        /// **Logic**:
        ///     1. The caller will be its first owner  
        ///     2. The call will also be the payer of resources.
        ///     3. The bucket can be connected to a single cluster (currently). 
        ///     4. To allocate more cluster resources one should use function `bucket_alloc_into_cluster
        /// 
        /// **Permissions**:
        ///     1. Anyone can create a bucket
        ///
        /// **Params**:
        ///     1. `bucket_params` is configuration used by clients and nodes. See the [data structure of BucketParams](https://docs.cere.network/ddc/protocols/contract-params-schema)
        ///     2. `cluster_id` is the id of the cluster bucket belongs to
        ///     3. `owner_id` is the optional argument to specify the owner of the bucket (**Note: we should check from security standpoint if it is sound to allow a third party to create buckets for you, which might drain funds; new payment model might be the solution**))
        /// 
        /// **Events**:
        ///     1. BucketCreated(bucket id, owner id)
        /// 
        /// **Storage** (items are written to storage):
        ///     1. account (if owner account doesn't exist prior to call)
        ///     2. bucket
        ///     2. bucket_params
        ///     Note: there is currently no methods to delete bucket, hence these items will be stored during the lifetime of the contract
        /// 
        /// **Errors**: 
        ///     1. if bucket id is not the same as params id (may be triggered by faulty logic in other method)
        /// 
        /// **Returns**:
        ///     `bucket_id` - id of the newly created bucket.
        #[ink(message, payable)]
        pub fn bucket_create(&mut self, bucket_params: BucketParams, cluster_id: ClusterId, owner_id: Option<AccountId>) -> BucketId {
            self.message_bucket_create(bucket_params, cluster_id, owner_id).unwrap()
        }

        /// **Description**:
        ///     Change owner of the bucket.
        ///
        /// **Logic**:
        ///     1. If owner of the bucket triggers the method, newly provided owner will be assigned 
        /// 
        /// **Permissions**:
        ///     1. Only bucket owner 
        ///
        /// **Params**:
        ///     1. `bucket_id` is the id of the bucket of interest
        ///     2. `owner_id` is the AccoundId of the new bucket owner
        /// 
        /// **Events**:
        ///     1. No events emitted
        /// 
        /// **Storage** (items are written to storage):
        ///     1. New owner id written to bucket storage
        /// 
        /// **Errors**: 
        ///     1. transaction will revert if the caller is not the owner 
        /// 
        /// **Returns**:
        ///     Nothing
        #[ink(message, payable)]
        pub fn bucket_change_owner(&mut self, bucket_id: BucketId, owner_id: AccountId) -> () {
        self.message_bucket_change_owner(bucket_id, owner_id).unwrap()
        }

        /// **Description**:
        ///     Allocate some resources of a cluster to a bucket
        ///
        /// **Logic**:
        ///     1. The amount of resources is given per vnode (total resources will be `resource` times the number of vnodes).
        /// 
        /// **Permissions**:
        ///     1. Bucket owner
        ///     2. Cluster manager 
        ///
        /// **Params**:
        ///     1. `bucket_id` is the id of the bucket of interest
        ///     2. `resource` is the amount of resources added to the bucket (**Note: clarify why amount of resource has to be multiplied per vnode**)
        /// 
        /// **Events**:
        ///     1. BucketAllocated(bucket id, cluster id, resource)
        /// 
        /// **Storage** (items are written to storage):
        ///     1. cluster storage write
        ///     2. bucket storage write
        ///     3. account storage write
        /// 
        /// **Errors**: 
        ///     1. transaction will revert if the cluster does not have enough available resources 
        ///     2. transaction will revert if the caller is not the owner or cluster manager
        /// 
        /// **Returns**:
        ///     Nothing
        #[ink(message)]
        pub fn bucket_alloc_into_cluster(&mut self, bucket_id: BucketId, resource: Resource) -> () {
            self.message_bucket_alloc_into_cluster(bucket_id, resource).unwrap()
        }

        /// **Description**:
        ///     Settle the due costs of a bucket from its payer account to the cluster account
        ///
        /// **Logic**:
        ///     1. The owner of the bucket is charged the funds with method 'settle_flow'
        ///     2. Cluster is allocated funds with method 'revenues.increase'
        ///     **Note: do we actually want to collect payments from buckets this way?**
        /// 
        /// **Permissions**:
        ///     1. Anyone
        ///
        /// **Params**:
        ///     1. `bucket_id` is the id of the bucket for which payment will be done 
        /// 
        /// **Events**:
        ///     1. BucketSettlePayment(bucket_id, cluster_id)
        /// 
        /// **Storage** (items are written to storage):
        ///     1. cluster storage write
        ///     2. bucket storage write
        ///     3. account storage write
        /// 
        /// **Errors**: 
        ///     1. transaction will revert if bucket with provided id does not exist
        /// 
        /// **Returns**:
        ///     Nothing
        #[ink(message)]
        pub fn bucket_settle_payment(&mut self, bucket_id: BucketId) {
            self.message_bucket_settle_payment(bucket_id).unwrap()
        }

        /// **Description**:
        ///     Change the `bucket_params` 
        ///
        /// **Logic**:
        ///     1. Bucket params is the configuration used by clients and nodes.
        /// 
        /// **Permissions**:
        ///     1. Bucket owner
        ///
        /// **Params**:
        ///     1. `bucket_id` is the id of the bucket of interest
        ///     2. `params` See the [data structure of BucketParams](https://docs.cere.network/ddc/protocols/contract-params-schema)
        /// 
        /// **Events**:
        ///     1. None (**Do we want to add event here?**)
        /// 
        /// **Storage** (items are written to storage):
        ///     1. params storage write
        ///
        /// **Errors**: 
        ///     1. transaction will revert if bucket with provided id does not exist
        ///     2. transaction will revert if the caller is not the owner of the bucket
        /// 
        /// **Returns**:
        ///     Nothing
        #[ink(message, payable)]
        pub fn bucket_change_params(&mut self, bucket_id: BucketId, params: BucketParams) {
            self.message_bucket_change_params(bucket_id, params).unwrap();
        }

        /// **Description**:
        ///     Get the current status of a bucket 
        ///
        /// **Logic**:
        ///     1. Calculate the current status of the bucket(rent), fetch readers and writers and return
        /// 
        /// **Permissions**:
        ///     1. Anyone
        ///
        /// **Params**:
        ///     1. `bucket_id` is the id of the bucket of interest
        /// 
        /// **Events**:
        ///     1. None 
        /// 
        /// **Storage** (items are written to storage):
        ///     1. None
        ///
        /// **Errors**: 
        ///     1. None
        /// 
        /// **Returns**:
        ///     BucketStatus(bucket_id, bucket, params, writer_ids, reader_ids, rent_covered_until_ms)
        #[ink(message)]
        pub fn bucket_get(&self, bucket_id: BucketId) -> Result<BucketStatus> {
            self.message_bucket_get(bucket_id)
        }

        /// **Description**:
        ///     Iterate through all buckets
        ///
        /// **Logic**:
        ///     1. The algorithm for paging is: start with `offset = 1` and `limit = 20` 
        ///     2. The function returns a `(results, max_id)`. 
        ///     3. Call again with `offset += limit`, until `offset >= max_id`.
        ///     4. The optimal `limit` depends on the size of params. (**Note: what does it mean?**)
        ///     5. For all returned buckets the same logic is apllied as for call `bucket_get`
        ///     6. Result can be filtered by owner
        /// 
        /// **Permissions**:
        ///     1. Anyone
        ///
        /// **Params**:
        ///     1. `offset` is the number of buckets to skip
        ///     2. `limit` is the number of buckets per page
        ///     3. `filter_owner_id` is the optional parameter, which filters accounts on the selected page (**Note: filter ideally should be before pagination**) 
        /// 
        /// **Events**:
        ///     1. None 
        /// 
        /// **Storage** (items are written to storage):
        ///     1. None
        ///
        /// **Errors**: 
        ///     1. None
        /// 
        /// **Returns**:
        ///     Vec<BucketStatus>, vec_length; BucketStatus is (bucket_id, bucket, params, writer_ids, reader_ids, rent_covered_until_ms)
        #[ink(message)]
        pub fn bucket_list(&self, offset: u32, limit: u32, filter_owner_id: Option<AccountId>) -> (Vec<BucketStatus>, u32) {
            self.message_bucket_list(offset, limit, filter_owner_id)
        }

        /// **Description**:
        ///     Get the buckets owner by an account
        ///
        /// **Logic**:
        ///     1. Iterate through all buckets and return all buckets owned by account
        ///     2. This method returns bucket struct, not the status
        /// 
        /// **Permissions**:
        ///     1. Anyone
        ///
        /// **Params**:
        ///     1. `owner_id` is the account of the buckets owner 
        /// 
        /// **Events**:
        ///     1. None 
        /// 
        /// **Storage** (items are written to storage):
        ///     1. None
        ///
        /// **Errors**: 
        ///     1. None
        /// 
        /// **Returns**:
        ///     Vec<Bucket>
        #[ink(message)]
        pub fn bucket_list_for_account(&self, owner_id: AccountId) -> Vec<Bucket> {
            self.message_bucket_list_for_account(owner_id)
        }

        /// **Description**:
        ///     Set availiablity of the bucket
        ///
        /// **Logic**:
        ///     1. Update availablity of the bucket. 
        ///     2. True equals public availability, false means private bucket availability 
        /// 
        /// **Permissions**:
        ///     1. Bucket owner
        ///     2. Cluster manager
        ///
        /// **Params**:
        ///     1. `bucket_id` is the buckets of interest
        ///     2. `public_availability` is the bool representing public availability 
        /// 
        /// **Events**:
        ///     1. None 
        /// 
        /// **Storage** (items are written to storage):
        ///     1. None
        ///
        /// **Errors**: 
        ///     1. Transaction fails, if it is called by someone who's not a bucket owner or cluster manager
        /// 
        /// **Returns**:
        ///     Nothing
        #[ink(message)]
        pub fn bucket_set_availability(&mut self, bucket_id: BucketId, public_availability: bool) -> () {
            self.message_bucket_set_availability(bucket_id, public_availability).unwrap()
        }

        /// **Description**:
        ///     Set max resource cap to be charged by CDN for public bucket
        ///
        /// **Logic**:
        ///     1. Update availablity of the bucket. 
        ///     2. True equals public availability, false means private bucket availability 
        /// 
        /// **Permissions**:
        ///     1. Bucket owner
        ///     2. Cluster manager
        ///
        /// **Params**:
        ///     1. `bucket_id` is the buckets of interest
        ///     2. `public_availability` is the bool representing public availability 
        /// 
        /// **Events**:
        ///     1. None 
        /// 
        /// **Storage** (items are written to storage):
        ///     1. None
        ///
        /// **Errors**: 
        ///     1. Transaction fails, if it is called by someone who's not a bucket owner or cluster manager
        /// 
        /// **Returns**:
        ///     Nothing
        /// 
        #[ink(message)]
        pub fn bucket_set_resource_cap(&mut self, bucket_id: BucketId, new_resource_cap: Resource) -> () {
            self.message_bucket_set_resource_cap(bucket_id, new_resource_cap).unwrap()
        }

        /// **Description**:
        ///     Method retrieves bucket writers
        ///
        /// **Logic**:
        ///     1. Read from bucket_perms the permission for relevant bucket
        ///     2. Return vector of AccountIds who have permissions
        /// 
        /// **Permissions**:
        ///     1. Anyone
        ///
        /// **Params**:
        ///     1. `bucket_id` is the buckets of interest
        /// 
        /// **Events**:
        ///     1. None 
        /// 
        /// **Storage** (items are written to storage):
        ///     1. None
        ///     (**Note: we shall research how to combine this storage with bucket storage**)
        ///
        /// **Errors**: 
        ///     1. None
        /// 
        /// **Returns**:
        ///     Vector of AccountIds
        #[ink(message)]
        pub fn get_bucket_writers(&mut self, bucket_id: BucketId) -> Vec<AccountId> {
            self.message_get_bucket_writers(bucket_id).unwrap()
        }

        /// **Description**:
        ///     Set permission for the writer of the bucket
        ///
        /// **Logic**:
        ///     1. Grant permission for new writer via bucket_perms
        /// 
        /// **Permissions**:
        ///     1. Bucket owner
        ///     2. Cluster manager
        ///
        /// **Params**:
        ///     1. `bucket_id` is the buckets of interest
        ///     2. `writer` is the account which will receive write permission
        /// 
        /// **Events**:
        ///     1. None 
        /// 
        /// **Storage** (items are written to storage):
        ///     1. bucket_perms storage
        ///
        /// **Errors**: 
        ///     1. Transaction fails, if it is called by someone who's not a bucket owner or cluster manager
        /// 
        /// **Returns**:
        ///     Nothing
        #[ink(message)]
        pub fn bucket_set_writer_perm(&mut self, bucket_id: BucketId, writer: AccountId) -> () {
            self.message_grant_writer_permission(bucket_id, writer).unwrap()
        }

        /// **Description**:
        ///     Revoke permission for the writer of the bucket
        ///
        /// **Logic**:
        ///     1. Revoke permission for new writer via bucket_perms
        /// 
        /// **Permissions**:
        ///     1. Bucket owner
        ///     2. Cluster manager
        ///
        /// **Params**:
        ///     1. `bucket_id` is the buckets of interest
        ///     2. `writer` is the account for whome write permission will be revoked 
        /// 
        /// **Events**:
        ///     1. None 
        /// 
        /// **Storage** (items are written to storage):
        ///     1. bucket_perms storage
        ///
        /// **Errors**: 
        ///     1. Transaction fails, if it is called by someone who's not a bucket owner or cluster manager
        /// 
        /// **Returns**:
        ///     Nothing
        #[ink(message)]
        pub fn bucket_revoke_writer_perm(&mut self, bucket_id: BucketId, writer: AccountId) -> () {
            self.message_revoke_writer_permission(bucket_id, writer).unwrap()
        }

        /// **Description**:
        ///     Method retrieves bucket readers
        ///
        /// **Logic**:
        ///     1. Read from bucket_perms the permission for relevant bucket
        ///     2. Return vector of AccountIds who have permissions
        /// 
        /// **Permissions**:
        ///     1. Anyone
        ///
        /// **Params**:
        ///     1. `bucket_id` is the buckets of interest
        /// 
        /// **Events**:
        ///     1. None 
        /// 
        /// **Storage** (items are written to storage):
        ///     1. None
        ///     (**Note: we shall research how to combine this storage with bucket storage**)
        ///
        /// **Errors**: 
        ///     1. None
        /// 
        /// **Returns**:
        ///     Vector of AccountIds
        #[ink(message)]
        pub fn get_bucket_readers(&mut self, bucket_id: BucketId) -> Vec<AccountId> {
            self.message_get_bucket_readers(bucket_id).unwrap()
        }

        /// **Description**:
        ///     Set permission for the reader of the bucket
        ///
        /// **Logic**:
        ///     1. Grant permission for new reader via bucket_perms
        /// 
        /// **Permissions**:
        ///     1. Bucket owner
        ///     2. Cluster manager
        ///
        /// **Params**:
        ///     1. `bucket_id` is the buckets of interest
        ///     2. `reader` is the account which will receive read permission
        /// 
        /// **Events**:
        ///     1. None 
        /// 
        /// **Storage** (items are written to storage):
        ///     1. bucket_perms storage
        ///
        /// **Errors**: 
        ///     1. Transaction fails, if it is called by someone who's not a bucket owner or cluster manager
        /// 
        /// **Returns**:
        ///     Nothing
        #[ink(message)]
        pub fn bucket_set_reader_perm(&mut self, bucket_id: BucketId, reader: AccountId) -> () {
            self.message_grant_reader_permission(bucket_id, reader).unwrap()
        }

        /// **Description**:
        ///     Revoke permission for the reader of the bucket
        ///
        /// **Logic**:
        ///     1. Revoke permission for new reader via bucket_perms
        /// 
        /// **Permissions**:
        ///     1. Bucket owner
        ///     2. Cluster manager
        ///
        /// **Params**:
        ///     1. `bucket_id` is the buckets of interest
        ///     2. `reader` is the account for whome read permission will be revoked 
        /// 
        /// **Events**:
        ///     1. None 
        /// 
        /// **Storage** (items are written to storage):
        ///     1. bucket_perms storage
        ///
        /// **Errors**: 
        ///     1. Transaction fails, if it is called by someone who's not a bucket owner or cluster manager
        /// 
        /// **Returns**:
        ///     Nothing
        /// Revoke permission for the reader of the bucket
        #[ink(message)]
        pub fn bucket_revoke_reader_perm(&mut self, bucket_id: BucketId, reader: AccountId) -> () {
            self.message_revoke_reader_permission(bucket_id, reader).unwrap()
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
        /// **Description**:
        ///     Create a new cluster.
        ///
        /// **Logic**:
        ///     1. The caller will be its first manager.
        ///     2. The cluster is split in a number of vnodes. The vnodes are assigned to the given physical nodes in a round-robin. 
        /// 
        /// **Permissions**:
        ///     1. Only nodes of providers that trust the cluster manager can be used (see `node_trust_manager`). The assignment can be changed with the function `cluster_replace_node`.
        ///
        /// **Params**:
        ///     1. `cluster_params` is configuration used by clients and nodes. In particular, this describes the semantics of vnodes. See the [data structure of ClusterParams](https://docs.cere.network/ddc/protocols/contract-params-schema)
        /// 
        /// **Events**:
        ///     1. ClusterCreated(cluster id, manager id, cluster params)
        /// 
        /// **Storage** (items are written to storage):
        ///     1. cluster 
        ///     2. cluster_params
        ///     Note: there is currently no methods to delete cluster, hence these items will be stored during the lifetime of the contract
        /// 
        /// **Errors**: 
        ///     1. if caller is not trusted by nodes included in the cluster 
        ///     2. if cluster id is not the same as params id (may be triggered by faulty logic in other method)
        /// 
        /// **Returns**:
        ///     `cluster_id` - id of the newly created cluster.
       #[ink(message, payable)]
        pub fn cluster_create(&mut self, _unused: AccountId, vnode_count: u32, node_ids: Vec<NodeId>, cluster_params: ClusterParams) -> ClusterId {
            self.message_cluster_create(vnode_count, node_ids, cluster_params).unwrap()
        }

        /// **Description**:
        ///     Reserve more resources for the cluster from the free capacity of nodes.
        ///
        /// **Logic**:
        ///     1. The amount of resources is given per vnode (total resources will be `resource` times the number of vnodes).
        ///     2. Each node within cluster has to have higher available free resources then amount of Resource times number of vnodes per node
        /// 
        /// **Permissions**:
        ///     1. Cluster manager
        ///
        /// **Params**:
        ///     1. `cluster_id` is the id of cluster of interest
        ///     2. `amount` is the amount of resources to be reserved per vnode
        /// 
        /// **Events**:
        ///     1. ClusterReserveResource(cluster_id, resource)
        /// 
        /// **Storage** (items are written to storage):
        ///     1. cluster 
        ///     2. nodes
        ///     Note: there is currently no methods to free resources within cluster
        /// 
        /// **Errors**: 
        ///     1. if there are not enough free resources available within some node within a cluster
        /// 
        /// **Returns**:
        ///     Nothing
        #[ink(message)]
        pub fn cluster_reserve_resource(&mut self, cluster_id: ClusterId, amount: Resource) -> () {
            self.message_cluster_reserve_resource(cluster_id, amount).unwrap()
        }

        /// **Description**:
        ///     Re-assign a vnode to another physical node.
        ///
        /// **Logic**:
        ///     1. Free resource on the node where vnode was previously hosted 
        ///     2. Take resources from new node 
        ///     3. Update vnode
        ///     4. The cluster manager can only use nodes of providers that trust him (see `node_trust_manager`)
        /// 
        /// **Permissions**:
        ///     1. Cluster manager
        ///
        /// **Params**:
        ///     1. `cluster_id` is the id of cluster of interest
        ///     2. `vnode_i` is the index of vnode within the cluster
        ///     3. `new_node_id` is the id of node to which vnode is moved to
        /// 
        /// **Events**:
        ///     1. ClusterNodeReplaced(cluster_id, node_id: new_node_id, vnode_index)
        /// 
        /// **Storage** (items are written to storage):
        ///     1. cluster 
        ///     2. nodes
        ///    
        /// **Errors**: 
        ///     1. if there are not enough free resources available within some node within a cluster
        ///     2. vnode with this id should exist
        ///     3. if the tx is called by one who's not a cluster manager
        /// 
        /// **Returns**:
        ///     Nothing
        #[ink(message)]
        pub fn cluster_replace_node(&mut self, cluster_id: ClusterId, vnode_i: VNodeIndex, new_node_id: NodeId) -> () {
            self.message_cluster_replace_node(cluster_id, vnode_i, new_node_id).unwrap()
        }

        /// **Description**:
        ///     Trigger the distribution of revenues from the cluster to the providers.
        ///
        /// **Logic**:
        ///     1. Collect network and cluster management fee
        ///     2. Decrese undistributed revenue from the cluster
        ///     3. Payout tokens from SC to accounts who are providers of nodes, which host vnodes
        ///     (**Note (left by Aurel) TODO: set a maximum node count, or support paging. TODO: aggregate the payments per node_id or per provider_id.**)
        /// 
        /// **Permissions**:
        ///     1. Anyone
        ///
        /// **Params**:
        ///     1. `cluster_id` is the id of cluster of interest
        /// 
        /// **Events**:
        ///     1. ClusterDistributeRevenues(cluster_id, provider_id: node.provider_id)
        /// 
        /// **Storage** (items are written to storage):
        ///     1. cluster 
        ///     2. network_fee
        ///    
        /// **Errors**: 
        ///     1. None
        /// 
        /// **Returns**:
        ///     Nothing
        #[ink(message)]
        pub fn cluster_distribute_revenues(&mut self, cluster_id: ClusterId) {
            self.message_cluster_distribute_revenues(cluster_id).unwrap()
        }

        /// **Description**:
        ///     Change the `cluster_params`
        ///
        /// **Logic**:
        ///     1. Change params for cluster, which are configuration used by clients and nodes
        ///     2. See the [data structure of ClusterParams](https://docs.cere.network/ddc/protocols/contract-params-schema)
        /// 
        /// **Permissions**:
        ///     1. Cluster manager
        ///
        /// **Params**:
        ///     1. `cluster_id` is the id of cluster of interest
        ///     2. `cluster_params` is the configuration used by clients and nodes.
        /// 
        /// **Events**:
        ///     1. None
        /// 
        /// **Storage** (items are written to storage):
        ///     1. params
        ///    
        /// **Errors**: 
        ///     1. if params do not exist for this cluster id
        ///     3. if the tx is called by one who's not a cluster manager
        /// 
        /// **Returns**:
        ///     Nothing
        #[ink(message, payable)]
        pub fn cluster_change_params(&mut self, cluster_id: ClusterId, params: ClusterParams) {
            self.message_cluster_change_params(cluster_id, params).unwrap();
        }

        /// **Description**:
        ///     Get the current status of a cluster
        ///
        /// **Logic**:
        ///     1. Fetch cluster structure and cluster params and return them with id
        /// 
        /// **Permissions**:
        ///     1. Anyone
        ///
        /// **Params**:
        ///     1. `cluster_id` is the id of cluster of interest
        /// 
        /// **Events**:
        ///     1. None
        /// 
        /// **Storage** (items are written to storage):
        ///     1. None
        ///    
        /// **Errors**: 
        ///     1. None
        /// 
        /// **Returns**:
        ///     ClusterStatus(cluster_id, cluster, params)
        #[ink(message)]
        pub fn cluster_get(&self, cluster_id: ClusterId) -> Result<ClusterStatus> {
            self.message_cluster_get(cluster_id)
        }

        /// **Description**:
        ///     Get cluster list
        ///
        /// **Logic**:
        ///     1. Iterate through all clusters
        ///     2. The algorithm for paging is: start with `offset = 1` and `limit = 20`. The function returns a `(results, max_id)`. Call again with `offset += limit`, until `offset >= max_id`.
        ///     3. The optimal `limit` depends on the size of params
        ///     4. The results can be filtered by manager. Note that paging must still be completed fully
        ///     (**Note: same suggestion as for bucket list**)
        /// 
        /// **Permissions**:
        ///     1. Anyone
        ///
        /// **Params**:
        ///     1. `offset` skip a number of cluster
        ///     2. `limit` number of clusters per page
        ///     3. `filter_manager_id` optional filter to return only clusters where provided account is the manager
        /// 
        /// **Events**:
        ///     1. None
        /// 
        /// **Storage** (items are written to storage):
        ///     1. None
        ///    
        /// **Errors**: 
        ///     1. None
        /// 
        /// **Returns**:
        ///     Vec<ClusterStatus>, u32 where ClusterStatus(cluster_id, cluster, params) and u32 is the number of clusters
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

    /// The respective share of revenues of a CDN cluster for a provider was distributed.
    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct CdnClusterDistributeRevenues {
        #[ink(topic)]
        cluster_id: ClusterId,
        #[ink(topic)]
        provider_id: AccountId,
    }

    impl DdcBucket {
        /// **Description**:
        ///     Create a new CDN cluster
        ///
        /// **Logic**:
        ///     1. The caller will be its first manager
        ///     2. The CDN node ids are provided, which will form a cluster
        /// 
        /// **Permissions**:
        ///     1. Only CDN nodes of providers that trust the cluster manager can be used (see `cdn_node_trust_manager`)
        ///
        /// **Params**:
        ///     1. `cdn_node_ids` are the ids of nodes which will form a cluster
        /// 
        /// **Events**:
        ///     1. CdnClusterCreated(cluster_id, manager)
        /// 
        /// **Storage** (items are written to storage):
        ///     1. cdn_cluster 
        ///     Note: there is currently no methods to delete cluster, hence these items will be stored during the lifetime of the contract
        /// 
        /// **Errors**: 
        ///     1. if caller is not trusted by cdn nodes included in the cluster 
        /// 
        /// **Returns**:
        ///     `cluster_id` - id of the newly created CDN cluster
        #[ink(message, payable)]
        pub fn cdn_cluster_create(&mut self, cdn_node_ids: Vec<NodeId>) -> ClusterId {
            self.message_cdn_cluster_create(cdn_node_ids).unwrap()
        }

        /// **Description**:
        ///     Set rate for streaming (price per gb)
        /// 
        /// **Logic**:
        ///     1. The caller will be its first manager
        ///     2. The CDN node ids are provided, which will form a cluster
        /// 
        /// **Permissions**:
        ///     1. Only nodes of providers that trust the cluster manager can be used (see `cdn_node_trust_manager`)
        ///
        /// **Params**:
        ///     1. `cluster_id` are the id of CDN cluster of interest
        ///     2. `usd_per_gb` is the rate of USD per gb that the cluster will receive for streaming data (take the $ value and multiply by 10^10, i.e. 10 decimals)
        /// 
        /// **Events**:
        ///     1. None
        /// 
        /// **Storage** (items are written to storage):
        ///     1. cdn_cluster 
        /// 
        /// **Errors**: 
        ///     1. if caller is not the CDN cluster manager
        /// 
        /// **Returns**:
        ///     Nothing
        #[ink(message, payable)]
        pub fn cdn_set_rate(&mut self,  cluster_id: ClusterId, usd_per_gb: Balance) -> () {
            self.message_cdn_set_rate(cluster_id, usd_per_gb).unwrap()
        }
        
        /// **Description**:
        ///     Get rate for streaming (price per gb)
        /// 
        /// **Logic**:
        ///     1. Method reads the rate from cluster storage
        /// 
        /// **Permissions**:
        ///     1. Anyone
        ///
        /// **Params**:
        ///     1. `cluster_id` are the id of CDN cluster of interest
        /// 
        /// **Events**:
        ///     1. None
        /// 
        /// **Storage** (items are written to storage):
        ///     1. None 
        /// 
        /// **Errors**: 
        ///     1. if CDN cluster with provided id doesn't exist
        /// 
        /// **Returns**:
        ///     USD per gb (dollar value has 10 decimals)
        #[ink(message, payable)]
        pub fn cdn_get_rate(&self,  cluster_id: ClusterId) -> Balance {
            self.message_cdn_get_rate(cluster_id).unwrap()
        }

        /// **Description**:
        ///     Charge payments from users and allocate undistributed payments to CDN nodes
        /// 
        /// **Logic**:
        ///     1. Users are charged balance from their bonded balances within accounts
        ///     2. CDN nodes are credited with undistributed revenues
        ///     3. CDN cluster total revenues are increased,  which can be distributed between CDN node providers via method cdn_cluster_distribute_revenues
        ///     4. protocol fees are charged here
        ///     5. Bucket consumption cap is updated for public buckets
        /// 
        /// **Permissions**:
        ///     1. Validator
        ///
        /// **Params**:
        ///     1. `aggregates_accounts` are the tuples consisting of account ids and respective resources consumed
        ///     2. `aggregates_nodes` are the tuples consisting of node ids and respective resources consumed
        ///     3. `aggregates_buckets` are the tuples consisting of bucket ids and respective resources consumed
        /// 
        /// **Events**:
        ///     1. None
        /// 
        /// **Storage** (items are written to storage):
        ///     1. cdn_cluster 
        ///     2. cdn_nodes
        ///     3. buckets
        ///     4. accounts
        ///     5. protocol
        /// 
        /// **Errors**: 
        ///     1. if caller is not the validator
        /// 
        /// **Returns**:
        ///     Nothing
        #[ink(message)]
        pub fn cdn_cluster_put_revenue(&mut self, cluster_id: ClusterId, aggregates_accounts: Vec<(AccountId, u128)>, aggregates_nodes: Vec<(u32, u128)>, aggregates_buckets: Vec<(BucketId, Resource)>, era: u64) -> () {
            self.message_cdn_cluster_put_revenue(cluster_id, aggregates_accounts, aggregates_nodes, aggregates_buckets, era).unwrap()
        }

        /// **Description**:
        ///     Trigger the distribution of revenues from the cluster to the CDN node providers
        /// 
        /// **Logic**:
        ///     1. Undistributed CDN Node payments will be trasnferred 
        ///     2. CDN cluster revenue will decrease
        ///     3. Management fees are charged here
        /// 
        /// **Permissions**:
        ///     1. Anyone
        ///
        /// **Params**:
        ///     1. `cluster_id` is the id of CDN cluster of interest
        ///     
        /// **Events**:
        ///     1. ClusterDistributeRevenues(cluster_id, provider_id: node.provider_id)
        /// 
        /// **Storage** (items are written to storage):
        ///     1. cdn_cluster 
        ///     2. cdn_nodes
        /// 
        /// **Errors**: 
        ///     1. if caller is not the validator
        /// 
        /// **Returns**:
        ///     Nothing
        #[ink(message)]
        pub fn cdn_cluster_distribute_revenues(&mut self, cluster_id: ClusterId) {
            self.message_cdn_cluster_distribute_revenues(cluster_id).unwrap()
        }

         /// **Description**:
        ///     Get the current status of a CDN cluster
        ///
        /// **Logic**:
        ///     1. Fetch cluster structure and return ir with id
        /// 
        /// **Permissions**:
        ///     1. Anyone
        ///
        /// **Params**:
        ///     1. `cluster_id` is the id of CDN cluster of interest
        /// 
        /// **Events**:
        ///     1. None
        /// 
        /// **Storage** (items are written to storage):
        ///     1. None
        ///    
        /// **Errors**: 
        ///     1. None
        /// 
        /// **Returns**:
        ///     CdnClusterStatus(cluster_id, cluster)
        #[ink(message)]
        pub fn cdn_cluster_get(&self, cluster_id: ClusterId) -> Result<CdnClusterStatus> {
            self.message_cdn_cluster_get(cluster_id)
        }

        /// **Description**:
        ///     Get CDN cluster list
        ///
        /// **Logic**:
        ///     1. Iterate through all CDN clusters
        ///     2. The algorithm for paging is: start with `offset = 1` and `limit = 20`. The function returns a `(results, max_id)`. Call again with `offset += limit`, until `offset >= max_id`.
        ///     3. The optimal `limit` depends on the size of params
        ///     4. The results can be filtered by manager. Note that paging must still be completed fully
        ///     (**Note: same suggestion as for bucket list**)
        /// 
        /// **Permissions**:
        ///     1. Anyone
        ///
        /// **Params**:
        ///     1. `offset` skip a number of cluster
        ///     2. `limit` number of clusters per page
        ///     3. `filter_manager_id` optional filter to return only clusters where provided account is the manager
        /// 
        /// **Events**:
        ///     1. None
        /// 
        /// **Storage** (items are written to storage):
        ///     1. None
        ///    
        /// **Errors**: 
        ///     1. None
        /// 
        /// **Returns**:
        ///     Vec<CdnClusterStatus>, u32 where CdnClusterStatus(cluster_id, cluster) and u32 is the number of clusters
        #[ink(message)]
        pub fn cdn_cluster_list(&self, offset: u32, limit: u32, filter_manager_id: Option<AccountId>) -> (Vec<CdnClusterStatus>, u32) {
            self.message_cdn_cluster_list(offset, limit, filter_manager_id)
        }
    }
    // ---- End CDN Cluster ----

    // ---- Committer ----

    impl DdcBucket {
        /// **Description**:
        ///     CDN node operator sets the commit for current era
        /// 
        /// **Logic**:
        ///     1. Check if commits exist for provided account
        ///     2. If not, create an empty vector of commits
        ///     3. If commits for provided node exists -> remove it from the vector
        ///     4. Push the latest commit to the vector
        /// 
        /// **Permissions**:
        ///     1. **Add permission: CDN owner or CDN cluster manager**
        ///
        /// **Params**:
        ///     1. `cdn_owner` is the account id of CDN cluster of interest (**Should be optimised and omitted**)
        ///     2. `node_id` is the CDN node id of interest
        ///     3. `commit` is the hash representing the root of the Merkle Tree, which consists of all logs for latest era
        ///     
        /// **Events**:
        ///     1. None
        /// 
        /// **Storage** (items are written to storage):
        ///     1. committer
        /// 
        /// **Errors**: 
        ///     1. None
        /// 
        /// **Returns**:
        ///     Nothing
        #[ink(message)]
        pub fn set_commit(&mut self, cdn_owner: AccountId, node_id: NodeId, commit: Commit) {
            self.message_set_commit(cdn_owner, node_id, commit);
        }

        /// **Description**:
        ///     Return the latest commits submitted by CDN node operator
        /// 
        /// **Logic**:
        ///     1. Fetch from committer_store the mapping representing commits
        ///     2. Return commits related to provided account
        /// 
        /// **Permissions**:
        ///     1. None
        ///
        /// **Params**:
        ///     1. `cdn_owner` is the account id of CDN cluster of interest (**Should be optimised and omitted**)
        ///     2. `node_id` is the CDN node id of interest
        ///     3. `commit` is the structure, which stores the hash representing the root of the Merkle Tree, which consists of all logs for latest era, as well as timestamps and other data
        ///     
        /// **Events**:
        ///     1. None
        /// 
        /// **Storage** (items are written to storage):
        ///     1. None
        /// 
        /// **Errors**: 
        ///     1. None
        /// 
        /// **Returns**:
        ///     Vec<(NodeId, Commit)> where the tuple represents the id of the CDN node, and the commit structure
        #[ink(message)]
        pub fn get_commit(&self, cdn_owner: AccountId) -> Vec<(NodeId, Commit)> {
            self.message_get_commit(cdn_owner)
        }

        /// **Description**:
        ///     Return last era and timestamp validated for CDN node
        /// 
        /// **Logic**:
        ///     1. Fetch from committer_store the mapping representing validated commits
        ///     2. Return data for the last validated commit for provided CDN Node
        /// 
        /// **Permissions**:
        ///     1. None
        ///
        /// **Params**:
        ///     1. `node_id` is the CDN node id of interest
        ///     
        /// **Events**:
        ///     1. None
        /// 
        /// **Storage** (items are written to storage):
        ///     1. None
        /// 
        /// **Errors**: 
        ///     1. None
        /// 
        /// **Returns**:
        ///     Era and timestamp
        #[ink(message)]
        pub fn get_validated_commit(&self, node: NodeId) -> EraAndTimestamp {
            self.message_get_validated_commit(node)
        }

        /// **Description**:
        ///     Set the new configs for era
        /// 
        /// **Logic**:
        ///     1. Era configuration is an important part of the SC
        ///     2. CDN Node managers will synchronise with it to submit commits
        ///     3. Validators will synchronise and wait for validation phase
        /// 
        /// **Permissions**:
        ///     1. (**Allow only specific role to adjust this configuration**)
        ///
        /// **Params**:
        ///     1. `era_config` is the configuration of the era for the SC, which consists of start timestamp, interval, commit & validation durations
        ///     
        /// **Events**:
        ///     1. None
        /// 
        /// **Storage** (items are written to storage):
        ///     1. committer
        /// 
        /// **Errors**: 
        ///     1. None
        /// 
        /// **Returns**:
        ///     Nothing
        #[ink(message)]
        pub fn set_era(&mut self, era_config: EraConfig) -> () {
            self.message_set_era(era_config).unwrap();
        }
    
        /// **Description**:
        ///     Return current status of an era
        /// 
        /// **Logic**:
        ///     1. Current era phase is calculated
        ///     2. Previous era's timestamps are calculated
        /// 
        /// **Permissions**:
        ///     1. Anyone
        ///
        /// **Params**:
        ///     1. None
        ///     
        /// **Events**:
        ///     1. None
        /// 
        /// **Storage** (items are written to storage):
        ///     1. None
        /// 
        /// **Errors**: 
        ///     1. None
        /// 
        /// **Returns**:
        ///     EraStatus(current_era, current_phase, previous_era, prev_era_from_timestamp, prev_era_to_timestamp)
        #[ink(message)]
        pub fn get_era(&self) -> EraStatus {
            self.message_get_era()
        }

        /// **Description**:
        ///     Return current era settings
        /// 
        /// **Logic**:
        ///     1. Simply return provided era settings
        ///     2. This data can be used to predict the future phases without fetching data with `get_era` method as Era Configuration is not expected to change often
        /// 
        /// **Permissions**:
        ///     1. Anyone
        ///
        /// **Params**:
        ///     1. None
        ///     
        /// **Events**:
        ///     1. None
        /// 
        /// **Storage** (items are written to storage):
        ///     1. None
        /// 
        /// **Errors**: 
        ///     1. None
        /// 
        /// **Returns**:
        ///     EraConfig(start, interval, commit_duration, validation_duration)
        #[ink(message)]
        pub fn get_era_settings(&self) -> EraConfig {
            self.message_get_era_settings()
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
        pub fn cdn_node_create(&mut self, node_params: Params) -> NodeId {
            self.message_cdn_node_create(node_params).unwrap()
        }


        /// Change the `node_params`, which is configuration used by clients and nodes.
        ///
        /// See the [data structure of NodeParams](https://docs.cere.network/ddc/protocols/contract-params-schema)
        #[ink(message, payable)]
        pub fn cdn_node_change_params(&mut self, node_id: NodeId, params: NodeParams) {
            self.message_cdn_node_change_params(node_id, params).unwrap();
        }

        /// Get the current state of the cdn node
        #[ink(message)]
        pub fn cdn_node_get(&self, node_id: NodeId) -> Result<CdnNodeStatus> {
            self.message_cdn_node_get(node_id)
        }

        /// Iterate through all nodes.
        ///
        /// The algorithm for paging is: start with `offset = 1` and `limit = 20`. The function returns a `(results, max_id)`. Call again with `offset += limit`, until `offset >= max_id`.
        /// The optimal `limit` depends on the size of params.
        ///
        /// The results can be filtered by owner. Note that paging must still be completed fully.
        #[ink(message)]
        pub fn cdn_node_list(&self, offset: u32, limit: u32, filter_provider_id: Option<AccountId>) -> (Vec<CdnNodeStatus>, u32) {
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

    // ---- Protocol ----

    impl DdcBucket {
        /// Get the Fee Percentage Basis Points that will be charged by the protocol
        #[ink(message)]
        pub fn get_fee_bp(&self) -> u32 {
            self.message_get_fee_bp()
        }

        /// Return the last commit submitted by CDN node operator
        #[ink(message)]
        pub fn set_fee_bp(&mut self, fee_bp: u32) -> () {
            self.message_set_fee_bp(fee_bp).unwrap();
        }

        /// Return fees accumulated by the protocol
        #[ink(message)]
        pub fn get_protocol_revenues(&self) -> Cash {
            self.message_get_fee_revenues()
        }

        /// Pay the revenues accumulated by the protocol
        #[ink(message)]
        pub fn protocol_withdraw_revenues(&mut self, amount: u128) -> () {
            self.message_withdraw_revenues(amount).unwrap();
        }
    }
    // ---- End Protocol ----

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

        /// As user, bond some amount of tokens from the withdrawable balance. These funds will be used to pay for CDN node service.
        #[ink(message, payable)]
        pub fn account_bond(&mut self, bond_amount: Balance) -> () {
            self.message_account_bond(bond_amount).unwrap()
        }

        /// As user, unbond a specified amount of tokens. The tokens will be locked for some time, as defined by contract owner.
        #[ink(message, payable)]
        pub fn account_unbond(&mut self, amount_to_unbond: Cash) -> () {
            self.message_account_unbond(amount_to_unbond).unwrap()
        }

        /// As user, move the unbonded tokens back to withdrawable balance state. 
        /// 
        /// This can be triggered after unbonded_timestamp
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

    // ---- Accounts ----
    impl DdcBucket {
        /// Get all Account IDs stored in the SC
        #[ink(message, payable)]
        pub fn get_accounts(&self) -> Vec<AccountId> {
            self.message_get_accounts()
        }
    }
    // ---- End Accounts ----


    // ---- Utils ----
    /// One token with 10 decimals.
    pub const TOKEN: Balance = 10_000_000_000;
    pub const DEFAULT_BASIS_POINTS: u32 = 500;

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
