use ink_lang as ink;

use crate::ddc_bucket::account::entity::Account;
use crate::ddc_bucket::cluster::entity::ClusterInfo;
use crate::ddc_bucket::flow::Flow;
use crate::ddc_bucket::node::entity::{NodeStatusInCluster, NodeKey};
use crate::ddc_bucket::schedule::{Schedule, MS_PER_MONTH};
use crate::ddc_bucket::Error::*;
use crate::ddc_bucket::*;

use super::env_utils::*;

fn setup() -> DdcBucket {
    set_caller(admin_id());
    set_callee(contract_id());
    let contract = DdcBucket::new();
    set_balance(contract_id(), 10);
    contract
}

const KB_PER_GB: Balance = 1_048_576;

struct TestCluster {
    contract: DdcBucket,

    provider_id0: AccountId,
    provider_id1: AccountId,
    provider_id2: AccountId,

    node_key0: NodeKey,
    node_key1: NodeKey,
    node_key2: NodeKey,
    node_params0: NodeParams,
    node_params1: NodeParams,
    node_params2: NodeParams,
    v_nodes0: Vec<VNodeToken>,
    v_nodes1: Vec<VNodeToken>,
    v_nodes2: Vec<VNodeToken>,

    cdn_node_key0: CdnNodeKey,
    cdn_node_key1: CdnNodeKey,
    cdn_node_key2: CdnNodeKey,
    cdn_node_params0: CdnNodeParams,
    cdn_node_params1: CdnNodeParams,
    cdn_node_params2: CdnNodeParams,

    manager_id: AccountId,
    cluster_id: ClusterId,
    cluster_params: ClusterParams,
    cluster_v_nodes: Vec<VNodeToken>,
    nodes_keys: Vec<NodeKey>,
    cdn_nodes_keys: Vec<CdnNodeKey>,
    rent_per_v_node: Balance,
    capacity: u32,
    reserved_resource: u32,
}

fn new_cluster() -> TestCluster {

    let mut contract: DdcBucket = setup();

    let provider_id0 = AccountId::from([0xae, 0x7d, 0xe8, 0x17, 0xa4, 0xa5, 0x12, 0x57, 0xd2, 0x49, 0x64, 0x28, 0x3b, 0x25, 0x69, 0x09, 0xdf, 0x0c, 0x99, 0x97, 0xc0, 0x3e, 0x2b, 0x88, 0x02, 0x02, 0xee, 0x10, 0xf4, 0x4d, 0x72, 0x48]);
    let provider_id1 = AccountId::from([0xc4, 0xba, 0xfd, 0x6a, 0xa1, 0x5a, 0x14, 0xd6, 0xee, 0xf2, 0xea, 0x92, 0xb7, 0xc6, 0x84, 0x51, 0x68, 0x39, 0xbe, 0x96, 0xd6, 0xbf, 0xca, 0xa3, 0x68, 0xd2, 0x4f, 0xff, 0x09, 0x85, 0xa7, 0x1e]);
    let provider_id2 = AccountId::from([0xfa, 0x01, 0x28, 0xf8, 0xe1, 0x32, 0xc6, 0x81, 0x21, 0x06, 0xa5, 0xce, 0xae, 0x6d, 0xcf, 0xf3, 0xd2, 0xc0, 0x1b, 0xb0, 0x13, 0xf2, 0xd7, 0x75, 0x6f, 0x20, 0xf9, 0x50, 0x00, 0xd6, 0xc7, 0x2b]);
    let manager_id = AccountId::from([0xd2, 0xc5, 0xea, 0xa2, 0x0c, 0xd0, 0x4e, 0xfb, 0x3f, 0x10, 0xb8, 0xad, 0xa9, 0xa4, 0x4f, 0xe0, 0x85, 0x41, 0x1f, 0x59, 0xf2, 0x34, 0x1a, 0x92, 0xa3, 0x48, 0x4f, 0x04, 0x51, 0x87, 0x68, 0x54]);

    set_balance(provider_id0, 1000 * TOKEN);
    set_balance(provider_id1, 1000 * TOKEN);
    set_balance(provider_id2, 1000 * TOKEN);
    set_balance(manager_id, 1000 * TOKEN);

    let rent_per_v_node: Balance = 10 * TOKEN;
    let reserved_resource = 10;
    let capacity = 100;


    // Create the 1st storage node
    let node_key0 = AccountId::from([0x0a; 32]);
    let node_params0 = NodeParams::from("{\"url\":\"https://ddc.cere.network/storage/0\"}");
    let capacity0 = capacity;
    set_caller_value(provider_id0, CONTRACT_FEE_LIMIT);
    contract.node_create(
        node_key0,
        node_params0.clone(),
        capacity0,
        rent_per_v_node
    ).unwrap();


    // Create the 2nd storage node
    let node_key1 = AccountId::from([0x0b; 32]);
    let node_params1 = NodeParams::from("{\"url\":\"https://ddc-1.cere.network/storage/1\"}");
    let capacity1 = capacity;
    set_caller_value(provider_id1, CONTRACT_FEE_LIMIT);
    contract.node_create(
        node_key1,
        node_params1.clone(),
        capacity1,
        rent_per_v_node
    ).unwrap();


    // Create the 3rd storage node
    let node_key2 = AccountId::from([0x0c; 32]);
    let node_params2 = NodeParams::from("{\"url\":\"https://ddc-2.cere.network/storage/2\"}");
    let capacity2 = capacity;
    set_caller_value(provider_id2, CONTRACT_FEE_LIMIT);
    let node_key2 = contract.node_create(
        node_key2,
        node_params2.clone(),
        capacity2,
        rent_per_v_node,
    ).unwrap();


    // Create the 1st cdn node
    let cdn_node_key0 = AccountId::from([0x0d; 32]);
    let cdn_node_params0 = CdnNodeParams::from("{\"url\":\"https://ddc.cere.network/cdn/0\"}");
    set_caller_value(provider_id0, CONTRACT_FEE_LIMIT);
    contract.cdn_node_create(
        cdn_node_key0, 
        cdn_node_params0.clone()
    ).unwrap();


    // Create the 2nd cdn node
    let cdn_node_key1 = AccountId::from([0x0e; 32]);
    let cdn_node_params1 = CdnNodeParams::from("{\"url\":\"https://ddc.cere.network/cdn/1\"}");
    set_caller_value(provider_id1, CONTRACT_FEE_LIMIT);
    contract.cdn_node_create(
        cdn_node_key1, 
        cdn_node_params1.clone()
    ).unwrap();


    // Create the 3rd cdn node
    let cdn_node_key2 = AccountId::from([0x0f; 32]);
    let cdn_node_params2 = CdnNodeParams::from("{\"url\":\"https://ddc.cere.network/cdn/2\"}");
    set_caller_value(provider_id2, CONTRACT_FEE_LIMIT);
    contract.cdn_node_create(
        cdn_node_key2, 
        cdn_node_params2.clone()
    ).unwrap();


    // Create a Cluster
    let cluster_params = ClusterParams::from("{}");
    set_caller_value(manager_id, CONTRACT_FEE_LIMIT);
    let cluster_id = contract.cluster_create(
        cluster_params.clone(),
    ).unwrap();


    // Grant trusted manager_id permission from node providers to the cluster manager_id
    for provider_id in [provider_id0, provider_id1, provider_id2] {
        set_caller_value(provider_id, CONTRACT_FEE_LIMIT);
        contract.grant_trusted_manager_permission(manager_id).unwrap();
        let expected_perm = Permission::ClusterManagerTrustedBy(provider_id);
        assert!(contract.has_permission(manager_id, expected_perm));
    }


    // Add the 1st storage node to the Cluster
    let v_nodes0 = vec![1, 2, 3];
    set_caller_value(manager_id, CONTRACT_FEE_LIMIT);
    contract.cluster_add_node(
        cluster_id, 
        node_key0, 
        v_nodes0.clone()
    ).unwrap();


    // Add the 2nd storage node to the Cluster
    let v_nodes1 = vec![4, 5, 6];
    set_caller_value(manager_id, CONTRACT_FEE_LIMIT);
    contract.cluster_add_node(
        cluster_id, 
        node_key1, 
        v_nodes1.clone()
    ).unwrap();


    // Add the 3rd storage node to the Cluster
    let v_nodes2 = vec![7, 8, 9];
    set_caller_value(manager_id, CONTRACT_FEE_LIMIT);
    contract.cluster_add_node(
        cluster_id, 
        node_key2, 
        v_nodes2.clone()
    ).unwrap();


    // Add the 1st cdn node to the Cluster
    set_caller_value(manager_id, CONTRACT_FEE_LIMIT);
    contract.cluster_add_cdn_node(
        cluster_id, 
        cdn_node_key0, 
    ).unwrap();


    // Add the 2nd cdn node to the Cluster
    set_caller_value(manager_id, CONTRACT_FEE_LIMIT);
    contract.cluster_add_cdn_node(
        cluster_id, 
        cdn_node_key1, 
    ).unwrap();


    // Add the 3rd cdn node to the Cluster
    set_caller_value(manager_id, CONTRACT_FEE_LIMIT);
    contract.cluster_add_cdn_node(
        cluster_id, 
        cdn_node_key2, 
    ).unwrap();


    set_caller(manager_id);
    contract.cluster_reserve_resource(cluster_id, reserved_resource);


    let nodes_keys = vec![
        node_key0,
        node_key1,
        node_key2
    ];

    let cdn_nodes_keys = vec![
        cdn_node_key0,
        cdn_node_key1,
        cdn_node_key2
    ];

    let mut cluster_v_nodes = Vec::<VNodeToken>::new();
    cluster_v_nodes.extend(v_nodes0.clone());
    cluster_v_nodes.extend(v_nodes1.clone());
    cluster_v_nodes.extend(v_nodes2.clone());

    TestCluster {
        contract,

        provider_id0,
        provider_id1,
        provider_id2,

        node_key0,
        node_key1,
        node_key2,
        node_params0,
        node_params1,
        node_params2,
        v_nodes0,
        v_nodes1,
        v_nodes2,

        cdn_node_key0,
        cdn_node_key1,
        cdn_node_key2,
        cdn_node_params0,
        cdn_node_params1,
        cdn_node_params2,

        manager_id,
        cluster_id,
        cluster_params,
        cluster_v_nodes,
        rent_per_v_node,
        nodes_keys,
        cdn_nodes_keys,
        capacity,
        reserved_resource,
    }
}


