use ink_lang as ink;

use crate::ddc_bucket::*;
use crate::ddc_bucket::tests::topology::Topology;

use super::{as_gateway::*, as_storage::*, as_user::*, env_utils::*, node::*};
use crate::ddc_bucket::tests::as_cluster_manager::replace_node;

#[ink::test]
fn storage_network_works() {
    let accounts = get_accounts();
    set_balance(accounts.charlie, 1000 * TOKEN);
    set_balance(accounts.django, 1000 * TOKEN);
    set_balance(accounts.eve, 1000 * TOKEN);
    let cluster_manager = accounts.alice;

    let mut contract = DdcBucket::new();

    let vnode_specs = vec![
        (accounts.charlie, "charlie-0"),
        (accounts.django, "django-0"),
        (accounts.eve, "eve-0"),
        (accounts.charlie, "charlie-1"),
        (accounts.django, "django-1"),
        (accounts.eve, "eve-1"),
    ];
    let partition_count = vnode_specs.len() as u32;

    // Provide storage VNodes.
    let mut storage_nodes: Vec<TestStorage> =
        vnode_specs.iter().map(|spec| {
            TestStorage::new(&mut contract, spec.0, spec.1)
        }).collect();
    assert_ne!(storage_nodes[0].vnode.url, storage_nodes[1].vnode.url, "nodes must have different URLs");

    // Create a storage Cluster.
    {
        let node_ids = storage_nodes.iter().map(|node|
            node.vnode.vnode_id).collect();

        let topology = Topology::new(STORAGE_ENGINE, partition_count);

        push_caller_value(cluster_manager, CONTRACT_FEE_LIMIT);
        let storage_cluster_id = contract.cluster_create(cluster_manager, partition_count, node_ids, topology.to_string().unwrap())?;
        pop_caller();

        for node in &mut storage_nodes {
            node.vnode.join_cluster(storage_cluster_id);
        }
    }

    // Provide one gateway VNode.
    let mut gateway_node = TestGateway::new(&mut contract, accounts.alice, "alice");

    // Create a gateway Cluster.
    {
        let node_ids = vec![gateway_node.vnode.vnode_id];

        let topology = Topology::new(GATEWAY_ENGINE, 1);

        push_caller_value(cluster_manager, CONTRACT_FEE_LIMIT);
        let gateway_cluster_id = contract.cluster_create(cluster_manager, partition_count, node_ids, topology.to_string().unwrap())?;
        pop_caller();

        gateway_node.vnode.join_cluster(gateway_cluster_id);
    }

    // Create a user with a storage bucket.
    let user = TestUser::new(&mut contract, accounts.bob)?;

    let mut execute_action = |action: Action, expect_nodes: &[usize]| {
        let request = user.make_request(&contract, action).unwrap();
        let storage_requests = gateway_node.handle_request(&contract, request).unwrap();

        // Forward requests to storage nodes.
        assert_eq!(storage_requests.len(), expect_nodes.len());
        for (request_i, &node_i) in expect_nodes.iter().enumerate() {
            storage_nodes[node_i].handle_request(&contract, &storage_requests[request_i]).unwrap();
        }
    };

    // Simulate write requests to the gateway into different shards.
    let routing0 = (u32::MAX / partition_count as u32) * 0 + 123;
    let routing1 = (u32::MAX / partition_count as u32) * 1 + 123;
    let routing4 = (u32::MAX / partition_count as u32) * 4 + 123;

    execute_action(
        Action { routing_key: routing0, data: "data in shard 0".to_string(), op: Op::Write },
        &[0, 1, 2]);
    execute_action(
        Action { routing_key: routing1, data: "data in shard 1".to_string(), op: Op::Write },
        &[1, 2, 3]);
    execute_action(
        Action { routing_key: routing4, data: "data in shard 4".to_string(), op: Op::Write },
        &[4, 5, 0]);

    // Simulate read requests to the gateway.
    execute_action(
        Action { routing_key: routing0, data: "data in shard 0".to_string(), op: Op::Read },
        &[0, 1, 2]);
    execute_action(
        Action { routing_key: routing1, data: "data in shard 1".to_string(), op: Op::Read },
        &[1, 2, 3]);
    execute_action(
        Action { routing_key: routing4, data: "data in shard 4".to_string(), op: Op::Read },
        &[4, 5, 0]);

    // Replace a node.
    let old_vnode_id = storage_nodes.first().unwrap().vnode.vnode_id;
    let new_vnode_id = storage_nodes.last().unwrap().vnode.vnode_id;
    replace_node(&mut contract, old_vnode_id, new_vnode_id);
}
