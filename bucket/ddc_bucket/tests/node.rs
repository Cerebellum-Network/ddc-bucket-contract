use std::collections::HashSet;

use crate::ddc_bucket::*;
use crate::ddc_bucket::tests::cluster::Topology;

use super::env_utils::*;

pub struct TestNode {
    pub provider_id: AccountId,
    pub vnode_ids: HashSet<VNodeId>,
    pub cluster_ids: HashSet<ClusterId>,
    pub engine_name: String,
    pub url: String,
}

impl TestNode {
    pub fn new(provider_id: AccountId, engine_name: &str, node_name: &str) -> Self {
        let url = format!("https://node-{}.ddc.cere.network/{}/", node_name, engine_name);

        Self { provider_id, vnode_ids: Default::default(), cluster_ids: Default::default(), engine_name: engine_name.into(), url }
    }

    pub fn join_cluster(&mut self, contract: &mut DdcBucket, cluster_id: ClusterId) -> Result<()> {
        push_caller(self.provider_id);

        let rent_per_month: Balance = 10 * CURRENCY;
        let vnode_params = self.url.clone();

        let vnode_id = contract.vnode_create(cluster_id, rent_per_month, vnode_params)?;
        self.vnode_ids.insert(vnode_id);
        self.cluster_ids.insert(cluster_id);

        pop_caller();
        Ok(())
    }
}

pub fn find_cluster(contract: &DdcBucket, engine_name: &str) -> Result<Cluster> {
    // Discover the available clusters.
    let (clusters, _count) = contract.cluster_list(0, 20);

    // Pick the first one that provides the right engine.
    let cluster = clusters.iter()
        .find(|cluster| {
            let topology = Topology::from_str(&cluster.cluster_params).unwrap();
            topology.engine_name == engine_name
        })
        .expect(&format!("No cluster found for engine {}", engine_name));

    Ok(cluster.clone())
}

pub struct TestRequest {
    pub url: String,
    pub bucket_id: BucketId,
    pub sender: AccountId,
    pub action: Action,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Action {
    pub routing_key: usize,
    pub data: String,
    pub op: Op,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Op {
    Write,
    Read,
}