struct TestBucket {
    bucket_id: BucketId,
    owner_id: AccountId,
    resource: u32,
}

fn new_bucket(ctx: &mut TestCluster) -> TestBucket {
    let owner_id = AccountId::from([0xd4, 0x8f, 0x63, 0x67, 0xe2, 0x15, 0x51, 0xdf, 0x11, 0x1c, 0x92, 0x69, 0x0d, 0x04, 0x3f, 0x75, 0xcb, 0x39, 0xf8, 0x27, 0xbb, 0xc7, 0x46, 0x4b, 0x8d, 0x5d, 0x70, 0xd1, 0x02, 0xaa, 0x71, 0x0a]);
    set_balance(owner_id, 1000 * TOKEN);
    set_caller_value(owner_id, CONTRACT_FEE_LIMIT);

    let bucket_id = ctx.contract.bucket_create(
        "{}".to_string(), 
        ctx.cluster_id, 
        None
    );

    // Reserve some resources for the bucket from the cluster.
    set_caller_value(owner_id, CONTRACT_FEE_LIMIT);
    let resource = 1;
    ctx.contract.bucket_alloc_into_cluster(bucket_id, resource);

    // Deposit some value to pay for buckets.
    set_caller_value(owner_id, 10 * TOKEN);
    ctx.contract.account_deposit();

    TestBucket {
        bucket_id,
        owner_id,
        resource,
    }
}

