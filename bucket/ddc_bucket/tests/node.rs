use std::collections::HashSet;

use crate::ddc_bucket::*;
use crate::ddc_bucket::tests::topology::Topology;

use super::env_utils::*;

pub struct TestNode {
    pub provider_id: AccountId,
    pub node_id: NodeId,
    pub cluster_ids: HashSet<ClusterId>,
    pub engine_name: String,
    pub url: String,
}

impl TestNode {
    pub fn new(contract: &mut DdcBucket, provider_id: AccountId, engine_name: &str, node_name: &str) -> Self {
        let url = format!("https://node-{}.ddc.cere.network/{}/", node_name, engine_name);
        let node_params = url.clone();
        let rent_per_month: Balance = 10 * TOKEN;

        push_caller_value(provider_id, CONTRACT_FEE_LIMIT);
        let node_id = contract.node_create(rent_per_month, node_params).unwrap();
        pop_caller();

        Self { provider_id, node_id, cluster_ids: Default::default(), engine_name: engine_name.into(), url }
    }

    pub fn join_cluster(&mut self, cluster_id: ClusterId) {
        self.cluster_ids.insert(cluster_id);
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
    pub routing_key: u32,
    pub data: String,
    pub op: Op,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Op {
    Write,
    Read,
}
