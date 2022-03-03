use crate::ddc_bucket::*;

use super::test_utils::*;

pub struct TestNode {
    pub provider_id: AccountId,
    pub vnode_ids: Vec<VNodeId>,
    pub engine_name: String,
}

impl TestNode {
    pub fn new(provider_id: AccountId, engine_name: &str) -> Self {
        Self { provider_id, vnode_ids: vec![], engine_name: engine_name.into() }
    }

    pub fn join_cluster(&mut self, ddc_bucket: &mut DdcBucket, cluster_id: ClusterId) -> Result<()> {
        push_caller(self.provider_id);

        let rent_per_month: Balance = 10 * CURRENCY;
        let vnode_params = "{\"url\":\"https://ddc.cere.network/bucket/{BUCKET_ID}\"}";

        let vnode_id = ddc_bucket.vnode_create(cluster_id, rent_per_month, vnode_params.to_string())?;
        self.vnode_ids.push(vnode_id);

        pop_caller();
        Ok(())
    }
}