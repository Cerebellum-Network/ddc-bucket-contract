use ink_lang as ink;

use crate::ddc_bucket::account::entity::Account;
// use crate::ddc_bucket::cdn_node::entity::CdnNodeKey;
use crate::ddc_bucket::cluster::entity::ClusterInfo;
use crate::ddc_bucket::flow::Flow;
use crate::ddc_bucket::node::entity::{NodeStatus, NodeKey};
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
    manager: AccountId,
    cluster_id: ClusterId,
    provider_id0: AccountId,
    provider_id1: AccountId,
    provider_id2: AccountId,
    node_key0: NodeKey,
    node_key1: NodeKey,
    node_key2: NodeKey,
    rent_per_vnode: Balance,
    node_keys: Vec<NodeKey>,
    vnodes: Vec<u64>,
    vnodes_wrapper: Vec<Vec<u64>>,
    node_params0: &'static str,
    node_params1: &'static str,
    node_params2: &'static str,
    capacity: u32,
    reserved: u32,
}

fn new_cluster() -> TestCluster {
    let accounts = get_accounts();
    set_balance(accounts.charlie, 1000 * TOKEN);
    set_balance(accounts.django, 1000 * TOKEN);
    let provider_id0 = accounts.alice;
    let provider_id1 = accounts.bob;
    let provider_id2 = accounts.charlie;
    let manager = accounts.django;

    let mut contract = setup();

    // Provide a Node.
    let rent_per_vnode: Balance = 10 * TOKEN;
    let node_params0 = "{\"url\":\"https://ddc.cere.network/bucket/{BUCKET_ID}\"}";
    let capacity = 100;

    for provider_id in [provider_id0, provider_id1, provider_id2] {
        set_caller_value(provider_id, CONTRACT_FEE_LIMIT);
        contract.node_trust_manager(manager);
        let expected_perm = Permission::ManagerTrustedBy(provider_id);
        assert!(contract.has_permission(manager, expected_perm));
    }

    set_caller_value(provider_id0, CONTRACT_FEE_LIMIT);
    let node_key0 = contract.node_create(
        AccountId::from([0x0a; 32]),
        rent_per_vnode,
        node_params0.to_string(),
        capacity,
        NodeStatus::ADDING,
    );

    // Provide another Node.
    let node_params1 = "{\"url\":\"https://ddc-1.cere.network/bucket/{BUCKET_ID}\"}";
    set_caller_value(provider_id1, CONTRACT_FEE_LIMIT);

    let node_key1 = contract.node_create(
        AccountId::from([0x0b; 32]),
        rent_per_vnode,
        node_params1.to_string(),
        capacity,
        NodeStatus::ADDING,
    );

    // Provide another Node.
    let node_params2 = "{\"url\":\"https://ddc-2.cere.network/bucket/{BUCKET_ID}\"}";
    set_caller_value(provider_id2, CONTRACT_FEE_LIMIT);

    let node_key2 = contract.node_create(
        AccountId::from([0x0c; 32]),
        rent_per_vnode,
        node_params2.to_string(),
        capacity,
        NodeStatus::ADDING,
    );

    // Create a Cluster.
    let node_keys = vec![
        AccountId::from([0x0a; 32]), 
        AccountId::from([0x0b; 32]), 
        AccountId::from([0x0c; 32])
    ];
    let cluster_params = "{}";
    let mut vnodes_wrapper = Vec::<Vec<u64>>::new();

    let vnodes_for_first_node = vec![1, 2, 3];
    let vnodes_for_second_node = vec![4, 5, 6];
    let vnodes_for_third_node = vec![7, 8, 9];

    vnodes_wrapper.push(vnodes_for_first_node);
    vnodes_wrapper.push(vnodes_for_second_node);
    vnodes_wrapper.push(vnodes_for_third_node);

    set_caller_value(manager, CONTRACT_FEE_LIMIT);

    let cluster_id = contract.cluster_create(
        manager,
        vnodes_wrapper.clone(),
        vec![node_key0, node_key1, node_key2],
        cluster_params.to_string(),
    );

    let reserved = 10;
    contract.cluster_reserve_resource(cluster_id, reserved);

    let mut vnodes = Vec::<u64>::new();

    for v_nodes_vec in vnodes_wrapper.clone() {
        for v_node in v_nodes_vec {
            vnodes.push(v_node.clone());
        }
    }

    TestCluster {
        contract,
        manager,
        cluster_id,
        provider_id0,
        provider_id1,
        provider_id2,
        node_key0,
        node_key1,
        node_key2,
        rent_per_vnode,
        vnodes_wrapper,
        vnodes,
        node_keys,
        node_params0,
        node_params1,
        node_params2,
        capacity,
        reserved,
    }
}

