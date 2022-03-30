use crate::ddc_bucket::*;

use super::as_gateway::GATEWAY_ENGINE;
use super::as_storage::STORAGE_ENGINE;
use super::env_utils::*;
use super::node::{Action, find_cluster, TestRequest};
use super::topology::BucketParams;

const BUCKET_PARAMS: BucketParams = BucketParams { replication: 3 };

pub struct TestUser {
    pub account_id: AccountId,
    pub storage_bucket_id: BucketId,
}

impl TestUser {
    pub fn new(contract: &mut DdcBucket, account_id: AccountId) -> Result<Self> {
        // Deposit some value.
        push_caller_value(account_id, 10 * TOKEN);
        contract.deposit();
        pop_caller();

        let storage_bucket_id = Self::create_bucket(contract, account_id, STORAGE_ENGINE)?;

        Ok(Self { account_id, storage_bucket_id })
    }

    pub fn create_bucket(contract: &mut DdcBucket, account_id: AccountId, engine_name: &str) -> Result<BucketId> {
        // Choose a cluster.
        let cluster_id = find_cluster(contract, engine_name)?.cluster_id;

        push_caller_value(account_id, CONTRACT_FEE_LIMIT);
        let bucket_id = contract.bucket_create(BUCKET_PARAMS.to_string().unwrap(), cluster_id);
        pop_caller();

        // Allocate the bucket to the cluster.
        push_caller_value(account_id, CONTRACT_FEE_LIMIT);
        contract.bucket_alloc_into_cluster(bucket_id);
        pop_caller();

        Ok(bucket_id)
    }

    pub fn make_request(&self, contract: &DdcBucket, action: Action) -> Result<TestRequest> {
        // Find a gateway cluster.
        let cluster = find_cluster(contract, GATEWAY_ENGINE)?;
        // Pick a gateway node.
        let node_id = *cluster.vnodes.first().expect("empty cluster");
        let node = contract.node_get(node_id)?;
        // Get the URL of the gateway.
        let url = node.node_params;
        // Prepare a request.
        let request = TestRequest {
            url,
            bucket_id: self.storage_bucket_id,
            sender: self.account_id,
            action,
        };
        Ok(request)
    }
}
