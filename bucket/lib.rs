//! The DDC smart contract implementing cluster-based services, and bucket-based subscriptions.

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
    use committer::store::*;
    use ink_storage::traits::SpreadAllocate;
    use node::{entity::*, store::*};
    use perm::store::*;
    use topology::store::*;

    use crate::ddc_bucket::cdn_node::store::CdnNodeStore;
    use crate::ddc_bucket::perm::entity::Permission;

    use self::cdn_node::entity::{CdnNodeInfo, CdnNodeKey, CdnNodeParams};
    use self::account::entity::Account;
    use self::protocol::store::{ProtocolStore, NetworkFeeConfig};
    use self::topology::store::TopologyStore;

    pub mod account;
    pub mod admin;
    pub mod bucket;
    pub mod cash;
    pub mod cdn_node;
    pub mod cluster;
    pub mod committer;
    pub mod currency;
    pub mod flow;
    pub mod node;
    pub mod perm;
    pub mod protocol;
    pub mod schedule;
    pub mod topology;

    // ---- Global state ----
    /// The main DDC smart contract.
    #[ink(storage)]
    #[derive(SpreadAllocate, Default)]
    pub struct DdcBucket {
        perms: PermStore,
        buckets: BucketStore,
        clusters: ClusterStore,
        cdn_nodes: CdnNodeStore,
        nodes: NodeStore,
        topology: TopologyStore,
        accounts: AccountStore,
        committer: CommitterStore,
        protocol: ProtocolStore,
    }

    impl DdcBucket {
        /// Create a new contract.
        ///
        /// The caller will be admin of the contract.
        #[ink(constructor)]
        pub fn new() -> Self {
            ink_lang::utils::initialize_contract(|contract: &mut Self| {
                let admin = Self::env().caller();
                // Make the creator of this contract a super-admin.
                contract.perms.grant_permission(admin, Permission::SuperAdmin);                
                contract.committer.init(admin);
                contract.protocol.init(admin, DEFAULT_PROTOCOL_FEE_BP);
            })
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

    /// The availiablity of the bucket was updated.
    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct BucketAvailabilityUpdated {
        #[ink(topic)]
        bucket_id: BucketId,
        #[ink(topic)]
        public_availability: bool,
    }

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct BucketParamsSet {
        #[ink(topic)]
        bucket_id: BucketId,
        bucket_params: BucketParams,
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
        pub fn bucket_create(
            &mut self,
            bucket_params: BucketParams,
            cluster_id: ClusterId,
            owner_id: Option<AccountId>,
        ) -> BucketId {
            self.message_bucket_create(bucket_params, cluster_id, owner_id)
                .unwrap()
        }

        /// Change owner of the bucket
        ///
        /// Provide the account of new owner
        #[ink(message, payable)]
        pub fn bucket_change_owner(&mut self, bucket_id: BucketId, owner_id: AccountId) -> () {
            self.message_bucket_change_owner(bucket_id, owner_id)
                .unwrap()
        }

        /// Allocate some resources of a cluster to a bucket.
        ///
        /// The amount of resources is given per vnode (total resources will be `resource` times the number of vnodes).
        #[ink(message)]
        pub fn bucket_alloc_into_cluster(&mut self, bucket_id: BucketId, resource: Resource) -> () {
            self.message_bucket_alloc_into_cluster(bucket_id, resource)
                .unwrap()
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
            self.message_bucket_change_params(bucket_id, params)
                .unwrap();
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
        pub fn bucket_list(
            &self,
            offset: u32,
            limit: u32,
            filter_owner_id: Option<AccountId>,
        ) -> (Vec<BucketStatus>, u32) {
            self.message_bucket_list(offset, limit, filter_owner_id)
        }

        /// Iterate through all buckets and return only those owned by owner
        ///
        /// This method returns bucket struct, not the status
        #[ink(message)]
        pub fn bucket_list_for_account(&self, owner_id: AccountId) -> Vec<Bucket> {
            self.message_bucket_list_for_account(owner_id)
        }

        /// Set availiablity of the bucket
        #[ink(message)]
        pub fn bucket_set_availability(
            &mut self,
            bucket_id: BucketId,
            public_availability: bool,
        ) -> () {
            self.message_bucket_set_availability(bucket_id, public_availability)
                .unwrap()
        }

        /// Set max resource cap to be charged by CDN for public bucket
        #[ink(message)]
        pub fn bucket_set_resource_cap(
            &mut self,
            bucket_id: BucketId,
            new_resource_cap: Resource,
        ) -> () {
            self.message_bucket_set_resource_cap(bucket_id, new_resource_cap)
                .unwrap()
        }

        /// Set permission for the reader of the bucket
        #[ink(message)]
        pub fn get_bucket_writers(&mut self, bucket_id: BucketId) -> Vec<AccountId> {
            self.message_get_bucket_writers(bucket_id).unwrap()
        }

        /// Set permission for the writer of the bucket
        #[ink(message)]
        pub fn bucket_set_writer_perm(&mut self, bucket_id: BucketId, writer: AccountId) -> () {
            self.message_grant_writer_permission(bucket_id, writer)
                .unwrap()
        }

        /// Revoke permission for the writer of the bucket
        #[ink(message)]
        pub fn bucket_revoke_writer_perm(&mut self, bucket_id: BucketId, writer: AccountId) -> () {
            self.message_revoke_writer_permission(bucket_id, writer)
                .unwrap()
        }

        /// Set permission for the reader of the bucket
        #[ink(message)]
        pub fn get_bucket_readers(&mut self, bucket_id: BucketId) -> Vec<AccountId> {
            self.message_get_bucket_readers(bucket_id).unwrap()
        }

        /// Set permission for the reader of the bucket
        #[ink(message)]
        pub fn bucket_set_reader_perm(&mut self, bucket_id: BucketId, reader: AccountId) -> () {
            self.message_grant_reader_permission(bucket_id, reader)
                .unwrap()
        }

        /// Revoke permission for the reader of the bucket
        #[ink(message)]
        pub fn bucket_revoke_reader_perm(&mut self, bucket_id: BucketId, writer: AccountId) -> () {
            self.message_revoke_reader_permission(bucket_id, writer)
                .unwrap()
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
    pub struct ClusterNodeAdded {
        #[ink(topic)]
        cluster_id: ClusterId,
        #[ink(topic)]
        node_key: NodeKey,
    }

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct ClusterNodeRemoved {
        #[ink(topic)]
        cluster_id: ClusterId,
        #[ink(topic)]
        node_key: NodeKey,
    }

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct ClusterCdnNodeAdded {
        #[ink(topic)]
        cluster_id: ClusterId,
        #[ink(topic)]
        cdn_node_key: CdnNodeKey,
    }

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct ClusterCdnNodeRemoved {
        #[ink(topic)]
        cluster_id: ClusterId,
        #[ink(topic)]
        cdn_node_key: CdnNodeKey,
    }

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct ClusterParamsSet {
        #[ink(topic)]
        cluster_id: ClusterId,
        cluster_params: ClusterParams,
    }


    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct ClusterRemoved {
        #[ink(topic)]
        cluster_id: ClusterId,
    }

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct ClusterNodeStatusSet {
        #[ink(topic)]
        node_key: NodeKey,
        #[ink(topic)]
        cluster_id: ClusterId,
        status: NodeStatusInCluster
    }

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct ClusterCdnNodeStatusSet {
        #[ink(topic)]
        cdn_node_key: CdnNodeKey,
        #[ink(topic)]
        cluster_id: ClusterId,
        status: NodeStatusInCluster
    }

    /// A vnode was re-assigned to new node.
    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct ClusterNodeReplaced {
        #[ink(topic)]
        cluster_id: ClusterId,
        #[ink(topic)]
        node_key: NodeKey,
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

        /// Creates a cluster of Storage nodes and CDN nodes.
        ///
        /// This endpoint creates a cluster of Storage nodes and CDN nodes with specific parameters.
        /// The caller will be the cluster manager (cluster owner). In order to add a Storage or CDN node, the manager must be authorized by the node owner using the `trust_manager` endpoint or be the node owner.
        ///
        /// # Parameters
        ///
        /// * `cluster_params` - [Cluster parameters](https://docs.cere.network/ddc/protocols/contract-params-schema#cluster-parameters) in protobuf format.
        ///
        /// # Output
        ///
        /// Returns ID of the created cluster.
        ///
        /// # Events
        ///
        /// * `ClusterCreated` event on successful cluster creation.
        ///
        /// # Errors
        ///
        /// * `InvalidClusterParams` error if there is an invalid cluster parameter. 
        #[ink(message, payable)]
        pub fn cluster_create(
            &mut self,
            cluster_params: ClusterParams,
        ) -> Result<ClusterId> {
            self.message_cluster_create(cluster_params)
        }

        /// Adds a Storage node to the targeting cluster.
        ///
        /// This endpoint adds a physical Storage node along with its virtual nodes to the targeting cluster.
        /// Virtual nodes determines a token (position) on the ring in terms of Consistent Hashing.
        /// The Storage node can be added to the cluster by cluster manager only.
        ///
        /// # Parameters
        ///
        /// * `cluster_id` - ID of the targeting cluster.
        /// * `node_key` - Public Key associated with the Storage node.
        /// * `v_nodes` - List of tokens (positions) related to the Storage node.
        ///
        /// # Output
        ///
        /// Returns nothing.
        ///
        /// # Events
        ///
        /// * `ClusterNodeAdded` event on successful Storage node addition.
        ///
        /// # Errors
        ///
        /// * `ClusterDoesNotExist` error if the cluster does not exist.
        /// * `OnlyTrustedClusterManager` error if the caller is not a trusted cluster manager.
        /// * `NodeDoesNotExist` error if the adding Storage node does not exist.
        /// * `NodeIsAddedToCluster(ClusterId)` error if the adding Storage node is already added to this or another cluster.
        /// * `AtLeastOneVNodeHasToBeAssigned(ClusterId, NodeKey)` error if there is a Storage node without any virtual nodes in the cluster.
        /// * `VNodesSizeExceedsLimit` error if virtual nodes length exceeds storage capacity.
        #[ink(message, payable)]
        pub fn cluster_add_node(
            &mut self,
            cluster_id: ClusterId,
            node_key: NodeKey,
            v_nodes: Vec<VNodeToken>,
        ) -> Result<()> {
            self.message_cluster_add_node(cluster_id, node_key, v_nodes)
        }

        /// Removes a Storage node from the targeting cluster.
        ///
        /// This endpoint removes a physical Storage node along with its virtual nodes from the targeting cluster.
        /// The Storage node can be removed from the cluster either by cluster manager or by the node owner.
        ///
        /// # Parameters
        ///
        /// * `cluster_id` - ID of the targeting cluster.
        /// * `node_key` - Public Key associated with the Storage node.
        ///
        /// # Output
        ///
        /// Returns nothing.
        ///
        /// # Events
        ///
        /// * `ClusterNodeRemoved` event on successful Storage node removal.
        ///
        /// # Errors
        ///
        /// * `ClusterDoesNotExist` error if the cluster does not exist.
        /// * `OnlyClusterManagerOrNodeProvider` error if the caller is not the cluster manager or node owner.
        /// * `NodeDoesNotExist` error if the removing Storage node does not exist.
        /// * `NodeIsNotAddedToCluster(ClusterId)` error if the removing Storage node is not in this cluster.
        #[ink(message)]
        pub fn cluster_remove_node(
            &mut self,
            cluster_id: ClusterId,
            node_key: NodeKey,
        ) -> Result<()> {
            self.message_cluster_remove_node(cluster_id, node_key)
        }


        /// Reasignes existing virtual nodes in the targeting cluster.
        ///
        /// This endpoint reasignes existing virtual nodes to another physical Storage node within the same cluster.
        /// All specifying virtual nodes must pre-exist in the cluster and the new physical Storage node must be added to the cluster preliminary.
        ///
        /// # Parameters
        ///
        /// * `cluster_id` - ID of the targeting cluster.
        /// * `v_nodes` - List of tokens (positions) to reasign for the new physical Storage node.
        /// * `new_node_key` - Public Key associated with the Storage node that is being reasigned to the specified virtual nodes.
        ///
        /// # Output
        ///
        /// Returns nothing.
        ///
        /// # Events
        ///
        /// * `ClusterNodeReplaced` event on successful virtual node reassignment.
        ///
        /// # Errors
        ///
        /// * `ClusterDoesNotExist` error if the cluster does not exist.
        /// * `OnlyClusterManager` error if the caller is not the cluster manager.
        /// * `NodeDoesNotExist` error if the new Storage node does not exist.
        /// * `NodeIsNotAddedToCluster(ClusterId)` error if the new Storage node is not added to this cluster.
        /// * `NodeIsAddedToCluster(ClusterId)` error if the new Storage node is in another cluster.
        /// * `VNodeIsNotAssignedToNode(ClusterId, VNodeToken)` error if the there is some virtual node that is being reasigned, but this virtual node is not assigned to any physical node.
        /// * `VNodeIsAlreadyAssignedToNode(NodeKey)` - error if there is some virtual node that is already assigned to other physical node within the same cluster.
        /// * `AtLeastOneVNodeHasToBeAssigned(ClusterId, NodeKey)` error if there is a Storage node without any virtual nodes in the cluster.
        /// * `VNodesSizeExceedsLimit` error if virtual nodes length exceeds storage capacity.
        #[ink(message)]
        pub fn cluster_replace_node(
            &mut self,
            cluster_id: ClusterId,
            v_nodes: Vec<VNodeToken>,
            new_node_key: NodeKey,
        ) -> Result<()> {
            self.message_cluster_replace_node(
                cluster_id, 
                v_nodes, 
                new_node_key
            )
        }

        /// Adds a CDN node to the targeting cluster.
        ///
        /// This endpoint adds a CDN node to the targeting cluster.
        /// The CDN node can be added to the cluster by cluster manager only.
        ///
        /// # Parameters
        ///
        /// * `cluster_id` - ID of the targeting cluster.
        /// * `cdn_node_key` - Public Key associated with the CDN node.
        ///
        /// # Output
        ///
        /// Returns nothing.
        ///
        /// # Events
        ///
        /// * `ClusterCdnNodeAdded` event on successful CDN node addition.
        ///
        /// # Errors
        ///
        /// * `ClusterDoesNotExist` error if the cluster does not exist.
        /// * `OnlyTrustedClusterManager` error if the caller is not a trusted cluster manager.
        /// * `CdnNodeDoesNotExist` error if the adding CDN node does not exist.
        /// * `CdnNodeIsAddedToCluster(ClusterId)` error if the adding CDN node is already added to this or another cluster.
        #[ink(message, payable)]
        pub fn cluster_add_cdn_node(
            &mut self,
            cluster_id: ClusterId,
            cdn_node_key: CdnNodeKey,
        ) -> Result<()> {
            self.message_cluster_add_cdn_node(cluster_id, cdn_node_key)
        }

        /// Removes a CDN node from the targeting cluster.
        ///
        /// This endpoint removes a CDN node the targeting cluster.
        /// The CDN node can be removed from the cluster either by cluster manager or by the node owner.
        ///
        /// # Parameters
        ///
        /// * `cluster_id` - ID of the targeting cluster.
        /// * `cdn_node_key` - Public Key associated with the CDN node.
        ///
        /// # Output
        ///
        /// Returns nothing.
        ///
        /// # Events
        ///
        /// * `ClusterCdnNodeRemoved` event on successful CDN node removal.
        ///
        /// # Errors
        ///
        /// * `ClusterDoesNotExist` error if the cluster does not exist.
        /// * `OnlyClusterManagerOrCdnNodeProvider` error if the caller is not the cluster manager or node owner.
        /// * `CdnNodeDoesNotExist` error if the removing CDN node does not exist.
        /// * `CdnNodeIsNotAddedToCluster(ClusterId)` error if the removing CDN node is not in this cluster.
        #[ink(message)]
        pub fn cluster_remove_cdn_node(
            &mut self,
            cluster_id: ClusterId,
            cdn_node_key: CdnNodeKey,
        ) -> Result<()> {
            self.message_cluster_remove_cdn_node(cluster_id, cdn_node_key)
        }
        
        /// Sets parameters for the targeting cluster.
        ///
        /// This enpoint updates [cluster parameters](https://docs.cere.network/ddc/protocols/contract-params-schema#cluster-parameters) in protobuf format. 
        /// All cluster parameters must be specified as the endpoint works using SET approach.
        ///
        /// # Parameters
        ///
        /// * `cluster_id` - ID of the targeting cluster.
        /// * `cluster_params` - [Cluster parameters](https://docs.cere.network/ddc/protocols/contract-params-schema#cluster-parameters) in protobuf format.
        ///
        /// # Output
        ///
        /// Returns nothing.
        ///
        /// # Events
        ///
        /// * `ClusterParamsSet` event on successful cluster params setting.
        ///
        /// # Errors
        ///
        /// * `OnlyClusterManager` error if the caller is not the cluster manager.
        /// * `ClusterDoesNotExist` error if the cluster does not exist.
        #[ink(message, payable)]
        pub fn cluster_set_params(
            &mut self, 
            cluster_id: ClusterId, 
            cluster_params: ClusterParams
        ) -> Result<()> {
            self.message_cluster_set_params(cluster_id, cluster_params)
        }

        /// Removes a cluster.
        ///
        /// This enpoint removes the cluster if it does not contain any nodes.
        /// Only an empty cluster can be removed.
        ///
        /// # Parameters
        ///
        /// * `cluster_id` - ID of the targeting cluster.
        ///
        /// # Output
        ///
        /// Returns nothing.
        ///
        /// # Events
        ///
        /// * `ClusterRemoved` event on successful cluster removal.
        ///
        /// # Errors
        ///
        /// * `OnlyClusterManager` error if the caller is not the cluster manager.
        /// * `ClusterDoesNotExist` error if the cluster does not exist.
        /// * `ClusterIsNotEmpty` error if the removing cluster contains some Storage or CDN nodes.
        #[ink(message)]
        pub fn cluster_remove(
            &mut self, 
            cluster_id: ClusterId, 
        ) -> Result<()> {
            self.message_cluster_remove(cluster_id)
        }

        /// Changes Storage node status.
        ///
        /// This enpoint changes Storage node status in a cluster.
        ///
        /// # Parameters
        ///
        /// * `cluster_id` - ID of the targeting cluster.
        /// * `node_key` - Public Key associated with the Storage node.
        /// * `status` - Status for the targeting Storage node, can be one of the following: ACTIVE, ADDING, DELETING, OFFLINE.
        ///
        /// # Output
        ///
        /// Returns nothing.
        ///
        /// # Events
        ///
        /// * `ClusterNodeStatusSet` event on successful Storage status change.
        ///
        /// # Errors
        ///
        /// * `OnlyClusterManager` error if the caller is not the cluster manager.
        /// * `ClusterDoesNotExist` error if the cluster does not exist.
        /// * `NodeIsNotAddedToCluster(ClusterId)` error if the Storage node is not in this cluster.
        #[ink(message)]
        pub fn cluster_set_node_status(
            &mut self, 
            cluster_id: ClusterId,
            node_key: NodeKey, 
            status_in_cluster: NodeStatusInCluster
        ) -> Result<()> {
            self.message_cluster_set_node_status(
                cluster_id, 
                node_key, 
                status_in_cluster
            )
        }

        /// Changes CDN node status.
        ///
        /// This enpoint changes CDN node status in a cluster.
        ///
        /// # Parameters
        ///
        /// * `cluster_id` - ID of the targeting cluster.
        /// * `cdn_node_key` - Public Key associated with the CDN node.
        /// * `status` - Status for the targeting CDN node, can be one of the following: ACTIVE, ADDING, DELETING, OFFLINE.
        ///
        /// # Output
        ///
        /// Returns nothing.
        ///
        /// # Events
        ///
        /// * `ClusterCdnNodeStatusSet` event on successful CDN status change.
        ///
        /// # Errors
        ///
        /// * `OnlyClusterManager` error if the caller is not the cluster manager.
        /// * `ClusterDoesNotExist` error if the cluster does not exist.
        /// * `CdnNodeIsNotAddedToCluster(ClusterId)` error if the CDN node is not in this cluster.
        #[ink(message)]
        pub fn cluster_set_cdn_node_status(
            &mut self, 
            cluster_id: ClusterId,
            cdn_node_key: CdnNodeKey, 
            status_in_cluster: NodeStatusInCluster
        ) -> Result<()> {
            self.message_cluster_set_cdn_node_status(
                cluster_id, 
                cdn_node_key, 
                status_in_cluster
            )
        }

        /// Gets a cluster.
        ///
        /// This enpoint gets the targeting cluster along with its parameters, Storage and CDN nodes.
        ///
        /// # Parameters
        ///
        /// * `cluster_id` - ID of the targeting cluster.
        ///
        /// # Output
        ///
        /// Returns `ClusterInfo` data transfer object.
        ///
        /// # Errors
        ///
        /// * `ClusterDoesNotExist` error if the cluster does not exist.
        #[ink(message)]
        pub fn cluster_get(
            &self, 
            cluster_id: ClusterId
        ) -> Result<ClusterInfo> {
            self.message_cluster_get(cluster_id)
        }

        /// Gets a paginated list of clusters.
        ///
        /// This enpoint gets a paginated list of clusters along with their parameters, Storage and CDN nodes.
        /// The algorithm for paging is: start with `offset = 1` and `limit = 20`. The function returns a `(results, max_id)`. Call again with `offset += limit`, until `offset >= max_id`.
        /// The optimal `limit` depends on the size of params.
        ///
        /// # Parameters
        ///
        /// * `offset` - starting offset.
        /// * `limit` - page limit.
        /// * `filter_manager_id` - optional filter by cluster manager.
        ///
        /// # Errors
        ///
        /// No errors. In case a pagination param is out of bounds, an empty list will be returned.
        #[ink(message)]
        pub fn cluster_list(
            &self,
            offset: u32,
            limit: u32,
            filter_manager_id: Option<AccountId>,
        ) -> (Vec<ClusterInfo>, u32) {
            self.message_cluster_list(offset, limit, filter_manager_id)
        }

        /// As manager, reserve more resources for the cluster from the free capacity of nodes.
        ///
        /// The amount of resources is given per vnode (total resources will be `resource` times the number of vnodes).
        #[ink(message)]
        pub fn cluster_reserve_resource(&mut self, cluster_id: ClusterId, amount: Resource) -> () {
            self.message_cluster_reserve_resource(cluster_id, amount)
                .unwrap()
        }

        /// Trigger the distribution of revenues from the cluster to the providers.
        #[ink(message)]
        pub fn cluster_distribute_revenues(&mut self, cluster_id: ClusterId) {
            self.message_cluster_distribute_revenues(cluster_id)
                .unwrap()
        }

        /// Set rate for streaming (price per gb)
        #[ink(message, payable)]
        pub fn cdn_set_rate(&mut self, cluster_id: ClusterId, usd_per_gb: Balance) -> () {
            self.message_cdn_set_rate(cluster_id, usd_per_gb).unwrap()
        }

        /// Get rate for streaming (price per gb)
        #[ink(message, payable)]
        pub fn cdn_get_rate(&self, cluster_id: ClusterId) -> Balance {
            self.message_cdn_get_rate(cluster_id).unwrap()
        }

        /// As validator, charge payments from users and allocate undistributed payments to CDN nodes.
        ///
        /// As a result CDN cluster revenue increases, which can be distributed between CDN node providers via method cdn_cluster_distribute_revenues.
        #[ink(message)]
        pub fn cluster_put_cdn_revenue(
            &mut self,
            cluster_id: ClusterId,
            aggregates_accounts: Vec<(AccountId, u128)>,
            aggregates_nodes: Vec<(CdnNodeKey, u128)>,
            aggregates_buckets: Vec<(BucketId, Resource)>,
            era: u64,
        ) -> () {
            self.message_cluster_put_cdn_revenue(
                cluster_id,
                aggregates_accounts,
                aggregates_nodes,
                aggregates_buckets,
                era,
            )
            .unwrap()
        }

        /// Trigger the distribution of revenues from the cluster to the CDN node providers.
        ///
        /// Anyone can call this method.
        ///
        /// Undistributed payments will be trasnferred, CDN cluster revenue will decrease.
        #[ink(message)]
        pub fn cluster_distribute_cdn_revenue(&mut self, cluster_id: ClusterId) {
            self.message_cluster_distribute_cdn_revenue(cluster_id)
                .unwrap()
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

    // ---- End CDN Cluster ----

    // ---- Committer ----

    impl DdcBucket {
        /// CDN node operator sets the commit for current era.
        #[ink(message)]
        pub fn set_commit(&mut self, cdn_owner: AccountId, cdn_node_key: CdnNodeKey, commit: Commit) {
            self.message_set_commit(cdn_owner, cdn_node_key, commit);
        }

        /// Return the last commit submitted by CDN node operator
        #[ink(message)]
        pub fn get_commit(&self, cdn_owner: AccountId) -> Vec<(CdnNodeKey, Commit)> {
            self.message_get_commit(cdn_owner)
        }

        /// Return last era validated per CDN node
        #[ink(message)]
        pub fn get_validated_commit(&self, cdn_node_key: CdnNodeKey) -> EraAndTimestamp {
            self.message_get_validated_commit(cdn_node_key)
        }

        /// Set the new configs for era
        #[ink(message)]
        pub fn set_era(&mut self, era_config: EraConfig) -> () {
            self.message_set_era(era_config).unwrap();
        }

        /// Return current status of an era
        #[ink(message)]
        pub fn get_era(&self) -> EraStatus {
            self.message_get_era()
        }

        /// Return current era settings
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
        cdn_node_key: CdnNodeKey,
        #[ink(topic)]
        provider_id: AccountId,
        cdn_node_params: CdnNodeParams,
        undistributed_payment: Balance,
    }

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct CdnNodeRemoved {
        #[ink(topic)]
        cdn_node_key: CdnNodeKey,
    }

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct CdnNodeParamsSet {
        #[ink(topic)]
        cdn_node_key: CdnNodeKey,
        cdn_node_params: CdnNodeParams,
    }

    impl DdcBucket {

        /// Creates a CDN node
        ///
        /// This endpoint creates a CDN node with specific parameters.
        /// The caller will be the node owner (node provider).
        ///
        /// # Parameters
        ///
        /// * `cdn_node_key` - Public Keys of the CDN node that should be treated as node identifier.
        /// * `cdn_node_params` - [CDN node parameters](https://docs.cere.network/ddc/protocols/contract-params-schema#node-params.proto) in protobuf format.
        ///
        /// # Output
        ///
        /// Returns Public Key of the created CDN node.
        ///
        /// # Events
        ///
        /// * `CdnNodeCreated` event on successful CDN node creation.
        ///
        /// # Errors
        ///
        /// * `CdnNodeAlreadyExists` error if a CDN node with the same Public Key is already created.
        /// * `InvalidParams(message)` error if there is some invalid configuration parameter.
        #[ink(message, payable)]
        pub fn cdn_node_create(
            &mut self, 
            cdn_node_key: CdnNodeKey, 
            cdn_node_params: CdnNodeParams
        ) -> Result<CdnNodeKey> {
            self.message_cdn_node_create(cdn_node_key, cdn_node_params)
        }

        /// Removes a CDN node.
        ///
        /// This enpoint removes the targeting CDN Node if it is not added to some cluster.
        /// Only a node that is not a member of some cluster can be removed.
        ///
        /// # Parameters
        ///
        /// * `cdn_node_key` - Public Key associated with the CDN node.
        ///
        /// # Output
        ///
        /// Returns nothing.
        ///
        /// # Events
        ///
        /// * `CdnNodeRemoved` event on successful CDN node removal.
        ///
        /// # Errors
        ///
        /// * `OnlyCdnNodeProvider` error if the caller is not the CDN node owner.
        /// * `CdnNodeDoesNotExist` error if the CDN node does not exist.
        /// * `CdnNodeIsAddedToCluster(ClusterId)` error if the removing CDN node is added to some cluster.
        #[ink(message)]
        pub fn cdn_node_remove(
            &mut self, 
            cdn_node_key: CdnNodeKey
        ) -> Result<()> {
            self.message_remove_cdn_node(cdn_node_key)
        }

        /// Sets parameters for the targeting CDN node.
        ///
        /// This enpoint updates [CDN node parameters](https://docs.cere.network/ddc/protocols/contract-params-schema#node-params.proto) in protobuf format. 
        /// All CDN node parameters must be specified as the endpoint works using SET approach.
        ///
        /// # Parameters
        ///
        /// * `cdn_node_key` - Public Key associated with the CDN node.
        /// * `cdn_node_params` - [CDN node parameters](https://docs.cere.network/ddc/protocols/contract-params-schema#node-params.proto) in protobuf format.
        ///
        /// # Output
        ///
        /// Returns nothing.
        ///
        /// # Events
        ///
        /// * `CdnNodeParamsSet` event on successful CDN node params setting.
        ///
        /// # Errors
        ///
        /// * `OnlyCdnNodeProvider` error if the caller is not the CDN node owner.
        /// * `CdnNodeDoesNotExist` error if the CDN node does not exist.
        #[ink(message, payable)]
        pub fn cdn_node_set_params(
            &mut self, 
            cdn_node_key: CdnNodeKey, 
            cdn_node_params: CdnNodeParams
        ) -> Result<()> {
            self.message_cdn_node_set_params(cdn_node_key, cdn_node_params)
        }

        /// Gets a CDN node.
        ///
        /// This enpoint gets the targeting CDN node along with its parameters.
        ///
        /// # Parameters
        ///
        /// * `cdn_node_key` - Public Key associated with the CDN node.
        ///
        /// # Output
        ///
        /// Returns `CdnNodeInfo` data transfer object.
        ///
        /// # Errors
        ///
        /// * `CdnNodeDoesNotExist` error if the CDN node does not exist.
        #[ink(message)]
        pub fn cdn_node_get(
            &self, 
            cdn_node_key: CdnNodeKey
        ) -> Result<CdnNodeInfo> {
            self.message_cdn_node_get(cdn_node_key)
        }

        /// Gets a paginated list of CDN nodes.
        ///
        /// This enpoint gets a paginated list of CDN nodes along with their parameters.
        /// The algorithm for paging is: start with `offset = 1` and `limit = 20`. The function returns a `(results, max_id)`. Call again with `offset += limit`, until `offset >= max_id`.
        /// The optimal `limit` depends on the size of params.
        ///
        /// # Parameters
        ///
        /// * `offset` - starting offset.
        /// * `limit` - page limit.
        /// * `filter_provider_id` - optional filter by CDN node owner.
        ///
        /// # Errors
        ///
        /// No errors. In case a pagination param is out of bounds, an empty list will be returned.
        #[ink(message)]
        pub fn cdn_node_list(
            &self,
            offset: u32,
            limit: u32,
            filter_provider_id: Option<AccountId>,
        ) -> (Vec<CdnNodeInfo>, u32) {
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
        node_key: NodeKey,
        #[ink(topic)]
        provider_id: AccountId,
        rent_per_month: Balance,
        node_params: NodeParams,
    }

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct NodeRemoved {
        #[ink(topic)]
        node_key: NodeKey,
    }

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct NodeParamsSet {
        #[ink(topic)]
        node_key: NodeKey,
        node_params: NodeParams,
    }

    impl DdcBucket {

        /// Creates a Storage node
        ///
        /// This endpoint creates a Storage node with specific parameters.
        /// The caller will be the node owner (node provider).
        ///
        /// # Parameters
        ///
        /// * `node_key` - Public Keys of the Storage node that should be treated as node identifier.
        /// * `node_params` - [Storage node parameters](https://docs.cere.network/ddc/protocols/contract-params-schema#node-params.proto) in protobuf format.
        /// * `capacity` - Measure used to evaluate physical node hardware resources.
        /// * `rent_per_month` - Renting per month.
        ///
        /// # Output
        ///
        /// Returns Public Key of the created Storage node.
        ///
        /// # Events
        ///
        /// * `NodeCreated` event on successful Storage node creation.
        ///
        /// # Errors
        ///
        /// * `NodeAlreadyExists` error if a Storage node with the same Public Key is already created.
        /// * `InvalidParams(message)` error if there is some invalid configuration parameter.
        #[ink(message, payable)]
        pub fn node_create(
            &mut self,
            node_key: NodeKey,
            node_params: NodeParams,
            capacity: Resource,
            rent_per_month: Balance
        ) -> Result<NodeKey> {
            self.message_node_create(
                node_key, 
                node_params, 
                capacity, 
                rent_per_month
            )
        }

        /// Removes a Storage node.
        ///
        /// This enpoint removes the targeting Storage Node if it is not added to some cluster.
        /// Only a node that is not a member of some cluster can be removed.
        ///
        /// # Parameters
        ///
        /// * `node_key` - Public Key associated with the Storage node.
        ///
        /// # Output
        ///
        /// Returns nothing.
        ///
        /// # Events
        ///
        /// * `NodeRemoved` event on successful Storage node removal.
        ///
        /// # Errors
        ///
        /// * `OnlyNodeProvider` error if the caller is not the Storage node owner.
        /// * `NodeDoesNotExist` error if the Storage node does not exist.
        /// * `NodeIsAddedToCluster(ClusterId)` error if the removing Storage node is added to some cluster.
        #[ink(message)]
        pub fn node_remove(
            &mut self, 
            node_key: NodeKey
        ) -> Result<()> {
            self.message_node_remove(node_key)
        }

        /// Sets parameters for the targeting Storage node.
        ///
        /// This enpoint updates [Storage node parameters](https://docs.cere.network/ddc/protocols/contract-params-schema#node-params.proto) in protobuf format. 
        /// All Storage node parameters must be specified as the endpoint works using SET approach.
        ///
        /// # Parameters
        ///
        /// * `node_key` - Public Key associated with the Storage node.
        /// * `node_params` - [Storage node parameters](https://docs.cere.network/ddc/protocols/contract-params-schema#node-params.proto) in protobuf format. 
        ///
        /// # Output
        ///
        /// Returns nothing.
        ///
        /// # Events
        ///
        /// * `NodeParamsSet` event on successful Storage node params setting.
        ///
        /// # Errors
        ///
        /// * `OnlyNodeProvider` error if the caller is not the Storage node owner.
        /// * `NodeDoesNotExist` error if the Storage node does not exist.
        #[ink(message, payable)]
        pub fn node_set_params(
            &mut self, 
            node_key: NodeKey, 
            node_params: NodeParams
        ) -> Result<()> {
            self.message_node_set_params(node_key, node_params)
        }

        /// Gets a Storage node.
        ///
        /// This enpoint gets the targeting Storage node along with its parameters.
        ///
        /// # Parameters
        ///
        /// * `node_key` - Public Key associated with the Storage node.
        ///
        /// # Output
        ///
        /// Returns `NodeInfo` data transfer object.
        ///
        /// # Errors
        ///
        /// * `NodeDoesNotExist` error if the Storage node does not exist.
        #[ink(message)]
        pub fn node_get(
            &self, 
            node_key: NodeKey
        ) -> Result<NodeInfo> {
            self.message_node_get(node_key)
        }

        /// Gets a paginated list of Storage nodes.
        ///
        /// This enpoint gets a paginated list of Storage nodes along with their parameters.
        /// The algorithm for paging is: start with `offset = 1` and `limit = 20`. The function returns a `(results, max_id)`. Call again with `offset += limit`, until `offset >= max_id`.
        /// The optimal `limit` depends on the size of params.
        ///
        /// # Parameters
        ///
        /// * `offset` - starting offset.
        /// * `limit` - page limit.
        /// * `filter_provider_id` - optional filter by Storage node owner.
        ///
        /// # Errors
        ///
        /// No errors. In case a pagination param is out of bounds, an empty list will be returned.
        #[ink(message)]
        pub fn node_list(
            &self,
            offset: u32,
            limit: u32,
            filter_provider_id: Option<AccountId>,
        ) -> (Vec<NodeInfo>, u32) {
            self.message_node_list(offset, limit, filter_provider_id)
        }

    }
    // ---- End Node ----

    // ---- Protocol ----

    impl DdcBucket {
        /// Get the Fee Percentage Basis Points that will be charged by the protocol
        #[ink(message)]
        pub fn get_protocol_fee_bp(&self) -> u128 {
            self.message_get_protocol_fee_bp()
        }

        /// Return fees accumulated by the protocol
        #[ink(message)]
        pub fn get_protocol_revenues(&self) -> Cash {
            self.message_get_fee_revenues()
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
    pub struct PermissionGranted {
        #[ink(topic)]
        account_id: AccountId,
        permission: Permission,
    }

    /// A permission was revoked from the account.
    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct PermissionRevoked {
        #[ink(topic)]
        account_id: AccountId,
        permission: Permission,
    }

    impl DdcBucket {
        /// Checks for permission existence.
        ///
        /// This enpoint checks whether the given account has the given permission.
        /// Super-admin will always have all permissions.
        ///
        /// # Parameters
        ///
        /// * `account_id` - account to check permissions.
        /// * `permission` - permission to check.
        ///
        /// # Output
        ///
        /// Returns true if the account has permissions, false otherwise.
        ///
        /// # Errors
        ///
        /// No errors.
        #[ink(message)]
        pub fn has_permission(
            &self, 
            account_id: AccountId,
            permission: Permission
        ) -> bool {
            self.perms.has_permission(account_id, permission)
        }

        /// Grants permissions for a cluster manager.
        ///
        /// This endpoint grants permissions for a cluster manager ro manage Storage or CDN node owner.
        /// After the permission is granted, the cluster manager can add nodes to the cluster.
        /// Permissions can be granted by Storage or CDN node owner.
        ///
        /// # Parameters
        ///
        /// * `manager` - cluster manager account.
        ///
        /// # Output
        ///
        /// Returns nothing.
        ///
        /// # Events
        ///
        /// * `PermissionGranted` event on successful manager permissions granting
        ///
        /// # Errors
        ///
        /// No errors. The endpoint is idempotent.
        #[ink(message, payable)]
        pub fn grant_trusted_manager_permission(
            &mut self, 
            manager: AccountId
        ) -> Result<()> {
            self.message_grant_trusted_manager_permission(manager)
        }
        
        /// Revokes permissions from cluster manager.
        ///
        /// This endpoint revokes permissions from a cluster manager to manage Storage or CDN node owner.
        /// After the permission is revoked, the cluster manager can add nodes to the cluster.
        /// Permissions can be revoked by Storage or CDN node owner.
        ///
        /// # Parameters
        ///
        /// * `manager` - cluster manager account.
        ///
        /// # Output
        ///
        /// Returns nothing.
        ///
        /// # Events
        ///
        /// * `PermissionRevoked` event on successful manager permissions revoking
        ///
        /// # Errors
        ///
        /// No errors. The endpoint is idempotent.
        #[ink(message)]
        pub fn revoke_trusted_manager_permission(
            &mut self, 
            manager: AccountId
        ) -> Result<()> {
            self.message_revoke_trusted_manager_permission(manager)
        }

    }
    // ---- End Permissions ----

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct NodeOwnershipTransferred {
        #[ink(topic)]
        account_id: AccountId,
        #[ink(topic)]
        node_key: NodeKey,
    }

    #[ink(event)]
    #[cfg_attr(feature = "std", derive(PartialEq, Debug, scale_info::TypeInfo))]
    pub struct CdnNodeOwnershipTransferred {
        #[ink(topic)]
        account_id: AccountId,
        #[ink(topic)]
        cdn_node_key: CdnNodeKey,
    }

    // ---- Admin ----
    impl DdcBucket {

        /// Grants any permission.
        ///
        /// This endpoint grants any permissions for any account by the Super-admin.
        ///
        /// # Parameters
        ///
        /// * `grantee` - account to grant permission.
        /// * `permission` - permission type.
        ///
        /// # Output
        ///
        /// Returns nothing.
        ///
        /// # Events
        ///
        /// * `PermissionGranted` event on successful permissions granting
        ///
        /// # Errors
        ///
        /// Returns `OnlySuperAdmin` error if the caller is not the Super-admin.
        #[ink(message)]
        pub fn admin_grant_permission(
            &mut self, 
            grantee: AccountId, 
            permission: Permission
        ) -> Result<()> {
            self.message_admin_grant_permission(grantee, permission)
        }

        /// Revokes any permission.
        ///
        /// This endpoint revokes any permissions from any account by the Super-admin.
        ///
        /// # Parameters
        ///
        /// * `grantee` - account to revoke permission.
        /// * `permission` - permission type.
        ///
        /// # Output
        ///
        /// Returns nothing.
        ///
        /// # Events
        ///
        /// * `PermissionRevoked` event on successful permissions revoking
        ///
        /// # Errors
        ///
        /// Returns `OnlySuperAdmin` error if the caller is not the Super-admin.
        #[ink(message)]
        pub fn admin_revoke_permission(
            &mut self, 
            grantee: AccountId, 
            permission: Permission
        ) -> Result<()> {
            self.message_admin_revoke_permission(grantee, permission)
        }

        /// Transfers Storage node ownership.
        ///
        /// This endpoint transfers Storage node ownership from Super-admin account to the targeting account forever.
        /// This action is usually required only once after the Storage node certification process.
        ///
        /// # Parameters
        ///
        /// * `node_key` - Public Key associated with the Storage node.
        /// * `new_owner` - New Storage node owner
        ///
        /// # Output
        ///
        /// Returns nothing.
        ///
        /// # Events
        ///
        /// * `NodeOwnershipTransferred` event on successful Storage node ownership transfer
        ///
        /// # Errors
        ///
        /// * `OnlySuperAdmin` error if the caller is not the Super-admin.
        /// * `NodeDoesNotExist` error if the Storage node does not exist.
        /// * `NodeProviderIsNotSuperAdmin` error if the owner of the targeting node is not the Super-admin.
        #[ink(message)]
        pub fn admin_transfer_node_ownership(
            &mut self, 
            node_key: NodeKey, 
            new_owner: AccountId
        ) -> Result<()> {
            self.message_admin_transfer_node_ownership(node_key, new_owner)
        }

        /// Transfers CDN node ownership.
        ///
        /// This endpoint transfers CDN node ownership from Super-admin account to the targeting account forever.
        /// This action is usually required only once after the CDN node certification process.
        ///
        /// # Parameters
        ///
        /// * `cdn_node_key` - Public Key associated with the CDN node.
        /// * `new_owner` - CDN node owner
        ///
        /// # Output
        ///
        /// Returns nothing.
        ///
        /// # Events
        ///
        /// * `CdnNodeOwnershipTransferred` event on successful CDN node ownership transfer
        ///
        /// # Errors
        ///
        /// * `OnlySuperAdmin` error if the caller is not the Super-admin.
        /// * `CdnNodeDoesNotExist` error if the Storage node does not exist.
        /// * `CdnNodeOwnerIsNotSuperAdmin` error if the owner of the targeting node is not the Super-admin.
        #[ink(message)]
        pub fn admin_transfer_cdn_node_ownership(
            &mut self, 
            cdn_node_key: CdnNodeKey, 
            new_owner: AccountId
        ) -> Result<()> {
            self.message_admin_transfer_cdn_node_ownership(cdn_node_key, new_owner)
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
        pub fn admin_set_network_fee_config(&mut self, config: NetworkFeeConfig) {
            self.message_admin_set_network_fee_config(config).unwrap();
        }

        /// Pay the revenues accumulated by the protocol
        #[ink(message)]
        pub fn admin_withdraw_protocol_revenues(&mut self, amount: u128) -> () {
            self.message_withdraw_revenues(amount).unwrap();
        }

        #[ink(message)]
        pub fn admin_set_protocol_fee_bp(&mut self, protocol_fee_bp: u128) -> () {
            self.message_set_protocol_fee_bp(protocol_fee_bp).unwrap();
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

    // ---- Topology ----
    impl DdcBucket {

        #[ink(message)]
        pub fn get_v_nodes_by_cluster(&self, cluster_id: ClusterId) -> Vec<VNodeToken> {
            self.message_get_v_nodes_by_cluster(cluster_id)
        }
    
        #[ink(message)]
        pub fn get_v_nodes_by_node(&self, node_key: NodeKey) -> Vec<VNodeToken> {
            self.message_get_v_nodes_by_node(node_key)
        }
    
        #[ink(message)]
        pub fn get_node_by_v_node(&self, cluster_id: ClusterId, v_node: VNodeToken) -> Result<NodeKey> {
            self.message_get_node_by_v_node(cluster_id, v_node)
        }

    }
    // ---- End Topology ----


    // ---- Constants ----
    /// One token with 10 decimals.
    pub const TOKEN: Balance = 10_000_000_000;
    pub const BASIS_POINTS: u128 = 10_000; // 100%
    pub const DEFAULT_PROTOCOL_FEE_BP: u128 = 500; // 5 %


    #[derive(Debug, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        NodeDoesNotExist,
        CdnNodeDoesNotExist,
        NodeAlreadyExists,
        CdnNodeAlreadyExists,
        AccountDoesNotExist,
        ParamsDoesNotExist,
        ParamsSizeExceedsLimit,
        OnlyOwner,
        OnlyNodeProvider,
        OnlyCdnNodeProvider,
        OnlyClusterManager,
        OnlyTrustedClusterManager,
        OnlySuperAdmin,
        OnlyClusterManagerOrNodeProvider,
        OnlyClusterManagerOrCdnNodeProvider,
        Unauthorized,
        ClusterDoesNotExist,
        ClusterIsNotEmpty,
        TopologyIsNotCreated(ClusterId),
        TopologyAlreadyExists,
        VNodesSizeExceedsLimit,
        NodeIsNotAddedToCluster(ClusterId),
        NodeIsAddedToCluster(ClusterId),
        CdnNodeIsNotAddedToCluster(ClusterId),
        CdnNodeIsAddedToCluster(ClusterId),
        VNodeDoesNotExistsInCluster(ClusterId),
        VNodeIsNotAssignedToNode(ClusterId, VNodeToken),
        VNodeIsAlreadyAssignedToNode(NodeKey),
        AtLeastOneVNodeHasToBeAssigned(ClusterId, NodeKey),
        NodeProviderIsNotSuperAdmin,
        CdnNodeOwnerIsNotSuperAdmin,
        BucketDoesNotExist,
        BondingPeriodNotFinished,
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