fn new_cluster_cdn() -> TestCluster {
    let accounts = get_accounts();
    set_balance(accounts.charlie, 1000 * TOKEN);
    set_balance(accounts.django, 1000 * TOKEN);
    let provider_id0 = accounts.alice;
    let provider_id1 = accounts.bob;
    let provider_id2 = accounts.charlie;
    let manager = accounts.django;

    let mut contract = setup();

    // Provide a Node.
    let rent_per_vnode: Balance = 10 * TOKEN;
    let node_params0 = "{\"url\":\"https://ddc.cere.network/bucket/{BUCKET_ID}\"}";
    let capacity = 100;

    for provider_id in [provider_id0, provider_id1, provider_id2] {
        set_caller_value(provider_id, CONTRACT_FEE_LIMIT);

        contract.node_trust_manager(manager);
        let expected_perm = Permission::ManagerTrustedBy(provider_id);
        assert!(contract.has_permission(manager, expected_perm));
    }

    set_caller_value(provider_id0, CONTRACT_FEE_LIMIT);

    let cdn_node_key0 = AccountId::from([0x0a; 32]);

    let node_key0 = contract.cdn_node_create(cdn_node_key0, node_params0.to_string());

    // Provide another Node.
    let node_params1 = "{\"url\":\"https://ddc-1.cere.network/bucket/{BUCKET_ID}\"}";
    set_caller_value(provider_id1, CONTRACT_FEE_LIMIT);

    let cdn_node_key1 = AccountId::from([0x0b; 32]);

    let node_key1 = contract.cdn_node_create(cdn_node_key1, node_params1.to_string());

    // Provide another Node.
    let node_params2 = "{\"url\":\"https://ddc-2.cere.network/bucket/{BUCKET_ID}\"}";
    set_caller_value(provider_id2, CONTRACT_FEE_LIMIT);
    
    let cdn_node_key2 = AccountId::from([0x0c; 32]);

    let node_key2 = contract.cdn_node_create(cdn_node_key2, node_params2.to_string());

    // Create a Cluster.
    let _cluster_params = "{}";

    // TODO: adjust after cdn cluster topology and node addition

    let vnodes = vec![1, 2, 3, 4, 5, 6];

    let node_keys = vec![node_key0, node_key1, node_key2];
    let mut vnodes_wrapper = Vec::<Vec<u64>>::new();
    vnodes_wrapper.push(vnodes);

    let mut vnodes_wrapper = Vec::<Vec<u64>>::new();

    let vnodes_for_first_node = vec![1, 2, 3];
    let vnodes_for_second_node = vec![4, 5, 6];
    let vnodes_for_third_node = vec![7, 8, 9];

    vnodes_wrapper.push(vnodes_for_first_node);
    vnodes_wrapper.push(vnodes_for_second_node);
    vnodes_wrapper.push(vnodes_for_third_node);

    set_caller_value(manager, CONTRACT_FEE_LIMIT);

    let cluster_id = contract.cdn_cluster_create(
        vec![
            AccountId::from([0x0a; 32]), 
            AccountId::from([0x0b; 32]), 
            AccountId::from([0x0c; 32])
        ]
    );

    let reserved = 10;

    let mut vnodes = Vec::<u64>::new();

    for v_nodes_vec in vnodes_wrapper.clone() {
        for v_node in v_nodes_vec {
            vnodes.push(v_node.clone());
        }
    }

    TestCluster {
        contract,
        manager,
        cluster_id,
        provider_id0,
        provider_id1,
        provider_id2,
        node_key0,
        node_key1,
        node_key2,
        rent_per_vnode,
        vnodes_wrapper,
        node_params0,
        node_params1,
        node_params2,
        capacity,
        reserved,
        node_keys,
        vnodes,
    }
}

struct TestBucket {
    bucket_id: BucketId,
    owner_id: AccountId,
    resource: u32,
}