#[ink::test]
fn cluster_create_works() {
    let ctx = new_cluster();
    let providers_ids = &[ctx.provider_id0, ctx.provider_id1, ctx.provider_id2];
    let node_keys = &[ctx.node_key0, ctx.node_key1, ctx.node_key2];
    let cdn_node_keys = &[ctx.cdn_node_key0, ctx.cdn_node_key1, ctx.cdn_node_key2];
    let node_params = &[ctx.node_params0.clone(), ctx.node_params1.clone(), ctx.node_params2.clone()];
    let cdn_node_params = &[ctx.cdn_node_params0.clone(), ctx.cdn_node_params1.clone(), ctx.cdn_node_params2.clone()];

    assert_eq!(ctx.cluster_id, 0, "cluster_id must start at 0");

    // Check cluster nodes
    {
        let node0 = ctx.contract.node_get(ctx.node_key0)?;
        let v_nodes0 = ctx.contract.get_v_nodes_by_node(ctx.node_key0);

        assert_eq!(
            node0,
            NodeInfo {
                node_key: ctx.node_key0,
                node: Node {
                    provider_id: ctx.provider_id0,
                    rent_per_month: ctx.rent_per_v_node,
                    free_resource: ctx.capacity - ctx.reserved_resource * 3,
                    node_params: ctx.node_params0.clone(),
                    cluster_id: Some(ctx.cluster_id),
                    status_in_cluster: Some(NodeStatusInCluster::ADDING),
                },
                v_nodes: v_nodes0
            }
        );

        let node1 = ctx.contract.node_get(ctx.node_key1)?;
        let v_nodes1 = ctx.contract.get_v_nodes_by_node(ctx.node_key1);

        assert_eq!(
            node1,
            NodeInfo {
                node_key: ctx.node_key1,
                node: Node {
                    provider_id: ctx.provider_id1,
                    rent_per_month: ctx.rent_per_v_node,
                    free_resource: ctx.capacity - ctx.reserved_resource * 3,
                    node_params: ctx.node_params1.clone(),
                    cluster_id: Some(ctx.cluster_id),
                    status_in_cluster: Some(NodeStatusInCluster::ADDING),
                },
                v_nodes: v_nodes1
            }
        );

        let node2 = ctx.contract.node_get(ctx.node_key2)?;
        let v_nodes2 = ctx.contract.get_v_nodes_by_node(ctx.node_key2);

        assert_eq!(
            node2,
            NodeInfo {
                node_key: ctx.node_key2,
                node: Node {
                    provider_id: ctx.provider_id2,
                    rent_per_month: ctx.rent_per_v_node,
                    free_resource: ctx.capacity - ctx.reserved_resource * 3,
                    node_params: ctx.node_params2.clone(),
                    cluster_id: Some(ctx.cluster_id),
                    status_in_cluster: Some(NodeStatusInCluster::ADDING),
                },
                v_nodes: v_nodes2
            }
        );
    }

    // Check the cluster
    {
        let cluster = ctx.contract.cluster_get(ctx.cluster_id)?;
        let cluster_v_nodes = ctx.contract.get_v_nodes_by_cluster(ctx.cluster_id);
        assert_eq!(
            cluster,
            ClusterInfo {
                cluster_id: ctx.cluster_id,
                cluster: Cluster {
                    manager_id: ctx.manager_id,
                    nodes_keys: ctx.nodes_keys,
                    resource_per_v_node: ctx.reserved_resource,
                    resource_used: 0,
                    cluster_params: ctx.cluster_params.clone(),
                    revenues: Cash(0),
                    total_rent: ctx.rent_per_v_node * ctx.cluster_v_nodes.len() as Balance,
                    cdn_nodes_keys: ctx.cdn_nodes_keys,
                    cdn_usd_per_gb: 104_857_600,
                    cdn_revenues: Cash(0),
                },
                cluster_v_nodes
            }
        );
    }

    // Check emitted events
    let mut events = get_events();
    events.reverse(); // Work with pop().

    // Storage node created event
    for i in 0..3 {
        assert!(matches!(events.pop().unwrap(), Event::NodeCreated(ev) if ev ==
            NodeCreated {
                node_key: node_keys[i],
                provider_id: providers_ids[i],
                rent_per_month: ctx.rent_per_v_node,
                node_params: node_params[i].clone()
            })
        );
    }

    // CDN node created event
    for i in 0..3 {
        assert!(matches!(events.pop().unwrap(), Event::CdnNodeCreated(ev) if ev ==
            CdnNodeCreated {
                cdn_node_key: cdn_node_keys[i],
                provider_id: providers_ids[i],
                cdn_node_params: cdn_node_params[i].clone(),
                undistributed_payment: 0
            })
        );
    }

    // Cluster created event
    assert!(
        matches!(events.pop().unwrap(), Event::ClusterCreated(ev) if ev ==
        ClusterCreated { 
            cluster_id: ctx.cluster_id, 
            manager: ctx.manager_id, 
            cluster_params: ctx.cluster_params.clone()
        })
    );

    // Permission granted event
    for provider_id in providers_ids {
        assert!(
            matches!(events.pop().unwrap(), Event::PermissionGranted(ev) if ev ==
            PermissionGranted { 
                account_id: ctx.manager_id, 
                permission: Permission::ClusterManagerTrustedBy(*provider_id) 
            })
        );
    }

    // Cluster storage node added event
    for i in 0..3 {
        assert!(matches!(events.pop().unwrap(), Event::ClusterNodeAdded(ev) if ev ==
            ClusterNodeAdded {
                cluster_id: ctx.cluster_id,
                node_key: node_keys[i]
            })
        );
    }

    // Cluster cdn node added event
    for i in 0..3 {
        assert!(matches!(events.pop().unwrap(), Event::ClusterCdnNodeAdded(ev) if ev ==
            ClusterCdnNodeAdded {
                cluster_id: ctx.cluster_id,
                cdn_node_key: cdn_node_keys[i]
            })
        );
    }

    // Cluster resource reserved event
    assert!(
        matches!(events.pop().unwrap(), Event::ClusterReserveResource(ev) if ev ==
        ClusterReserveResource { 
            cluster_id: ctx.cluster_id, 
            resource: ctx.reserved_resource 
        })
    );

    assert_eq!(events.len(), 0, "All events must be checked");

}


#[ink::test]
fn cluster_replace_node_only_manager() {
    let mut ctx = new_cluster();

    let not_manager = AccountId::from([0xee, 0x0a, 0xc9, 0x58, 0xa2, 0x0d, 0xe8, 0xda, 0x73, 0xb2, 0x05, 0xe9, 0xc6, 0x34, 0xa6, 0xb2, 0x23, 0xcc, 0x54, 0x30, 0x24, 0x5d, 0x89, 0xb6, 0x4d, 0x83, 0x9b, 0x6d, 0xca, 0xc4, 0xf8, 0x6d]);
    set_caller_value(not_manager, 0);

    // Reassign a vnode from node1 to node2.
    assert_eq!(
        ctx.contract.cluster_replace_node(
            ctx.cluster_id, 
            vec![1, 2, 3], 
            ctx.node_key2
        ),
        Err(OnlyClusterManager)
    );
}

