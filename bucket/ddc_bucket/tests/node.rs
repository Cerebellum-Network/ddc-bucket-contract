// use crate::ddc_bucket::tests::topology::Topology;
// use crate::ddc_bucket::*;

// use super::env_utils::*;

// pub struct TestNode {
//     pub provider_id: AccountId,
//     pub node_id: NodeId,
//     pub engine_name: String,
//     pub url: String,
// }

// impl TestNode {
//     pub fn new(
//         contract: &mut DdcBucket,
//         provider_id: AccountId,
//         manager_id: AccountId,
//         engine_name: &str,
//         node_name: &str,
//     ) -> Self {
//         let url = format!(
//             "https://node-{}.ddc.cere.network/{}/",
//             node_name, engine_name
//         );
//         let node_params = url.clone();
//         let rent_per_month: Balance = 10 * TOKEN;
//         let capacity = 100;

//         push_caller_value(provider_id, CONTRACT_FEE_LIMIT);
//         contract.node_trust_manager(manager_id);
//         pop_caller();

//         push_caller_value(provider_id, CONTRACT_FEE_LIMIT);
//         let node_id = contract.node_create(rent_per_month, node_params, capacity);
//         pop_caller();

//         Self {
//             provider_id,
//             node_id,
//             engine_name: engine_name.into(),
//             url,
//         }
//     }
// }

// pub fn find_cluster(contract: &DdcBucket, engine_name: &str) -> Result<ClusterStatus> {
//     // Discover the available clusters.
//     let (clusters, _count) = contract.cluster_list(1, 20, None);

//     // Pick the first one that provides the right engine.
//     let cluster = clusters
//         .iter()
//         .find(|cluster| {
//             let topology = Topology::from_str(&cluster.params).unwrap();
//             topology.engine_name == engine_name
//         })
//         .expect(&format!("No cluster found for engine {}", engine_name));

//     Ok(cluster.clone())
// }

// pub struct TestRequest {
//     pub url: String,
//     pub bucket_id: BucketId,
//     pub sender: AccountId,
//     pub action: Action,
// }

// #[derive(Clone, Debug, Eq, PartialEq)]
// pub struct Action {
//     pub routing_key: u32,
//     pub data: String,
//     pub op: Op,
// }

// #[derive(Clone, Debug, Eq, PartialEq)]
// pub enum Op {
//     Write,
//     Read,
// }