fn new_bucket(ctx: &mut TestCluster) -> TestBucket {
    let accounts = get_accounts();
    let owner_id = accounts.django;
    set_balance(owner_id, 1000 * TOKEN);
    set_caller_value(owner_id, CONTRACT_FEE_LIMIT);

    let bucket_id = ctx
        .contract
        .bucket_create("{}".to_string(), ctx.cluster_id, None);

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
    let provider_ids = &[ctx.provider_id0, ctx.provider_id1, ctx.provider_id2];
    let node_keys = &[ctx.node_key0, ctx.node_key1, ctx.node_key2];
    let node_params = &[ctx.node_params0, ctx.node_params1, ctx.node_params2];

    assert_eq!(ctx.cluster_id, 1, "cluster_id must start at 1");
    assert_eq!(ctx.node_key0, AccountId::from([0x0a; 32]), "node_id must start at 1");
    assert_ne!(ctx.node_key0, ctx.node_key1, "nodes must have unique IDs");

    // Check the nodes.
    {
        let node0 = ctx.contract.node_get(ctx.node_key0)?;

        assert_eq!(
            node0,
            NodeInfo {
                node_key: ctx.node_key0,
                node: Node {
                    provider_id: ctx.provider_id0,
                    rent_per_month: ctx.rent_per_vnode,
                    free_resource: ctx.capacity - ctx.reserved * 3,
                    status: NodeStatus::ADDING,
                    node_params: ctx.node_params0.to_string(),
                }
            }
        );

        let node1 = ctx.contract.node_get(ctx.node_key1)?;
        assert_eq!(
            node1,
            NodeInfo {
                node_key: ctx.node_key1,
                node: Node {
                    provider_id: ctx.provider_id1,
                    rent_per_month: ctx.rent_per_vnode,
                    free_resource: ctx.capacity - ctx.reserved * 3,
                    status: NodeStatus::ADDING,
                    node_params: ctx.node_params1.to_string(),
                }
            }
        );

        let node2 = ctx.contract.node_get(ctx.node_key2)?;
        assert_eq!(
            node2,
            NodeInfo {
                node_key: ctx.node_key2,
                node: Node {
                    provider_id: ctx.provider_id2,
                    rent_per_month: ctx.rent_per_vnode,
                    free_resource: ctx.capacity - ctx.reserved * 3,
                    status: NodeStatus::ADDING,
                    node_params: ctx.node_params2.to_string(),
                }
            }
        );
    }

    // Check the initial state of the cluster.
    {
        let cluster = ctx.contract.cluster_get(ctx.cluster_id)?;
        assert_eq!(
            cluster,
            ClusterInfo {
                cluster_id: ctx.cluster_id,
                cluster: Cluster {
                    manager_id: ctx.manager,
                    node_keys: ctx.node_keys,
                    v_nodes: ctx.vnodes_wrapper,
                    resource_per_vnode: ctx.reserved,
                    resource_used: 0,
                    revenues: Cash(0),
                    total_rent: ctx.rent_per_vnode * ctx.vnodes.len() as Balance,
                },
                params: "{}".to_string(),
            }
        );
    }

    // Check the events.
    let mut evs = get_events();
    evs.reverse(); // Work with pop().

    // Providers trust Manager.
    for provider_id in provider_ids {
        assert!(
            matches!(evs.pop().unwrap(), Event::PermissionGranted(ev) if ev ==
            PermissionGranted { account_id: ctx.manager, permission: Permission::ManagerTrustedBy(*provider_id) })
        );
    }

    // Nodes created.
    for i in 0..3 {
        assert!(matches!(evs.pop().unwrap(), Event::NodeCreated(ev) if ev ==
            NodeCreated {
                node_key: node_keys[i],
                provider_id: provider_ids[i],
                rent_per_month: ctx.rent_per_vnode,
                node_params: node_params[i].to_string() 
            })
        );
    }

    // Cluster setup.
    assert!(
        matches!(evs.pop().unwrap(), Event::ClusterCreated(ev) if ev ==
        ClusterCreated { cluster_id: ctx.cluster_id, manager: ctx.manager, cluster_params: "{}".to_string() })
    );

    assert!(
        matches!(evs.pop().unwrap(), Event::ClusterReserveResource(ev) if ev ==
        ClusterReserveResource { cluster_id: ctx.cluster_id, resource: ctx.reserved })
    );

    assert_eq!(evs.len(), 0, "all events must be checked");
}

#[ink::test]
fn cluster_replace_node_only_manager() {
    let mut ctx = new_cluster();
    let not_manager = get_accounts().alice;
    set_caller_value(not_manager, 0);

    // Reassign a vnode from node1 to node2.
    assert_eq!(
        ctx.contract
            .message_cluster_replace_node(
                ctx.cluster_id, 
                vec![1, 2, 3], 
                ctx.node_key2
            ),
        Err(UnauthorizedClusterManager)
    );
}

#[ink::test]
fn cluster_replace_node_only_trusted_manager() {
    let mut ctx = new_cluster();

    // The provider stops trusting the manager.
    set_caller(ctx.provider_id2);
    ctx.contract.node_distrust_manager(ctx.manager);

    set_caller_value(ctx.manager, 0);

    // The manager cannot use nodes of the provider.
    assert_eq!(
        ctx.contract
            .message_cluster_replace_node(
                ctx.cluster_id, 
                vec![1, 2, 3],
                ctx.node_key2
            ),
        Err(ClusterManagerIsNotTrusted)
    );
}