#[ink::test]
fn cluster_replace_node_works() {
    let mut ctx = new_cluster();

    set_caller(ctx.manager_id);
    // Reassign a vnode from node0 to node2.
    ctx.contract.cluster_replace_node(
        ctx.cluster_id, 
        vec![1, 3], 
        ctx.node_key2
    )?;

    // Check the last event
    let ev = get_events().pop().unwrap();
    assert!(matches!(ev, Event::ClusterNodeReplaced(ev) if ev ==
        ClusterNodeReplaced { 
            cluster_id: ctx.cluster_id, 
            node_key: ctx.node_key2
        }
    ));

    let mut cluster_v_nodes = Vec::<VNodeToken>::new();
    cluster_v_nodes.extend(vec![2]);
    cluster_v_nodes.extend(ctx.v_nodes1.clone());
    cluster_v_nodes.extend(ctx.v_nodes2.clone());
    cluster_v_nodes.extend(vec![1, 3]);
    cluster_v_nodes.sort();

    let mut cluster_info = ctx.contract.cluster_get(ctx.cluster_id)?;
    cluster_info.cluster_v_nodes.sort();
    assert_eq!(&cluster_info.cluster_v_nodes, &cluster_v_nodes, "a v_node must be replaced");

    let mut v_nodes0 = ctx.contract.get_v_nodes_by_node(ctx.node_key0.clone());
    v_nodes0.sort();
    let mut v_nodes1 = ctx.contract.get_v_nodes_by_node(ctx.node_key1.clone());
    v_nodes1.sort();
    let mut v_nodes2 = ctx.contract.get_v_nodes_by_node(ctx.node_key2.clone());
    v_nodes2.sort();

    assert_eq!(&v_nodes0, &vec![2], "v_nodes must be replaced for the 1st node");
    assert_eq!(&v_nodes1, &vec![4, 5, 6], "v_nodes must not be replaced for the 2nd node");
    assert_eq!(&v_nodes2, &vec![1, 3, 7, 8, 9], "v_nodes must be assigned to the 3rd node");

    // Check the changed state of the nodes.
    let expected_resources = [
        (ctx.node_key0, 100 - 10),
        (ctx.node_key1, 100 - 10 - 10 - 10),
        (ctx.node_key2, 100 - 10 - 10 - 10 - 10 - 10),
    ];

    for (node_key, available) in expected_resources {
        let node_info = ctx.contract.node_get(node_key).unwrap();
        assert_eq!(
            node_info.node.free_resource, available,
            "resources must have shifted between nodes"
        );
    }
}


#[ink::test]
fn cluster_reserve_works() {
    let mut ctx = new_cluster();
    set_caller(ctx.manager_id);

    // Reserve more resources.
    ctx.contract.cluster_reserve_resource(ctx.cluster_id, 5);

    // Check the last event.
    let ev = get_events().pop().unwrap();
    assert!(matches!(ev, Event::ClusterReserveResource(ev) if ev ==
        ClusterReserveResource { 
            cluster_id: ctx.cluster_id, 
            resource: 5 
        }
    ));

    // Check the changed state of the cluster.
    let cluster = ctx.contract.cluster_get(ctx.cluster_id)?.cluster;
    assert_eq!(cluster.resource_per_v_node, 10 + 5);

    // Check the changed state of the nodes.
    let expected_resources = [
        (ctx.node_key0, 100 - 40 - 5),
        (ctx.node_key1, 100 - 40 - 5),
        (ctx.node_key2, 100 - 40 - 5),
    ];
    for (node_id, available) in expected_resources {
        assert_eq!(
            ctx.contract.node_get(node_id)?.node.free_resource,
            available,
            "more resources must be reserved from the nodes"
        );
    }
}


#[ink::test]
fn cluster_management_validation_works() {
    let mut ctx = new_cluster();
    
    let not_manager = AccountId::from([0xee, 0x0a, 0xc9, 0x58, 0xa2, 0x0d, 0xe8, 0xda, 0x73, 0xb2, 0x05, 0xe9, 0xc6, 0x34, 0xa6, 0xb2, 0x23, 0xcc, 0x54, 0x30, 0x24, 0x5d, 0x89, 0xb6, 0x4d, 0x83, 0x9b, 0x6d, 0xca, 0xc4, 0xf8, 0x6d]);
    set_balance(not_manager, 1000 * TOKEN);

    set_caller(not_manager);
    assert_eq!(
        ctx.contract.cluster_replace_node(
            ctx.cluster_id, 
            vec![1, 2, 3],
            ctx.node_key1.clone()
        ),
        Err(OnlyClusterManager),
        "only the manager_id can modify the cluster"
    );

    let bad_node_id = AccountId::from([0xf6, 0x8f, 0x06, 0xa8, 0x26, 0xba, 0xaf, 0x7f, 0xbd, 0x9b, 0xff, 0x3d, 0x1e, 0xec, 0xae, 0xef, 0xc7, 0x7a, 0x01, 0x6d, 0x0b, 0xaf, 0x4c, 0x90, 0x55, 0x6e, 0x7b, 0x15, 0x73, 0x46, 0x9c, 0x76]);
    set_caller(ctx.manager_id);
    assert_eq!(
        ctx.contract
            .cluster_replace_node(
                ctx.cluster_id, 
                vec![1, 2, 3],
                bad_node_id
            ),
        Err(NodeDoesNotExist),
        "cluster replacement node must exist"
    );

}

#[ink::test]
fn cdn_cluster_gas_converter_works() {
    println!("Creating new cdn cluster");

    let mut ctx = new_cluster();

    // The provider stops trusting the manager_id.
    println!("Cdn cluster id is {}", ctx.cluster_id);
    set_caller(ctx.manager_id);
    ctx.contract.cdn_set_rate(ctx.cluster_id, 3_750_000_000);
    set_caller(ctx.provider_id0);
    let rate = ctx.contract.cdn_get_rate(ctx.cluster_id);

    let usd_per_cere = TOKEN / 100;
    set_caller(admin_id());
    ctx.contract.account_set_usd_per_cere(usd_per_cere);

    let usd_amount = ctx.contract.account_get_usd_per_cere();
    println!("Current usd amount is {}", usd_amount);

    println!("The current rate is {}", rate);

    let usd_per_kb = rate / KB_PER_GB;
    println!("The current rate per kb {}", usd_per_kb);

    let cere_per_kb = ctx.contract.accounts.1.to_cere(usd_per_kb);
    println!("The current cere rate per kb {}", cere_per_kb);
}


