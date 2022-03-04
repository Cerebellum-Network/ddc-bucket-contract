use crate::ddc_bucket::*;
use crate::ddc_bucket::tests::as_storage::STORAGE_ENGINE;

use super::as_gateway::GATEWAY_ENGINE;
use super::node::{find_cluster, TestRequest};
use super::env_utils::*;
use crate::ddc_bucket::tests::node::Action;

pub struct TestUser {
    pub account_id: AccountId,
    pub storage_bucket_id: BucketId,
}

impl TestUser {
    pub fn new(contract: &mut DdcBucket, account_id: AccountId) -> Result<Self> {
        let storage_bucket_id = Self::create_bucket(contract, account_id, STORAGE_ENGINE)?;

        Ok(Self { account_id, storage_bucket_id })
    }

    pub fn create_bucket(contract: &mut DdcBucket, account_id: AccountId, engine_name: &str) -> Result<BucketId> {
        push_caller_value(account_id, 0);
        let bucket_params = "".to_string();
        let bucket_id = contract.bucket_create(bucket_params.clone())?;
        pop_caller();

        // Choose a cluster.
        let cluster_id = find_cluster(contract, engine_name)?.cluster_id;

        // Allocate the bucket to the cluster, also depositing some value.
        push_caller_value(account_id, 10 * CURRENCY);
        contract.bucket_alloc_into_cluster(bucket_id, cluster_id)?;
        pop_caller();

        Ok(bucket_id)
    }

    pub fn make_request(&self, contract: &DdcBucket, action: Action) -> Result<TestRequest> {
        // Find a gateway cluster.
        let cluster = find_cluster(contract, GATEWAY_ENGINE)?;
        // Pick a gateway node.
        let vnode_id = *cluster.vnode_ids.first().expect("empty cluster");
        let vnode = contract.vnode_get(vnode_id)?;
        // Get the URL of the gateway.
        let url = vnode.vnode_params;
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
