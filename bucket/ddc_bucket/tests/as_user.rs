// use crate::ddc_bucket::*;
// use crate::ddc_bucket::Error::BucketDoesNotExist;

// use super::as_gateway::GATEWAY_ENGINE;
// use super::as_storage::STORAGE_ENGINE;
// use super::env_utils::*;
// use super::node::{Action, find_cluster, TestRequest};
// use super::topology::BucketParams;

// const BUCKET_PARAMS: BucketParams = BucketParams { replication: 3 };

// pub struct TestUser {
//     pub account_id: AccountId,
//     pub storage_bucket_id: BucketId,
// }

// impl TestUser {
//     pub fn new(contract: &mut DdcBucket, account_id: AccountId) -> Result<Self> {
//         // Deposit some value.
//         push_caller_value(account_id, 10 * TOKEN);
//         contract.account_deposit();
//         pop_caller();

//         let storage_bucket_id = Self::create_bucket(contract, account_id, STORAGE_ENGINE)?;

//         Ok(Self { account_id, storage_bucket_id })
//     }

//     pub fn create_bucket(contract: &mut DdcBucket, account_id: AccountId, engine_name: &str) -> Result<BucketId> {
//         // Choose a cluster.
//         let cluster_id = find_cluster(contract, engine_name)?.cluster_id;

//         push_caller_value(account_id, CONTRACT_FEE_LIMIT);
//         let bucket_id = contract.bucket_create(BUCKET_PARAMS.to_string().unwrap(), cluster_id, None);
//         pop_caller();

//         // Allocate the bucket to the cluster.
//         push_caller_value(account_id, CONTRACT_FEE_LIMIT);
//         let resource = 1;
//         contract.bucket_alloc_into_cluster(bucket_id, resource);
//         pop_caller();

//         Ok(bucket_id)
//     }

//     pub fn find_bucket(contract: &DdcBucket, account_id: AccountId) -> Result<BucketStatus> {
//         // Discover the buckets owned by the account.
//         let (buckets, _count) = contract.bucket_list(0, 20, Some(account_id));
//         buckets.first().cloned().ok_or(BucketDoesNotExist)
//     }

//     pub fn make_request(&self, contract: &DdcBucket, action: Action) -> Result<TestRequest> {
//         // Find own bucket.
//         let bucket_id = Self::find_bucket(contract, self.account_id)?.bucket_id;
//         assert_eq!(bucket_id, self.storage_bucket_id, "should find the bucket that we created before");

//         // Find a gateway cluster.
//         let cdn_cluster = find_cluster(contract, GATEWAY_ENGINE)?.cluster;
//         // Pick a gateway node.
//         let cdn_node_id = *cdn_cluster.vnodes.first().expect("empty cluster");
//         let cdn_node = contract.node_get(cdn_node_id)?;
//         // Get the URL of the gateway.
//         let url = cdn_node.params;
//         // Prepare a request.
//         let request = TestRequest {
//             url,
//             bucket_id,
//             sender: self.account_id,
//             action,
//         };
//         Ok(request)
//     }
// }