#[ink::test]
fn cdn_cluster_payment_works() {
    // todo: this test scenario must be revised as it does pure printing without any assertion
    println!("Creating new cdn cluster");

    let mut ctx = new_cluster();

    // The provider stops trusting the manager_id.
    println!("Cluster id is {}", ctx.cluster_id);
    set_caller(ctx.provider_id0);

    let usd_per_cere = TOKEN / 100;
    set_caller(admin_id());
    ctx.contract.account_set_usd_per_cere(usd_per_cere);

    let usd_amount = ctx.contract.account_get_usd_per_cere();
    println!("Current usd amount is {}", usd_amount);

    let rate = ctx.contract.cdn_get_rate(ctx.cluster_id);
    println!("The current rate is {}", rate);

    let usd_per_kb = rate / KB_PER_GB;
    println!("The current rate per kb {}", usd_per_kb);

    let cere_per_kb = ctx.contract.accounts.1.to_cere(usd_per_kb);
    println!("The current cere rate per kb {}", cere_per_kb);

    set_caller_value(ctx.provider_id0, 10 * TOKEN);
    ctx.contract.account_deposit();

    set_caller_value(ctx.provider_id0, 10 * TOKEN);
    ctx.contract.account_bond(5 * TOKEN);

    set_caller(ctx.provider_id0);
    ctx.contract.set_fee_bp(1_000);

    let account0_before_putting = ctx.contract.accounts.get(&ctx.provider_id0).unwrap();
    println!("Before putting revenue: {:?}", account0_before_putting);

    ctx.contract.cdn_cluster_put_revenue(
        ctx.cluster_id,
        vec![(ctx.provider_id0, 1000), (ctx.provider_id0, 541643)],
        vec![(ctx.cdn_node_key0, 1000), (ctx.cdn_node_key1, 541643)],
        vec![],
        5,
    );
    let account0_after_putting = ctx.contract.accounts.get(&ctx.provider_id0).unwrap();
    println!("After putting revenue: {:?}", account0_after_putting);

    let cluster_list_1 = ctx.contract.cluster_list(0, 10, None);
    println!("Cluster list one {:?}", cluster_list_1);
    let cdn_node0 = ctx.contract.cdn_nodes.get(ctx.cdn_node_key0).unwrap();
    let cdn_node1 = ctx.contract.cdn_nodes.get(ctx.cdn_node_key1).unwrap();

    println!("{:?}", cdn_node0);
    println!("{:?}", cdn_node1);
    let cluster0 = ctx.contract.clusters.get(ctx.cluster_id);
    println!("{:?}", cluster0);
    let cluster_list = ctx.contract.cluster_list(0, 10, None);
    println!("{:?}", cluster0);
    println!("{:?}", cluster_list);

    let validated_commit_node0 = ctx.contract.get_validated_commit(ctx.cdn_node_key0);
    println!("Validated commit: {:?}", validated_commit_node0);

    let fee = ctx.contract.get_fee_bp();
    println!("Protocol fee in basis points: {:?}", fee);

    let protocol_revenues = ctx.contract.get_protocol_revenues();
    println!("Protocol revenues: {:?}", protocol_revenues);

    set_caller(ctx.provider_id0);
    ctx.contract.cdn_cluster_distribute_revenues(ctx.cluster_id);

    let cdn_node0 = ctx.contract.cdn_nodes.get(ctx.cdn_node_key0).unwrap();
    let cdn_node1 = ctx.contract.cdn_nodes.get(ctx.cdn_node_key1).unwrap();
    println!("{:?}", cdn_node0);
    println!("{:?}", cdn_node1);

    let cdn_cluster_list = ctx.contract.cluster_list(0, 10, None);
    println!("{:?}", cdn_cluster_list);

    let account0_after_distributing = ctx.contract.accounts.get(&ctx.provider_id0).unwrap();
    println!("{:?}", account0_after_distributing);

}

fn bucket_settle_payment(ctx: &mut TestCluster, test_bucket: &TestBucket) {
    // Go to the future when some revenues are due.
    advance_block::<DefaultEnvironment>();
    // Pay the due thus far.
    set_caller_value(ctx.manager_id, CONTRACT_FEE_LIMIT);
    ctx.contract.bucket_settle_payment(test_bucket.bucket_id);
}

#[ink::test]
fn bucket_pays_cluster() {
    let ctx = &mut new_cluster();
    let test_bucket = &new_bucket(ctx);
    do_bucket_pays_cluster(ctx, test_bucket, 1).unwrap();
}

#[ink::test]
fn bucket_pays_cluster_at_new_rate() {
    let ctx = &mut new_cluster();

    let test_bucket = &new_bucket(ctx);
    // Set up an exchange rate manager_id.
    set_caller(admin_id());
    ctx.contract
        .admin_grant_permission(admin_id(), Permission::SetExchangeRate).unwrap();


    // Change the currency exchange rate.
    let usd_per_cere = 2;
    set_caller(admin_id());
    ctx.contract.account_set_usd_per_cere(usd_per_cere * TOKEN);

    do_bucket_pays_cluster(ctx, test_bucket, usd_per_cere).unwrap();
}

fn do_bucket_pays_cluster(
    ctx: &mut TestCluster,
    test_bucket: &TestBucket,
    usd_per_cere: Balance,
) -> Result<()> {
    let expected_rent = ctx.rent_per_v_node * ctx.cluster_v_nodes.len() as Balance;

    // Check the state before payment.
    let before = ctx
        .contract
        .account_get(test_bucket.owner_id)?
        .deposit
        .peek();
    let bucket = ctx.contract.bucket_get(test_bucket.bucket_id)?.bucket;
    assert_eq!(bucket.owner_id, test_bucket.owner_id);
    /* TODO: Not testable at the moment, see struct BucketInStatus.
    assert_eq!(bucket.flow,
               Flow {
                   from: test_bucket.owner_id,
                   schedule: Schedule::new(0, expected_rent),
               });
    */
    let timestamp_before = block_timestamp::<DefaultEnvironment>();
    bucket_settle_payment(ctx, &test_bucket);
    let timestamp_after = block_timestamp::<DefaultEnvironment>();

    // Check the last event.
    let ev = get_events().pop().unwrap();
    assert!(matches!(ev, Event::BucketSettlePayment(ev) if ev ==
        BucketSettlePayment {  
            bucket_id: test_bucket.bucket_id, 
            cluster_id: ctx.cluster_id 
        }
    ));

    // Check the state after payment.
    let after = ctx
        .contract
        .account_get(test_bucket.owner_id)?
        .deposit
        .peek();
    let spent = before - after;
    /* TODO: Not testable at the moment, see struct BucketInStatus.
    let bucket = ctx.contract.bucket_get(test_bucket.bucket_id)?.bucket;
    assert_eq!(bucket.flow,
               Flow {
                   from: test_bucket.owner_id,
                   schedule: Schedule::new(BLOCK_TIME, expected_rent),
               });
    */
    let timespan = timestamp_after - timestamp_before;
    let expect_revenues_usd = expected_rent * timespan as u128 / MS_PER_MONTH as u128;
    let expect_revenues = expect_revenues_usd / usd_per_cere;
    assert!(expect_revenues > 0);
    assert_eq!(
        expect_revenues, spent,
        "revenues must come from the bucket owner"
    );

    let cluster = ctx.contract.cluster_get(ctx.cluster_id)?.cluster;
    assert_eq!(
        cluster.revenues.peek(),
        expect_revenues,
        "must get revenues into the cluster"
    );

    Ok(())
}