#[ink::test]
fn cluster_replace_node_works() {
    let mut ctx = new_cluster();
    set_caller_value(ctx.manager, 0);

    // Reassign a vnode from node1 to node2.
    ctx.contract
        .cluster_replace_node(ctx.cluster_id, vec![1, 3], ctx.node_key2);

    // Check the last event.
    let ev = get_events().pop().unwrap();
    assert!(matches!(ev, Event::ClusterNodeReplaced(ev) if ev ==
        ClusterNodeReplaced { cluster_id: ctx.cluster_id, node_key: ctx.node_key2  }));

    let vnodes_for_replaced = vec![2];
    let vnodes_for_second_node = vec![4, 5, 6];
    let vnodes_for_third_node = vec![7, 8, 9];
    let vnodes_for_third_dup = vec![1, 3];

    let vnodes = vec![
        vnodes_for_replaced,
        vnodes_for_second_node,
        vnodes_for_third_node,
        vnodes_for_third_dup,
    ];

    // Check the changed state of the cluster.
    let cluster = ctx.contract.cluster_get(ctx.cluster_id)?.cluster;
    println!("cluster.v_nodes: {:?}", cluster.v_nodes.clone());
    println!("cluster.node_keys: {:?}", cluster.node_keys.clone());
    assert_eq!(&cluster.v_nodes, &vnodes, "a vnode must be replaced");

    // Check the changed state of the nodes.
    let expected_resources = [
        (ctx.node_key0, 100 - 10),
        (ctx.node_key1, 100 - 10 - 10 - 10),
        (ctx.node_key2, 100 - 10 - 10 - 10 - 10 - 10),
    ];

    for (node_key, available) in expected_resources {
        let node_status = ctx.contract.node_get(node_key).unwrap();
        assert_eq!(
            node_status.node.free_resource, available,
            "resources must have shifted between nodes"
        );
    }
}

#[ink::test]
fn cluster_reserve_works() {
    let mut ctx = new_cluster();
    set_caller_value(ctx.manager, 0);

    // Reserve more resources.
    ctx.contract.cluster_reserve_resource(ctx.cluster_id, 5);

    // Check the last event.
    let ev = get_events().pop().unwrap();
    assert!(matches!(ev, Event::ClusterReserveResource(ev) if ev ==
        ClusterReserveResource { cluster_id: ctx.cluster_id, resource: 5 }));

    // Check the changed state of the cluster.
    let cluster = ctx.contract.cluster_get(ctx.cluster_id)?.cluster;
    assert_eq!(cluster.resource_per_vnode, 10 + 5);

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

    let not_manager = ctx.provider_id0;
    set_caller_value(not_manager, 0);

    assert_eq!(
        ctx.contract
            .message_cluster_replace_node(
                ctx.cluster_id, 
                vec![1, 2, 3],
                AccountId::from([0x0a; 32])
            ),
        Err(UnauthorizedClusterManager),
        "only the manager can modify the cluster"
    );

    set_caller_value(ctx.manager, 0);

    let bad_node_id = AccountId::from([0x0d; 32]);
    assert_eq!(
        ctx.contract
            .message_cluster_replace_node(
                ctx.cluster_id, 
                vec![1, 2, 3],
                bad_node_id
            ),
        Err(NodeDoesNotExist),
        "cluster replacement node must exist"
    );

    assert_eq!(
        ctx.contract
            .message_cluster_create(vec![vec![1, 2, 3]], vec![bad_node_id], vec![1, 2, 3], "".to_string()),
        Err(NodeDoesNotExist),
        "cluster initial nodes must exist"
    );
}

