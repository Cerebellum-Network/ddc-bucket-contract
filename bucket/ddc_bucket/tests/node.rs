use crate::ddc_bucket::*;

use super::test_utils::*;

pub struct TestNode {
    pub provider_id: AccountId,
    pub vnode_ids: Vec<VNodeId>,
    pub engine_name: String,
    pub url: String,
}

impl TestNode {
    pub fn new(provider_id: AccountId, engine_name: &str, node_name: &str) -> Self {
        let url = format!("https://node-{}.ddc.cere.network/{}/", node_name, engine_name);
        Self { provider_id, vnode_ids: vec![], engine_name: engine_name.into(), url }
    }

    pub fn join_cluster(&mut self, ddc_bucket: &mut DdcBucket, cluster_id: ClusterId) -> Result<()> {
        push_caller(self.provider_id);

        let rent_per_month: Balance = 10 * CURRENCY;
        let vnode_params = self.url.clone();

        let vnode_id = ddc_bucket.vnode_create(cluster_id, rent_per_month, vnode_params)?;
        self.vnode_ids.push(vnode_id);

        pop_caller();
        Ok(())
    }
}

pub struct TestRequest {
    pub url: String,
    pub bucket_id: BucketId,
    pub sender: AccountId,
}