#[ink::test]
fn cluster_add_node_works() {
    let mut ctx = new_cluster();

    let new_provider_id = AccountId::from([0x3c, 0x08, 0xea, 0xa6, 0x89, 0xdf, 0x45, 0x2b, 0x77, 0xa1, 0xa5, 0x6b, 0x83, 0x10, 0x1e, 0x31, 0x06, 0xc9, 0xc7, 0xaf, 0xb3, 0xe9, 0xfd, 0x6f, 0xa6, 0x2b, 0x50, 0x00, 0xf6, 0xeb, 0xcb, 0x5a]);
    set_balance(new_provider_id, 1000 * TOKEN);

    let new_node_key = AccountId::from([0xc4, 0xcd, 0xaa, 0xfa, 0xf1, 0x30, 0x7d, 0x23, 0xf4, 0x99, 0x84, 0x71, 0xdf, 0x78, 0x59, 0xce, 0x06, 0x3d, 0xce, 0x78, 0x59, 0xc4, 0x3a, 0xe8, 0xef, 0x12, 0x0a, 0xbc, 0x43, 0xc4, 0x84, 0x31]);
    let rent_per_month = 100;
    let node_params = NodeParams::from("new_node");
    let capacity = 1000;

    set_caller_value(new_provider_id, CONTRACT_FEE_LIMIT);
    let new_node_key = ctx.contract.node_create(
        new_node_key,
        node_params.clone(),
        capacity,
        rent_per_month
    )?;

    set_caller_value(new_provider_id, CONTRACT_FEE_LIMIT);
    ctx.contract.grant_trusted_manager_permission(ctx.manager_id)?;
    assert!(
        matches!(
            get_events().pop().unwrap(), Event::PermissionGranted(ev) if ev == 
            PermissionGranted { 
                account_id: ctx.manager_id, 
                permission: Permission::ClusterManagerTrustedBy(new_provider_id) 
            }
        )
    );

    let new_v_nodes = vec![10, 11, 12];
    set_caller_value(ctx.manager_id, CONTRACT_FEE_LIMIT);
    ctx.contract.cluster_add_node(
        ctx.cluster_id, 
        new_node_key, 
        new_v_nodes.clone()
    )?;

    let nodes_keys = vec![
        ctx.node_key0,
        ctx.node_key1,
        ctx.node_key2,
        new_node_key,
    ];

    let cluster_v_nodes = vec![
        ctx.v_nodes0,
        ctx.v_nodes1,
        ctx.v_nodes2,
        new_v_nodes
    ];

    let mut cluster_info = ctx.contract.cluster_get(ctx.cluster_id)?;
    cluster_info.cluster_v_nodes.sort();
    assert!(matches!(cluster_info.cluster.nodes_keys, nodes_keys));
    assert!(matches!(cluster_info.cluster_v_nodes, cluster_v_nodes));
}


#[ink::test]
fn cluster_pays_providers() {
    let ctx = &mut new_cluster();
    let test_bucket = &new_bucket(ctx);
    bucket_settle_payment(ctx, &test_bucket);

    // Get state before the distribution.
    let to_distribute = ctx
        .contract
        .cluster_get(ctx.cluster_id)?
        .cluster
        .revenues
        .peek();

    let before0 = balance_of(ctx.provider_id0);
    let before1 = balance_of(ctx.provider_id1);
    let before2 = balance_of(ctx.provider_id2);
    let before_mgmt = balance_of(ctx.manager_id);

    let skip_events = get_events::<Event>().len();

    // Set a network fee.
    let network_fee_bp = 100; // 1%
    let cluster_management_fee_bp = 200; // 2%
    set_caller_value(admin_id(), CONTRACT_FEE_LIMIT);
    ctx.contract.admin_set_fee_config(FeeConfig {
        network_fee_bp,
        network_fee_destination: AccountId::default(),
        cluster_management_fee_bp,
    });

    let burned_fee = to_distribute * network_fee_bp / 10_000;
    let manager_fee = (to_distribute - burned_fee) * cluster_management_fee_bp / 10_000;
    let provider_fee: u128 = (to_distribute - burned_fee - manager_fee) / 3;

    // Distribute the revenues of the cluster to providers.
    ctx.contract.cluster_distribute_revenues(ctx.cluster_id);

    // Check the last events.
    let mut events = get_events();
    events.reverse(); // Work with pop().
    events.truncate(events.len() - skip_events);
    let expected_recipients = vec![ctx.provider_id0, ctx.provider_id1, ctx.provider_id2];

    for provider_id in expected_recipients {
        assert!(
            matches!(events.pop().unwrap(), Event::ClusterDistributeRevenues(ev) if ev ==
            ClusterDistributeRevenues { 
                cluster_id: ctx.cluster_id, 
                provider_id 
            })
        );
    }

    assert_eq!(events.len(), 0, "all events must be checked");

    // Get state after the distribution.
    let rounding_error = ctx
        .contract
        .cluster_get(ctx.cluster_id)?
        .cluster
        .revenues
        .peek();

    let earned0 = balance_of(ctx.provider_id0) - before0;
    let earned1 = balance_of(ctx.provider_id1) - before1;
    let earned2 = balance_of(ctx.provider_id2) - before2;
    let earned_mgmt = balance_of(ctx.manager_id) - before_mgmt;

    assert!(provider_fee > 0, "provider must earn something");
    assert_eq!(
        earned0, provider_fee,
        "providers must earn the correct amount"
    );
    assert_eq!(
        earned1, provider_fee,
        "providers must earn the correct amount"
    );
    assert_eq!(
        earned2, provider_fee,
        "providers must earn the correct amount"
    );

    assert!(burned_fee > 0, "the network must earn something");
    assert!(manager_fee > 0, "the manager_id must earn something");
    assert_eq!(
        earned_mgmt, manager_fee,
        "the manager_id must earn the correct amount"
    );

    assert!(to_distribute > 0);
    assert!(
        rounding_error < 10,
        "revenues must go out of the cluster (besides rounding)"
    );
    assert_eq!(
        earned0 + earned1 + earned2 + burned_fee + manager_fee + rounding_error,
        to_distribute,
        "all revenues must go to providers"
    );
}


#[ink::test]
fn bucket_create_works() {
    let ctx = &mut new_cluster();
    let test_bucket = &new_bucket(ctx);

    // Check the structure of the bucket including the payment flow.
    let total_rent = ctx.rent_per_v_node * ctx.cluster_v_nodes.len() as Balance;
    let expect_bucket = Bucket {
        owner_id: test_bucket.owner_id,
        cluster_id: ctx.cluster_id,
        flow: Flow {
            from: test_bucket.owner_id,
            schedule: Schedule::new(0, total_rent),
        },
        resource_reserved: test_bucket.resource,
        public_availability: false,
        resource_consumption_cap: 0,
    };

    // Check the status of the bucket.
    let bucket_status = ctx.contract.bucket_get(test_bucket.bucket_id)?;
    assert_eq!(
        bucket_status,
        BucketStatus {
            bucket_id: test_bucket.bucket_id,
            bucket: expect_bucket.into(),
            params: "{}".to_string(),
            writer_ids: vec![test_bucket.owner_id],
            reader_ids: vec![],
            rent_covered_until_ms: 297600000, // TODO: check this value.
        }
    );

    let mut events = get_events();
    events.reverse(); // Work with pop().
    events.truncate(8 - 3 - 2); // Skip 3 NodeCreated and 2 cluster setup events.

    // Create bucket.
    assert!(
        matches!(events.pop().unwrap(), Event::BucketCreated(ev) if ev ==
        BucketCreated {  bucket_id: test_bucket.bucket_id, owner_id: test_bucket.owner_id })
    );

    assert!(
        matches!(events.pop().unwrap(), Event::BucketAllocated(ev) if ev ==
        BucketAllocated { bucket_id: test_bucket.bucket_id, cluster_id: ctx.cluster_id, resource: test_bucket.resource })
    );

    // Deposit more.
    let net_deposit = 10 * TOKEN;
    assert!(matches!(events.pop().unwrap(), Event::Deposit(ev) if ev ==
        Deposit { account_id: test_bucket.owner_id, value: net_deposit }));

    assert_eq!(events.len(), 0, "all events must be checked");
}