#[ink::test]
fn cdn_cluster_gas_converter_works() {
    println!("Creating new cdn cluster");

    let mut ctx = new_cluster_cdn();

    println!("Got cdn cluster back");
    // The provider stops trusting the manager.
    println!("Cdn cluster id is {}", ctx.cluster_id);
    set_caller(ctx.manager);
    ctx.contract.cdn_set_rate(ctx.cluster_id, 3_750_000_000);
    set_caller(ctx.provider_id0);
    let rate = ctx.contract.cdn_get_rate(ctx.cluster_id);

    let usd_per_cere = TOKEN / 100;
    set_caller(ctx.provider_id0);
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
    println!("Creating new cdn cluster");

    let mut ctx = new_cluster_cdn();

    println!("Got cdn cluster back");
    // The provider stops trusting the manager.
    println!("Cdn cluster id is {}", ctx.cluster_id);
    set_caller(ctx.provider_id0);
    let rate = ctx.contract.cdn_get_rate(ctx.cluster_id);

    let usd_per_cere = TOKEN / 100;
    set_caller(ctx.provider_id0);
    ctx.contract.account_set_usd_per_cere(usd_per_cere);

    let usd_amount = ctx.contract.account_get_usd_per_cere();
    println!("Current usd amount is {}", usd_amount);

    println!("The current rate is {}", rate);

    let usd_per_kb = rate / KB_PER_GB;
    println!("The current rate per kb {}", usd_per_kb);

    let cere_per_kb = ctx.contract.accounts.1.to_cere(usd_per_kb);
    println!("The current cere rate per kb {}", cere_per_kb);

    set_caller_value(ctx.provider_id0, 10 * TOKEN);
    ctx.contract.account_deposit();

    set_caller(ctx.provider_id0);
    ctx.contract.account_bond(5 * TOKEN);

    set_caller(ctx.provider_id0);
    ctx.contract.set_fee_bp(1_000);

    let mut account = ctx.contract.accounts.get(&ctx.provider_id0).unwrap();
    println!("{:?}", account);

    ctx.contract.cdn_cluster_put_revenue(
        ctx.cluster_id,
        vec![(ctx.provider_id0, 1000), (ctx.provider_id0, 541643)],
        vec![(ctx.node_key0, 1000), (ctx.node_key1, 541643)],
        vec![],
        5,
    );
    account = ctx.contract.accounts.get(&ctx.provider_id0).unwrap();
    println!("{:?}", account);

    let cdn_cluster_list_one = ctx.contract.cdn_cluster_list(0, 10, None);
    print!("Cluster list one {:?}", cdn_cluster_list_one);
    let node0 = ctx.contract.cdn_nodes.get(ctx.node_key0).unwrap();
    let node1 = ctx.contract.cdn_nodes.get(ctx.node_key1).unwrap();

    println!("Node 1 {:?}", node0);
    println!("Node 2 {:?}", node1);
    // let cdn_cluster0 = ctx.contract.cdn_clusters.get(ctx.cluster_id);
    // print!("{:?}", cdn_cluster0);
    // let cdn_cluster_list = ctx.contract.cdn_cluster_list(0, 10, None);
    // print!("{:?}", cdn_cluster0);
    // print!("{:?}", cdn_cluster_list);

    let validated_commit_node0 = ctx.contract.get_validated_commit(ctx.node_key0);
    print!("Validated commit {:?}", validated_commit_node0);

    let fee = ctx.contract.get_fee_bp();
    print!("Protocol fee in basis points {:?}", fee);

    let protocol_revenues = ctx.contract.get_protocol_revenues();
    print!("Protocol revenues are {:?}", protocol_revenues);

    set_caller(ctx.provider_id0);
    ctx.contract.cdn_cluster_distribute_revenues(0);

    let node0 = ctx.contract.cdn_nodes.get(ctx.node_key0).unwrap();
    let node1 = ctx.contract.cdn_nodes.get(ctx.node_key1).unwrap();
    println!("{:?}", node0);
    println!("{:?}", node1);

    let cdn_cluster_list = ctx.contract.cdn_cluster_list(0, 10, None);
    print!("{:?}", cdn_cluster_list);

    account = ctx.contract.accounts.get(&ctx.provider_id0).unwrap();
    // let node0 = ctx.contract.cdn_nodes.get(ctx.node_key0).unwrap();
    // let node1 = ctx.contract.cdn_nodes.get(ctx.node_key1).unwrap();
    println!("{:?}", account);
    // let account1 = ctx.contract.accounts.get(&ctx.provider_id1).unwrap();
    // println!("{:?}", account1);
    // let account_balance = ink_env::balance(&ctx.provider_id0);
    // println!("{:?}", account_balance);
}

