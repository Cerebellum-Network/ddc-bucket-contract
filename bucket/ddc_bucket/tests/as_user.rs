use crate::ddc_bucket::*;

use super::node::TestRequest;
use super::test_utils::*;

pub struct TestUser {
    pub account_id: AccountId,
    pub storage_bucket_id: BucketId,
}

impl TestUser {
    pub fn new(ddc_bucket: &mut DdcBucket, account_id: AccountId) -> Result<Self> {
        let storage_bucket_id = Self::create_bucket(ddc_bucket, account_id, "storage")?;

        Ok(Self { account_id, storage_bucket_id })
    }

    pub fn create_bucket(ddc_bucket: &mut DdcBucket, account_id: AccountId, engine_name: &str) -> Result<BucketId> {
        push_caller_value(account_id, 0);
        let bucket_params = "".to_string();
        let bucket_id = ddc_bucket.bucket_create(bucket_params.clone())?;
        pop_caller();

        // Choose a cluster.
        let cluster_id = Self::find_cluster(ddc_bucket, engine_name)?.cluster_id;

        // Allocate the bucket to the cluster, also depositing some value.
        push_caller_value(account_id, 10 * CURRENCY);
        ddc_bucket.bucket_alloc_into_cluster(bucket_id, cluster_id)?;
        pop_caller();

        Ok(bucket_id)
    }
    pub fn find_cluster(ddc_bucket: &DdcBucket, engine_name: &str) -> Result<Cluster> {
        // Discover the available clusters.
        let (clusters, _count) = ddc_bucket.cluster_list(0, 20);

        // Pick the first one that provides the right engine.
        let cluster = clusters.iter()
            .find(|cluster|
                cluster.cluster_params.contains(engine_name))
            .expect(&format!("No cluster found for engine {}", engine_name));

        Ok(cluster.clone())
    }

    pub fn make_request(&self, ddc_bucket: &DdcBucket) -> Result<TestRequest> {
        // Find a gateway cluster.
        let cluster = Self::find_cluster(ddc_bucket, "gateway")?;
        // Pick a gateway node.
        let vnode_id = *cluster.vnode_ids.first().expect("empty cluster");
        let vnode = ddc_bucket.vnode_get(vnode_id)?;
        // Get the URL of the gateway.
        let url = vnode.vnode_params;
        // Prepare a request.
        let request = TestRequest {
            url,
            bucket_id: self.storage_bucket_id,
            sender: self.account_id,
        };
        Ok(request)
    }
}