#[ink::test]
fn account_deposit_works() {
    let account_id = AccountId::from([0x76, 0x95, 0x7c, 0xa6, 0xbe, 0xf5, 0xa3, 0x6d, 0x67, 0x0d, 0x3a, 0x84, 0xc6, 0x0a, 0xe2, 0xbb, 0xc9, 0x5e, 0xee, 0xde, 0x3a, 0x5f, 0x27, 0x0e, 0x26, 0xe3, 0x43, 0x4c, 0x46, 0xe2, 0x98, 0x10]);
    set_balance(account_id, 1000 * TOKEN);

    let mut contract = setup();

    assert_eq!(
        contract.account_get(account_id),
        Err(AccountDoesNotExist),
        "must not get a non-existent account"
    );

    let deposit = 10 * TOKEN;
    let deposit_after_fee = deposit;

    // Deposit some value.
    set_caller_value(account_id, deposit);
    contract.account_deposit();

    let account = contract.account_get(account_id)?;
    assert_eq!(
        account,
        Account {
            deposit: Cash(deposit_after_fee),
            payable_schedule: Schedule::empty(),
            bonded: Cash(0),
            unbonded_amount: Cash(0),
            negative: Cash(0),
            unbonded_timestamp: 0,
        },
        "must take deposit minus creation fee"
    );

    // Deposit more value.
    set_caller_value(account_id, deposit);
    contract.account_deposit();

    let account = contract.account_get(account_id)?;
    assert_eq!(
        account,
        Account {
            deposit: Cash(deposit_after_fee + deposit),
            payable_schedule: Schedule::empty(),
            bonded: Cash(0),
            unbonded_amount: Cash(0),
            negative: Cash(0),
            unbonded_timestamp: 0,
        },
        "must take more deposits without creation fee"
    );

    // Check events.
    let mut events = get_events();
    events.reverse(); // Work with pop().

    // First deposit event.
    assert!(matches!(events.pop().unwrap(), Event::Deposit(ev) if ev ==
        Deposit { account_id, value: deposit_after_fee }));

    // Second deposit event. No deposit_contract_fee because the account already exists.
    assert!(matches!(events.pop().unwrap(), Event::Deposit(ev) if ev ==
        Deposit { account_id, value: deposit }));

    assert_eq!(events.len(), 0, "all events must be checked");
}


#[ink::test]
fn node_set_params_works() {
    let mut ctx = new_cluster();

    // Change params.
    let new_node_params = NodeParams::from("new node params");
    set_caller_value(ctx.provider_id0, CONTRACT_FEE_LIMIT);
    ctx.contract.node_set_params(ctx.node_key0, new_node_params.clone())?;

    // Check the changed params.
    let status = ctx.contract.node_get(ctx.node_key0)?;
    assert_eq!(status.node.node_params, new_node_params);
}

#[ink::test]
fn node_set_params_only_owner() {
    let mut ctx = new_cluster();

    let not_provider = AccountId::from([0xee, 0x0a, 0xc9, 0x58, 0xa2, 0x0d, 0xe8, 0xda, 0x73, 0xb2, 0x05, 0xe9, 0xc6, 0x34, 0xa6, 0xb2, 0x23, 0xcc, 0x54, 0x30, 0x24, 0x5d, 0x89, 0xb6, 0x4d, 0x83, 0x9b, 0x6d, 0xca, 0xc4, 0xf8, 0x6d]);
    set_balance(not_provider, 1000 * TOKEN);
    // Change params.not_provider
    let new_node_params = NodeParams::from("new node params");
    set_caller_value(not_provider, CONTRACT_FEE_LIMIT);

    assert_eq!(
        ctx.contract.node_set_params(
            ctx.node_key0, 
            new_node_params
        ),
        Err(OnlyNodeOwner)
    );
}

#[ink::test]
fn cluster_change_params_works() {
    let mut ctx = new_cluster();

    // Change params.
    let new_cluster_params = NodeParams::from("new cluster params");
    set_caller_value(ctx.manager_id, CONTRACT_FEE_LIMIT);
    ctx.contract.cluster_set_params(ctx.cluster_id, new_cluster_params.clone())?;

    // Check the changed params.
    let cluster_info = ctx.contract.cluster_get(ctx.cluster_id)?;
    assert_eq!(cluster_info.cluster.cluster_params, new_cluster_params);
}

#[ink::test]
fn cluster_change_params_only_owner() {
    let ctx = &mut new_cluster();

    let not_manager = AccountId::from([0xee, 0x0a, 0xc9, 0x58, 0xa2, 0x0d, 0xe8, 0xda, 0x73, 0xb2, 0x05, 0xe9, 0xc6, 0x34, 0xa6, 0xb2, 0x23, 0xcc, 0x54, 0x30, 0x24, 0x5d, 0x89, 0xb6, 0x4d, 0x83, 0x9b, 0x6d, 0xca, 0xc4, 0xf8, 0x6d]);
    set_balance(not_manager, 1000 * TOKEN);
    // Change params.
    let new_cluster_params = NodeParams::from("new cluster params");
    set_caller_value(not_manager, CONTRACT_FEE_LIMIT);

    assert_eq!(
        ctx.contract.cluster_set_params(
            ctx.cluster_id, 
            new_cluster_params
        ),
        Err(OnlyClusterManager)
    );
}

#[ink::test]
fn bucket_change_params_works() {
    let ctx = &mut new_cluster();
    let test_bucket = &new_bucket(ctx);

    // Change params.
    set_caller_value(test_bucket.owner_id, CONTRACT_FEE_LIMIT);
    ctx.contract
        .bucket_change_params(test_bucket.bucket_id, "new params".to_string());

    // Check the changed params.
    let status = ctx.contract.bucket_get(test_bucket.bucket_id)?;
    assert_eq!(status.params, "new params");
}