fn bucket_settle_payment(ctx: &mut TestCluster, test_bucket: &TestBucket) {
    // Go to the future when some revenues are due.
    advance_block::<DefaultEnvironment>();
    // Pay the due thus far.
    set_caller_value(ctx.manager, CONTRACT_FEE_LIMIT);
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

    // Set up an exchange rate manager.
    set_caller_value(admin_id(), CONTRACT_FEE_LIMIT);
    ctx.contract
        .admin_grant_permission(admin_id(), Permission::SetExchangeRate);

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
    let expected_rent = ctx.rent_per_vnode * ctx.vnodes.len() as Balance;

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
        BucketSettlePayment {  bucket_id: test_bucket.bucket_id, cluster_id: ctx.cluster_id }));

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
fn cluster_add_node() {
    let ctx = &mut new_cluster();

    let new_provider_id = get_accounts().frank;
    set_balance(new_provider_id, 1000 * TOKEN);

    let rent_per_month = 100;
    let node_params = "new_node";
    let capacity = 1000;
    let status = NodeStatus::ACTIVE;

    set_caller_value(new_provider_id, CONTRACT_FEE_LIMIT);
    let new_node_id = ctx.contract.node_create(
        AccountId::from([0x0d; 32]),
        rent_per_month,
        node_params.to_string(),
        capacity,
        status,
    );

    set_caller_value(new_provider_id, CONTRACT_FEE_LIMIT);
    ctx.contract.node_trust_manager(ctx.manager);
    assert!(
        matches!(get_events().pop().unwrap(), Event::PermissionGranted(ev) if ev ==
        PermissionGranted { account_id: ctx.manager, permission: Permission::ManagerTrustedBy(new_provider_id) })
    );

    let mut node_keys = Vec::<NodeKey>::new();
    node_keys.push(ctx.node_key0);
    node_keys.push(ctx.node_key1);
    node_keys.push(ctx.node_key2);
    node_keys.push(new_node_id);

    let mut v_nodes = ctx.vnodes_wrapper.clone();
    v_nodes.push(vec![10, 11, 12]);

    set_caller_value(ctx.manager, CONTRACT_FEE_LIMIT);
    ctx.contract
        .cluster_add_node(ctx.cluster_id, node_keys.clone(), v_nodes.clone());

    let cluster_status = ctx.contract.cluster_get(ctx.cluster_id).unwrap();
    assert!(matches!(cluster_status.clone().cluster.node_keys.len(), 4));
    assert!(matches!(cluster_status.clone().cluster.v_nodes.len(), 4));
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
    let before_mgmt = balance_of(ctx.manager);

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
    let provider_fee = (to_distribute - burned_fee - manager_fee) / 3;

    // Distribute the revenues of the cluster to providers.
    ctx.contract.cluster_distribute_revenues(ctx.cluster_id);

    // Check the last events.
    let mut evs = get_events();
    evs.reverse(); // Work with pop().
    evs.truncate(evs.len() - skip_events);
    let expected_recipients = vec![ctx.provider_id0, ctx.provider_id1, ctx.provider_id2];

    for provider_id in expected_recipients {
        assert!(
            matches!(evs.pop().unwrap(), Event::ClusterDistributeRevenues(ev) if ev ==
            ClusterDistributeRevenues { cluster_id: ctx.cluster_id, provider_id })
        );
    }

    assert_eq!(evs.len(), 0, "all events must be checked");

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
    let earned_mgmt = balance_of(ctx.manager) - before_mgmt;

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
    assert!(manager_fee > 0, "the manager must earn something");
    assert_eq!(
        earned_mgmt, manager_fee,
        "the manager must earn the correct amount"
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
fn bucket_reserve_0_works() {
    let contract = setup();

    assert_eq!(
        contract.bucket_list(0, 10, None),
        (
            vec![BucketStatus {
                bucket_id: 0,
                bucket: BucketInStatus {
                    owner_id: AccountId::default(),
                    cluster_id: 0,
                    resource_reserved: 0,
                    public_availability: false,
                    resource_consumption_cap: 0,
                },
                params: "".to_string(),
                writer_ids: vec![AccountId::default()],
                reader_ids: vec![],
                rent_covered_until_ms: 18446744073709551615,
            }],
            1
        )
    );

    assert_eq!(
        contract.cluster_list(0, 10, None),
        (
            vec![ClusterInfo {
                cluster_id: 0,
                cluster: Cluster {
                    manager_id: AccountId::default(),
                    v_nodes: vec![],
                    resource_per_vnode: 0,
                    resource_used: 0,
                    revenues: Cash(0),
                    total_rent: 0,
                    node_keys: vec![]
                },
                params: "".to_string(),
            }],
            1
        )
    );

    assert_eq!(
        contract.node_list(0, 10, None),
        (
            vec![NodeInfo {
                node_key: AccountId::default(),
                node: Node {
                    provider_id: AccountId::default(),
                    rent_per_month: 0,
                    free_resource: 0,
                    status: NodeStatus::ACTIVE,
                    node_params: "".to_string(),
                }
            }],
            1
        )
    );
}

#[ink::test]
fn bucket_create_works() {
    let ctx = &mut new_cluster();
    let test_bucket = &new_bucket(ctx);

    assert_eq!(test_bucket.bucket_id, 1, "bucket_id must start at 1");

    // Check the structure of the bucket including the payment flow.
    let total_rent = ctx.rent_per_vnode * ctx.vnodes.len() as Balance;
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

    let mut evs = get_events();
    evs.reverse(); // Work with pop().
    evs.truncate(8 - 3 - 2); // Skip 3 NodeCreated and 2 cluster setup events.

    // Create bucket.
    assert!(
        matches!(evs.pop().unwrap(), Event::BucketCreated(ev) if ev ==
        BucketCreated {  bucket_id: test_bucket.bucket_id, owner_id: test_bucket.owner_id })
    );

    assert!(
        matches!(evs.pop().unwrap(), Event::BucketAllocated(ev) if ev ==
        BucketAllocated { bucket_id: test_bucket.bucket_id, cluster_id: ctx.cluster_id, resource: test_bucket.resource })
    );

    // Deposit more.
    let net_deposit = 10 * TOKEN;
    assert!(matches!(evs.pop().unwrap(), Event::Deposit(ev) if ev ==
        Deposit { account_id: test_bucket.owner_id, value: net_deposit }));

    assert_eq!(evs.len(), 0, "all events must be checked");
}

#[ink::test]
fn account_deposit_works() {
    let account_id = get_accounts().alice;
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
    let mut evs = get_events();
    evs.reverse(); // Work with pop().

    // First deposit event.
    assert!(matches!(evs.pop().unwrap(), Event::Deposit(ev) if ev ==
        Deposit { account_id, value: deposit_after_fee }));

    // Second deposit event. No deposit_contract_fee because the account already exists.
    assert!(matches!(evs.pop().unwrap(), Event::Deposit(ev) if ev ==
        Deposit { account_id, value: deposit }));

    assert_eq!(evs.len(), 0, "all events must be checked");
}

#[ink::test]
fn node_change_params_works() {
    let ctx = &mut new_cluster();

    // Change params.
    set_caller_value(ctx.provider_id0, CONTRACT_FEE_LIMIT);
    ctx.contract
        .node_change_params(ctx.node_key0, "new params".to_string());

    // Check the changed params.
    let status = ctx.contract.node_get(ctx.node_key0)?;
    assert_eq!(status.node.node_params, "new params");
}

#[ink::test]
#[should_panic]
fn node_change_params_only_owner() {
    let ctx = &mut new_cluster();

    // Change params.
    set_caller_value(get_accounts().bob, CONTRACT_FEE_LIMIT);
    ctx.contract
        .node_change_params(ctx.node_key0, "new params".to_string());
    // Panic.
}

#[ink::test]
fn cluster_change_params_works() {
    let ctx = &mut new_cluster();

    // Change params.
    set_caller_value(ctx.manager, CONTRACT_FEE_LIMIT);
    ctx.contract
        .cluster_change_params(ctx.cluster_id, "new params".to_string());

    // Check the changed params.
    let status = ctx.contract.cluster_get(ctx.cluster_id)?;
    assert_eq!(status.params, "new params");
}

#[ink::test]
#[should_panic]
fn cluster_change_params_only_owner() {
    let ctx = &mut new_cluster();

    // Change params.
    set_caller_value(get_accounts().bob, CONTRACT_FEE_LIMIT);
    ctx.contract
        .cluster_change_params(ctx.cluster_id, "new params".to_string());
    // Panic.
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
    let accounts = get_accounts();
    let owner_id1 = accounts.alice;
    let owner_id2 = accounts.bob;
    let owner_id3 = accounts.charlie;
    let cluster_id = 0;

    set_caller_value(owner_id1, CONTRACT_FEE_LIMIT);
    let bucket_id1 = ddc_bucket.bucket_create("".to_string(), cluster_id, None);
    let bucket_status1 = ddc_bucket.bucket_get(bucket_id1).unwrap();

    set_caller_value(owner_id2, CONTRACT_FEE_LIMIT);
    let bucket_id2 = ddc_bucket.bucket_create("".to_string(), cluster_id, None);
    let bucket_status2 = ddc_bucket.bucket_get(bucket_id2)?;

    assert_ne!(bucket_id1, bucket_id2);
    let count = 3;

    assert_eq!(
        ddc_bucket.bucket_list(1, 100, None),
        (vec![bucket_status1.clone(), bucket_status2.clone()], count)
    );

    assert_eq!(
        ddc_bucket.bucket_list(1, 2, None),
        (vec![bucket_status1.clone(), bucket_status2.clone()], count)
    );

    assert_eq!(
        ddc_bucket.bucket_list(1, 1, None),
        (
            vec![bucket_status1.clone() /*, bucket_status2.clone()*/],
            count
        )
    );
    assert_eq!(
        ddc_bucket.bucket_list(2, 1, None),
        (
            vec![/*bucket_status1.clone(),*/ bucket_status2.clone()],
            count
        )
    );

    assert_eq!(ddc_bucket.bucket_list(count, 20, None), (vec![], count));

    // Filter by owner.
    assert_eq!(
        ddc_bucket.bucket_list(1, 100, Some(owner_id1)),
        (
            vec![bucket_status1.clone() /*, bucket_status2.clone()*/],
            count
        )
    );

    assert_eq!(
        ddc_bucket.bucket_list(1, 100, Some(owner_id2)),
        (
            vec![/*bucket_status1.clone(),*/ bucket_status2.clone()],
            count
        )
    );

    assert_eq!(
        ddc_bucket.bucket_list(1, 100, Some(owner_id3)),
        (vec![], count)
    );
}

#[ink::test]
fn bucket_set_availability_works() {
    let ctx = &mut new_cluster();
    let test_bucket = &new_bucket(ctx);
    let status = ctx.contract.bucket_get(test_bucket.bucket_id)?;
    assert_eq!(status.bucket.public_availability, false);

    // Set public availability
    ctx.contract
        .bucket_set_availability(test_bucket.bucket_id, true);

    // Check the last event.
    let ev = get_events().pop().unwrap();
    assert!(matches!(ev, Event::BucketAvailabilityUpdated(ev) if ev ==
                     BucketAvailabilityUpdated { bucket_id: test_bucket.bucket_id, public_availability: true }));

    // Check the changed params.
    let status = ctx.contract.bucket_get(test_bucket.bucket_id)?;
    assert_eq!(status.bucket.public_availability, true);
}

#[ink::test]
fn node_list_works() {
    let mut ddc_bucket = setup();
    let accounts = get_accounts();
    let owner_id1 = accounts.alice;
    let owner_id2 = accounts.bob;
    let owner_id3 = accounts.charlie;
    let rent_per_month: Balance = 10 * TOKEN;

    // Create two Nodes.
    let node_params1 = "{\"url\":\"https://ddc-1.cere.network/bucket/{BUCKET_ID}\"}";
    let capacity = 100;
    set_caller_value(owner_id1, CONTRACT_FEE_LIMIT);
    let node_key1 = ddc_bucket.node_create(
        AccountId::from([0x0b; 32]),
        rent_per_month,
        node_params1.to_string(),
        capacity,
        NodeStatus::ADDING,
    );

    let node_params2 = "{\"url\":\"https://ddc-2.cere.network/bucket/{BUCKET_ID}\"}";
    set_caller_value(owner_id2, CONTRACT_FEE_LIMIT);
    let node_key2 = ddc_bucket.node_create(
        AccountId::from([0x0c; 32]),
        rent_per_month,
        node_params2.to_string(),
        capacity,
        NodeStatus::ADDING,
    );

    let node_status = ddc_bucket.node_get(AccountId::from([0x0b; 32])).unwrap();
    assert_eq!(owner_id1, node_status.node.provider_id.clone());

    assert_ne!(node_key1, node_key2);
    let count = 3;

    let node1 = NodeInfo {
        node_key: node_key1,
        node: Node {
            provider_id: owner_id1,
            rent_per_month,
            free_resource: capacity,
            status: NodeStatus::ADDING,
            node_params: node_params1.to_string(),
        },
    };

    let node2 = NodeInfo {
        node_key: node_key2,
        node: Node {
            provider_id: owner_id2,
            rent_per_month,
            free_resource: capacity,
            status: NodeStatus::ADDING,
            node_params: node_params2.to_string(),
        },
    };

    assert_eq!(
        ddc_bucket.node_list(1, 100, None),
        (vec![node1.clone(), node2.clone()], count)
    );

    assert_eq!(
        ddc_bucket.node_list(1, 2, None),
        (vec![node1.clone(), node2.clone()], count)
    );

    assert_eq!(
        ddc_bucket.node_list(1, 1, None),
        (vec![node1.clone() /*, node2.clone()*/], count)
    );

    assert_eq!(
        ddc_bucket.node_list(2, 1, None),
        (vec![/*node1.clone(),*/ node2.clone()], count)
    );

    assert_eq!(ddc_bucket.node_list(21, 20, None), (vec![], count));

    // Filter by owner.
    assert_eq!(
        ddc_bucket.node_list(1, 100, Some(owner_id1)),
        (vec![node1.clone() /*, node2.clone()*/], count)
    );

    assert_eq!(
        ddc_bucket.node_list(1, 100, Some(owner_id2)),
        (vec![/*node1.clone(),*/ node2.clone()], count)
    );

    assert_eq!(
        ddc_bucket.node_list(1, 100, Some(owner_id3)),
        (vec![], count)
    );
}
