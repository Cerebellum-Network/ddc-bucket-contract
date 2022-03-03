use crate::ddc_bucket::*;

use super::as_storage::STORAGE_ENGINE;
use super::node::{TestNode, TestRequest};

pub const GATEWAY_ENGINE: &str = "gateway";

pub struct TestGateway {
    pub vnode: TestNode,
}

impl TestGateway {
    pub fn new(provider_id: AccountId, node_name: &str) -> Self {
        Self { vnode: TestNode::new(provider_id, GATEWAY_ENGINE, node_name) }
    }

    pub fn handle_request(&self, contract: &DdcBucket, client_request: TestRequest) -> Result<Vec<TestRequest>> {
        assert_eq!(client_request.url, self.vnode.url, "wrong gateway URL");

        let mut storage_requests = vec![];

        // Find the storage cluster of this bucket.
        let bucket = contract.bucket_get(client_request.bucket_id)?;
        let cluster_id = bucket.cluster_ids.first().expect("bucket has no clusters");
        let storage_cluster = contract.cluster_get(*cluster_id)?;
        assert_eq!(storage_cluster.cluster_params, STORAGE_ENGINE, "cluster should run the storage engine");

        // Make a request to the right storage nodes.
        for vnode_id in storage_cluster.vnode_ids {
            let storage_vnode = contract.vnode_get(vnode_id)?;
            // Get the URL of the storage node.
            let storage_url = storage_vnode.vnode_params;
            // Prepare a request.
            let storage_request = TestRequest {
                url: storage_url,
                bucket_id: client_request.bucket_id,
                sender: client_request.sender,
                action: client_request.action.clone(),
            };
            storage_requests.push(storage_request);
        }

        Ok(storage_requests)
    }
}