#[ink::test]
#[should_panic]
fn bucket_change_params_only_owner() {
    let ctx = &mut new_cluster();
    let test_bucket = &new_bucket(ctx);

    // Change params.
    set_caller_value(get_accounts().bob, CONTRACT_FEE_LIMIT);
    ctx.contract
        .bucket_change_params(test_bucket.bucket_id, "new params".to_string());
    // Panic.
}

#[ink::test]
fn bucket_list_works() {
    let mut ddc_bucket = setup();

    let owner_id1 = AccountId::from([0xd8, 0x69, 0x19, 0x54, 0xea, 0xdc, 0x9a, 0xc0, 0x3d, 0x37, 0x56, 0x9f, 0x2a, 0xe8, 0xdf, 0x59, 0x34, 0x3f, 0x32, 0x65, 0xba, 0xd4, 0x16, 0xac, 0x07, 0xdf, 0x06, 0xeb, 0x4d, 0xbc, 0x6a, 0x66]);
    set_balance(owner_id1, 1000 * TOKEN);
    let owner_id2 = AccountId::from([0x2a, 0x5f, 0xbc, 0xcf, 0x71, 0x0b, 0x65, 0x04, 0x88, 0x91, 0x12, 0x7e, 0x5e, 0xe3, 0x78, 0xdb, 0x48, 0x63, 0x09, 0x44, 0xcc, 0xc5, 0x75, 0xbd, 0xa5, 0xaa, 0xa5, 0x0e, 0x77, 0xab, 0x7b, 0x4e]);
    set_balance(owner_id2, 1000 * TOKEN);
    let owner_id3 = AccountId::from([0x64, 0xef, 0xd7, 0xb4, 0x41, 0xb2, 0x58, 0xb5, 0x56, 0x6b, 0xfc, 0x4b, 0x19, 0xb8, 0xe5, 0x09, 0x5d, 0x17, 0xb3, 0xc3, 0x44, 0x38, 0x58, 0xa9, 0x7d, 0x20, 0x49, 0x39, 0xbd, 0xbd, 0xb6, 0x48]);
    set_balance(owner_id3, 1000 * TOKEN);

    let cluster_id = 0;

    set_caller_value(owner_id1, CONTRACT_FEE_LIMIT);
    let bucket_id1 = ddc_bucket.bucket_create("".to_string(), cluster_id, None);
    let bucket_status1 = ddc_bucket.bucket_get(bucket_id1).unwrap();

    set_caller_value(owner_id2, CONTRACT_FEE_LIMIT);
    let bucket_id2 = ddc_bucket.bucket_create("".to_string(), cluster_id, None);
    let bucket_status2 = ddc_bucket.bucket_get(bucket_id2)?;

    assert_ne!(bucket_id1, bucket_id2);
    let count = 2;

    assert_eq!(
        ddc_bucket.bucket_list(0, 100, None),
        (vec![bucket_status1.clone(), bucket_status2.clone()], count)
    );

    assert_eq!(
        ddc_bucket.bucket_list(0, 2, None),
        (vec![bucket_status1.clone(), bucket_status2.clone()], count)
    );

    assert_eq!(
        ddc_bucket.bucket_list(0, 1, None),
        (
            vec![bucket_status1.clone()],
            count
        )
    );
    assert_eq!(
        ddc_bucket.bucket_list(1, 1, None),
        (
            vec![bucket_status2.clone()],
            count
        )
    );

    assert_eq!(ddc_bucket.bucket_list(count, 20, None), (vec![], count));

    // Filter by owner.
    assert_eq!(
        ddc_bucket.bucket_list(0, 100, Some(owner_id1)),
        (
            vec![bucket_status1.clone()],
            count
        )
    );

    assert_eq!(
        ddc_bucket.bucket_list(0, 100, Some(owner_id2)),
        (
            vec![bucket_status2.clone()],
            count
        )
    );

    assert_eq!(
        ddc_bucket.bucket_list(0, 100, Some(owner_id3)),
        (vec![], count)
    );
}


#[ink::test]
fn node_list_works() {
    let ctx = new_cluster();

    let node_info = ctx.contract.node_get(ctx.node_key1)?;
    assert_eq!(ctx.provider_id1, node_info.node.provider_id.clone());

    let node1 = NodeInfo {
        node_key: ctx.node_key1,
        node: Node {
            provider_id: ctx.provider_id1,
            rent_per_month: ctx.rent_per_v_node,
            free_resource: ctx.capacity - ctx.reserved_resource * 3,
            cluster_id: Some(ctx.cluster_id),
            status_in_cluster: Some(NodeStatusInCluster::ADDING),
            node_params: ctx.node_params1,
        },
        v_nodes: ctx.v_nodes1.clone()
    };

    let node2 = NodeInfo {
        node_key: ctx.node_key2,
        node: Node {
            provider_id:ctx.provider_id2,
            rent_per_month: ctx.rent_per_v_node,
            free_resource: ctx.capacity - ctx.reserved_resource * 3,
            cluster_id: Some(ctx.cluster_id),
            status_in_cluster: Some(NodeStatusInCluster::ADDING),
            node_params: ctx.node_params2,
        },
        v_nodes: ctx.v_nodes2.clone()
    };

    let count = 3;

    assert_eq!(
        ctx.contract.node_list(1, 100, None),
        (vec![node1.clone(), node2.clone()], count)
    );

    assert_eq!(
        ctx.contract.node_list(1, 2, None),
        (vec![node1.clone(), node2.clone()], count)
    );

    assert_eq!(
        ctx.contract.node_list(1, 1, None),
        (vec![node1.clone() /*, node2.clone()*/], count)
    );

    assert_eq!(
        ctx.contract.node_list(2, 1, None),
        (vec![/*node1.clone(),*/ node2.clone()], count)
    );

    assert_eq!(ctx.contract.node_list(21, 20, None), (vec![], count));

    // Filter by owner.
    assert_eq!(
        ctx.contract.node_list(1, 100, Some(ctx.provider_id1)),
        (vec![node1.clone() /*, node2.clone()*/], count)
    );

    assert_eq!(
        ctx.contract.node_list(1, 100, Some(ctx.provider_id2)),
        (vec![/*node1.clone(),*/ node2.clone()], count)
    );

    let not_provider= AccountId::from([0xee, 0x0a, 0xc9, 0x58, 0xa2, 0x0d, 0xe8, 0xda, 0x73, 0xb2, 0x05, 0xe9, 0xc6, 0x34, 0xa6, 0xb2, 0x23, 0xcc, 0x54, 0x30, 0x24, 0x5d, 0x89, 0xb6, 0x4d, 0x83, 0x9b, 0x6d, 0xca, 0xc4, 0xf8, 0x6d]);

    assert_eq!(
        ctx.contract.node_list(1, 100, Some(not_provider)),
        (vec![], count)
    );
}
