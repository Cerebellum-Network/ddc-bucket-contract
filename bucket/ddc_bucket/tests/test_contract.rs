use ink_lang as ink;

use crate::ddc_bucket::*;
use crate::ddc_bucket::account::entity::Account;
use crate::ddc_bucket::contract_fee::{calculate_contract_fee, FEE_PER_BYTE, SIZE_PER_RECORD};
use crate::ddc_bucket::flow::Flow;
use crate::ddc_bucket::schedule::{MS_PER_MONTH, Schedule};

use super::env_utils::*;

struct TestCluster {
    contract: DdcBucket,
    manager: AccountId,
    cluster_id: ClusterId,
    provider_id0: AccountId,
    provider_id1: AccountId,
    provider_id2: AccountId,
    node_id0: NodeId,
    node_id1: NodeId,
    node_id2: NodeId,
    rent_per_vnode: Balance,
    partition_count: u32,
}

fn new_cluster() -> TestCluster {
    let accounts = get_accounts();
    set_balance(accounts.charlie, 1000 * TOKEN);
    let provider_id0 = accounts.alice;
    let provider_id1 = accounts.bob;
    let provider_id2 = accounts.charlie;
    let manager = accounts.charlie;

    let mut contract = DdcBucket::new();

    // Provide a Node.
    let rent_per_vnode: Balance = 10 * TOKEN;
    let node_params0 = "{\"url\":\"https://ddc.cere.network/bucket/{BUCKET_ID}\"}";
    let capacity = 100;
    push_caller_value(provider_id0, CONTRACT_FEE_LIMIT);
    let node_id0 = contract.node_create(rent_per_vnode, node_params0.to_string(), capacity);
    pop_caller();

    // Provide another Node.
    let node_params1 = "{\"url\":\"https://ddc-2.cere.network/bucket/{BUCKET_ID}\"}";
    push_caller_value(provider_id1, CONTRACT_FEE_LIMIT);
    let node_id1 = contract.node_create(rent_per_vnode, node_params1.to_string(), capacity);
    pop_caller();

    // Provide another Node.
    let node_params2 = "{\"url\":\"https://ddc-2.cere.network/bucket/{BUCKET_ID}\"}";
    push_caller_value(provider_id2, CONTRACT_FEE_LIMIT);
    let node_id2 = contract.node_create(rent_per_vnode, node_params2.to_string(), capacity);
    pop_caller();

    // Create a Cluster.
    let cluster_params = "{}";
    let partition_count = 6;
    push_caller_value(manager, CONTRACT_FEE_LIMIT);
    let cluster_id = contract.cluster_create(manager, partition_count, vec![node_id0, node_id1, node_id2], cluster_params.to_string());
    pop_caller();

    push_caller_value(manager, 0);
    contract.cluster_reserve_resource(cluster_id, 10);
    pop_caller();

    TestCluster { contract, manager, cluster_id, provider_id0, provider_id1, provider_id2, node_id0, node_id1, node_id2, rent_per_vnode, partition_count }
}


struct TestBucket {
    bucket_id: BucketId,
    owner_id: AccountId,
}

fn new_bucket(ctx: &mut TestCluster) -> TestBucket {
    let accounts = get_accounts();
    let owner_id = accounts.django;
    set_balance(owner_id, 1000 * TOKEN);

    push_caller_value(owner_id, CONTRACT_FEE_LIMIT);
    let bucket_id = ctx.contract.bucket_create("".to_string(), ctx.cluster_id);
    pop_caller();

    // Reserve some resources for the bucket from the cluster.
    push_caller_value(owner_id, CONTRACT_FEE_LIMIT);
    let resource = 1;
    ctx.contract.bucket_alloc_into_cluster(bucket_id, resource);
    pop_caller();

    // Deposit some value to pay for buckets.
    push_caller_value(owner_id, 10 * TOKEN);
    ctx.contract.account_deposit();
    pop_caller();

    TestBucket { bucket_id, owner_id }
}


#[ink::test]
fn cluster_create_works() {
    let ctx = new_cluster();
    push_caller_value(ctx.manager, 0);

    assert_ne!(ctx.node_id0, ctx.node_id1, "nodes must have unique IDs");

    // Check the initial state of the cluster.
    let cluster = ctx.contract.cluster_get(ctx.cluster_id)?;
    let expected_vnodes = &[
        ctx.node_id0, ctx.node_id1, ctx.node_id2,
        ctx.node_id0, ctx.node_id1, ctx.node_id2];
    assert_eq!(&cluster.vnodes, expected_vnodes, "cluster setup with nodes");
    assert_eq!(cluster.resource_per_vnode, 10);

    // Check the initial state of the nodes. 2 vnodes of size 10 are reserved from each.
    let expected_resources = [
        (ctx.node_id0, 100 - 10 - 10),
        (ctx.node_id1, 100 - 10 - 10),
        (ctx.node_id2, 100 - 10 - 10),
    ];
    for (node_id, available) in expected_resources {
        assert_eq!(
            ctx.contract.node_get(node_id)?.free_resource,
            available, "resources must be reserved from the nodes");
    }
}


#[ink::test]
fn cluster_replace_node_works() {
    let mut ctx = new_cluster();
    push_caller_value(ctx.manager, 0);

    // Reassign a vnode from node1 to node2.
    ctx.contract.cluster_replace_node(ctx.cluster_id, 1, ctx.node_id2);

    // Check the changed state of the cluster.
    let cluster = ctx.contract.cluster_get(ctx.cluster_id)?;
    assert_eq!(&cluster.vnodes,
               &[ctx.node_id0, /* changed */ ctx.node_id2, ctx.node_id2, ctx.node_id0, ctx.node_id1, ctx.node_id2],
               "a vnode must be replaced");

    // Check the changed state of the nodes.
    let expected_resources = [
        (ctx.node_id0, 100 - 10 - 10),
        (ctx.node_id1, 100 - 10 - 10 + 10),
        (ctx.node_id2, 100 - 10 - 10 - 10),
    ];
    for (node_id, available) in expected_resources {
        assert_eq!(
            ctx.contract.node_get(node_id)?.free_resource,
            available, "resources must have shifted between nodes");
    }
}


#[ink::test]
fn cluster_reserve_works() {
    let mut ctx = new_cluster();
    push_caller_value(ctx.manager, 0);

    // Reserve more resources.
    ctx.contract.cluster_reserve_resource(ctx.cluster_id, 5);

    // Check the changed state of the cluster.
    let cluster = ctx.contract.cluster_get(ctx.cluster_id)?;
    assert_eq!(cluster.resource_per_vnode, 10 + 5);

    // Check the changed state of the nodes.
    let expected_resources = [
        (ctx.node_id0, 100 - 10 - 10 - 5 - 5),
        (ctx.node_id1, 100 - 10 - 10 - 5 - 5),
        (ctx.node_id2, 100 - 10 - 10 - 5 - 5),
    ];
    for (node_id, available) in expected_resources {
        assert_eq!(
            ctx.contract.node_get(node_id)?.free_resource,
            available, "more resources must be reserved from the nodes");
    }
}


#[ink::test]
fn cluster_management_validation_works() {
    let mut ctx = new_cluster();

    let not_manager = ctx.provider_id0;
    push_caller_value(not_manager, 0);
    assert_eq!(
        ctx.contract.message_cluster_replace_node(ctx.cluster_id, 0, 1),
        Err(UnauthorizedClusterManager), "only the manager can modify the cluster");
    pop_caller();

    push_caller_value(ctx.manager, 0);

    let bad_node_id = ctx.node_id2 + 1;
    assert_eq!(
        ctx.contract.message_cluster_replace_node(ctx.cluster_id, 0, bad_node_id),
        Err(NodeDoesNotExist), "cluster replacement node must exist");

    assert_eq!(
        ctx.contract.message_cluster_create(ctx.manager, 2, vec![bad_node_id], "".to_string()),
        Err(NodeDoesNotExist), "cluster initial nodes must exist");
}


fn bucket_settle_payment(ctx: &mut TestCluster, test_bucket: &TestBucket) {
    // Go to the future when some revenues are due.
    advance_block::<DefaultEnvironment>().unwrap();

    // Pay the due thus far.
    push_caller_value(ctx.manager, CONTRACT_FEE_LIMIT);
    ctx.contract.bucket_settle_payment(test_bucket.bucket_id);
    pop_caller();
}


#[ink::test]
fn bucket_pays_cluster() {
    let ctx = &mut new_cluster();
    let test_bucket = &new_bucket(ctx);

    let expected_rent = ctx.rent_per_vnode * ctx.partition_count as Balance;

    // Check the state before payment.
    let before = ctx.contract
        .account_get(test_bucket.owner_id)?
        .deposit.peek();
    let bucket = ctx.contract.bucket_get(test_bucket.bucket_id)?;
    assert_eq!(bucket.owner_id, test_bucket.owner_id);
    assert_eq!(bucket.flows[0],
               Flow {
                   from: test_bucket.owner_id,
                   schedule: Schedule::new(0, expected_rent),
               });

    bucket_settle_payment(ctx, &test_bucket);

    // Check the state after payment.
    let after = ctx.contract
        .account_get(test_bucket.owner_id)?
        .deposit.peek();
    let spent = before - after;
    let bucket = ctx.contract.bucket_get(test_bucket.bucket_id)?;
    assert_eq!(bucket.flows[0],
               Flow {
                   from: test_bucket.owner_id,
                   schedule: Schedule::new(BLOCK_TIME, expected_rent),
               });

    let expect_revenues = expected_rent * BLOCK_TIME as u128 / MS_PER_MONTH as u128;
    assert!(expect_revenues > 0);
    assert_eq!(expect_revenues, spent, "revenues must come from the bucket owner");

    let cluster = ctx.contract.cluster_get(ctx.cluster_id)?;
    assert_eq!(cluster.revenues.peek(), expect_revenues, "must get revenues into the cluster");
}


#[ink::test]
fn cluster_pays_providers() {
    let ctx = &mut new_cluster();
    let test_bucket = &new_bucket(ctx);
    bucket_settle_payment(ctx, &test_bucket);

    // Get state before the distribution.
    let to_distribute = ctx.contract.cluster_get(ctx.cluster_id)?.revenues.peek();
    let before0 = balance_of(ctx.provider_id0);
    let before1 = balance_of(ctx.provider_id1);
    let before2 = balance_of(ctx.provider_id2);

    // Distribute the revenues of the cluster to providers.
    ctx.contract.cluster_distribute_revenues(ctx.cluster_id);

    // Get state after the distribution.
    let left_after_distribution = ctx.contract.cluster_get(ctx.cluster_id)?.revenues.peek();
    let earned0 = balance_of(ctx.provider_id0) - before0;
    let earned1 = balance_of(ctx.provider_id1) - before1;
    let earned2 = balance_of(ctx.provider_id2) - before2;

    assert!(to_distribute > 0);
    assert!(left_after_distribution < 10, "revenues must go out of the cluster (besides rounding)");
    assert!(earned0 > 0, "provider must earn something");
    assert_eq!(earned0, earned1, "providers must earn the same amount");
    assert_eq!(earned0, earned2, "providers must earn the same amount");
    assert_eq!(earned0 + earned1 + earned2 + left_after_distribution, to_distribute, "all revenues must go to providers");
}


#[ink::test]
fn ddc_bucket_works() {
    let accounts = get_accounts();
    set_balance(accounts.charlie, 1000 * TOKEN);
    let provider_id0 = accounts.alice;
    let provider_id1 = accounts.bob;
    let consumer_id = accounts.charlie;
    let cluster_manager = provider_id0;

    let mut ddc_bucket = DdcBucket::new();

    // Provide a Node.
    let rent_per_month: Balance = 10 * TOKEN;
    let node_params0 = "{\"url\":\"https://ddc.cere.network/bucket/{BUCKET_ID}\"}";
    let capacity = 100;
    push_caller_value(provider_id0, CONTRACT_FEE_LIMIT);
    let node_id0 = ddc_bucket.node_create(rent_per_month, node_params0.to_string(), capacity);
    pop_caller();

    // Provide another Node.
    let node_params1 = "{\"url\":\"https://ddc-2.cere.network/bucket/{BUCKET_ID}\"}";
    push_caller_value(provider_id1, CONTRACT_FEE_LIMIT);
    let node_id1 = ddc_bucket.node_create(rent_per_month, node_params1.to_string(), capacity);
    pop_caller();
    assert_ne!(node_id0, node_id1);

    // Create a Cluster.
    let cluster_params = "{}";
    push_caller_value(provider_id0, CONTRACT_FEE_LIMIT);
    let cluster_id = ddc_bucket.cluster_create(cluster_manager, 2, vec![node_id0, node_id1], cluster_params.to_string());
    pop_caller();

    push_caller_value(cluster_manager, 0);
    ddc_bucket.cluster_reserve_resource(cluster_id, 10);
    pop_caller();

    // Consumer discovers the Cluster with the 2 Nodes.
    let cluster = ddc_bucket.cluster_get(cluster_id)?;
    let total_rent = rent_per_month * 2;
    assert_eq!(cluster, Cluster {
        cluster_id,
        manager_id: cluster_manager,
        cluster_params: cluster_params.to_string(),
        vnodes: vec![node_id0, node_id1],
        resource_per_vnode: 10,
        resource_used: 0,
        revenues: Cash(0),
        total_rent,
    });
    let node0 = ddc_bucket.node_get(node_id0)?;
    assert_eq!(node0, Node {
        node_id: node_id0,
        provider_id: provider_id0,
        rent_per_month,
        node_params: node_params0.to_string(),
        free_resource: capacity - cluster.resource_per_vnode,
    });
    let node1 = ddc_bucket.node_get(node_id1)?;
    assert_eq!(node1, Node {
        node_id: node_id1,
        provider_id: provider_id1,
        rent_per_month,
        node_params: node_params1.to_string(),
        free_resource: capacity - cluster.resource_per_vnode,
    });

    // Deposit some value to pay for buckets.
    push_caller_value(consumer_id, 10 * TOKEN);
    ddc_bucket.account_deposit();
    pop_caller();

    // Create a bucket.
    push_caller_value(consumer_id, CONTRACT_FEE_LIMIT);
    let bucket_params = "{}".to_string();
    let bucket_id = ddc_bucket.bucket_create(bucket_params.clone(), cluster_id);
    pop_caller();

    // Allocate the bucket to the cluster.
    push_caller_value(consumer_id, CONTRACT_FEE_LIMIT);
    let resource_reserved = 1;
    ddc_bucket.bucket_alloc_into_cluster(bucket_id, resource_reserved);
    pop_caller();

    // Check the structure of the bucket including the payment flow.
    let bucket = ddc_bucket.bucket_get(bucket_id)?;
    assert_eq!(bucket, Bucket {
        owner_id: consumer_id,
        cluster_id,
        flows: vec![Flow { from: consumer_id, schedule: Schedule::new(0, total_rent) }],
        bucket_params: bucket_params.to_string(),
        resource_reserved,
    });

    // Deposit more value into the account.
    push_caller_value(consumer_id, 100 * TOKEN);
    ddc_bucket.account_deposit();
    pop_caller();

    // Check the status of the bucket.
    let bucket_status = ddc_bucket.bucket_get_status(bucket_id)?;
    assert_eq!(bucket_status, BucketStatus {
        bucket_id,
        bucket,
        writer_ids: vec![consumer_id],
        rent_covered_until_ms: 14559782400, // TODO: check this value.
    });

    // Go to the future when some revenues are due.
    advance_block::<DefaultEnvironment>().unwrap();

    ddc_bucket.cluster_distribute_revenues(cluster_id);

    let mut evs = get_events(7);
    evs.reverse(); // Work with pop().

    // Provider setup.
    assert!(matches!(evs.pop().unwrap(), Event::NodeCreated(ev) if ev ==
        NodeCreated { node_id: node_id0, provider_id: provider_id0, rent_per_month, node_params: node_params0.to_string() }));

    // Provider setup 2.
    assert!(matches!(evs.pop().unwrap(), Event::NodeCreated(ev) if ev ==
        NodeCreated { node_id: node_id1, provider_id: provider_id1, rent_per_month, node_params: node_params1.to_string() }));

    // Cluster setup.
    assert!(matches!(evs.pop().unwrap(), Event::ClusterCreated(ev) if ev ==
        ClusterCreated { cluster_id, manager: cluster_manager, cluster_params: cluster_params.to_string() }));

    // Deposit.
    let deposit_contract_fee = calculate_contract_fee(Account::RECORD_SIZE).peek();
    let net_deposit = 10 * TOKEN - deposit_contract_fee;
    assert!(matches!(evs.pop().unwrap(), Event::Deposit(ev) if ev ==
        Deposit { account_id: consumer_id, value: net_deposit }));

    // Create bucket.
    assert!(matches!(evs.pop().unwrap(), Event::BucketCreated(ev) if ev ==
        BucketCreated {  bucket_id, owner_id: consumer_id }));

    assert!(matches!(evs.pop().unwrap(), Event::BucketAllocated(ev) if ev ==
        BucketAllocated { bucket_id, cluster_id }));

    // Deposit more.
    let net_deposit = 100 * TOKEN; // No deposit_contract_fee because the account already exists.
    assert!(matches!(evs.pop().unwrap(), Event::Deposit(ev) if ev ==
        Deposit { account_id: consumer_id, value: net_deposit }));
}


#[ink::test]
fn bucket_list_works() {
    let mut ddc_bucket = DdcBucket::new();
    let accounts = get_accounts();
    let owner_id1 = accounts.alice;
    let owner_id2 = accounts.bob;
    let owner_id3 = accounts.charlie;
    let cluster_id = 0;

    push_caller_value(owner_id1, CONTRACT_FEE_LIMIT);
    let bucket_id1 = ddc_bucket.bucket_create("".to_string(), cluster_id);
    let bucket_status1 = ddc_bucket.bucket_get_status(bucket_id1).unwrap();
    pop_caller();

    push_caller_value(owner_id2, CONTRACT_FEE_LIMIT);
    let bucket_id2 = ddc_bucket.bucket_create("".to_string(), cluster_id);
    let bucket_status2 = ddc_bucket.bucket_get_status(bucket_id2)?;
    pop_caller();

    assert_ne!(bucket_id1, bucket_id2);
    let count = 2;

    assert_eq!(
        ddc_bucket.bucket_list_statuses(0, 100, None),
        (vec![bucket_status1.clone(), bucket_status2.clone()], count));

    assert_eq!(
        ddc_bucket.bucket_list_statuses(0, 2, None),
        (vec![bucket_status1.clone(), bucket_status2.clone()], count));

    assert_eq!(
        ddc_bucket.bucket_list_statuses(0, 1, None),
        (vec![bucket_status1.clone() /*, bucket_status2.clone()*/], count));

    assert_eq!(
        ddc_bucket.bucket_list_statuses(1, 1, None),
        (vec![/*bucket_status1.clone(),*/ bucket_status2.clone()], count));

    assert_eq!(
        ddc_bucket.bucket_list_statuses(20, 20, None),
        (vec![], count));

    // Filter by owner.
    assert_eq!(
        ddc_bucket.bucket_list_statuses(0, 100, Some(owner_id1)),
        (vec![bucket_status1.clone() /*, bucket_status2.clone()*/], count));

    assert_eq!(
        ddc_bucket.bucket_list_statuses(0, 100, Some(owner_id2)),
        (vec![/*bucket_status1.clone(),*/ bucket_status2.clone()], count));

    assert_eq!(
        ddc_bucket.bucket_list_statuses(0, 100, Some(owner_id3)),
        (vec![], count));
}


#[ink::test]
fn node_list_works() {
    let mut ddc_bucket = DdcBucket::new();
    let accounts = get_accounts();
    let owner_id1 = accounts.alice;
    let owner_id2 = accounts.bob;
    let owner_id3 = accounts.charlie;
    let rent_per_month: Balance = 10 * TOKEN;

    // Create two Nodes.
    let node_params1 = "{\"url\":\"https://ddc-1.cere.network/bucket/{BUCKET_ID}\"}";
    let capacity = 100;
    push_caller_value(owner_id1, CONTRACT_FEE_LIMIT);
    let node_id1 = ddc_bucket.node_create(rent_per_month, node_params1.to_string(), capacity);
    pop_caller();

    let node_params2 = "{\"url\":\"https://ddc-2.cere.network/bucket/{BUCKET_ID}\"}";
    push_caller_value(owner_id2, CONTRACT_FEE_LIMIT);
    let node_id2 = ddc_bucket.node_create(rent_per_month, node_params2.to_string(), capacity);
    pop_caller();

    assert_ne!(node_id1, node_id2);
    let count = 2;

    let node1 = Node {
        node_id: node_id1,
        provider_id: owner_id1,
        rent_per_month,
        node_params: node_params1.to_string(),
        free_resource: capacity,
    };

    let node2 = Node {
        node_id: node_id2,
        provider_id: owner_id2,
        rent_per_month,
        node_params: node_params2.to_string(),
        free_resource: capacity,
    };

    assert_eq!(
        ddc_bucket.node_list(0, 100, None),
        (vec![node1.clone(), node2.clone()], count));

    assert_eq!(
        ddc_bucket.node_list(0, 2, None),
        (vec![node1.clone(), node2.clone()], count));

    assert_eq!(
        ddc_bucket.node_list(0, 1, None),
        (vec![node1.clone() /*, node2.clone()*/], count));

    assert_eq!(
        ddc_bucket.node_list(1, 1, None),
        (vec![/*node1.clone(),*/ node2.clone()], count));

    assert_eq!(
        ddc_bucket.node_list(20, 20, None),
        (vec![], count));

    // Filter by owner.
    assert_eq!(
        ddc_bucket.node_list(0, 100, Some(owner_id1)),
        (vec![node1.clone() /*, node2.clone()*/], count));

    assert_eq!(
        ddc_bucket.node_list(0, 100, Some(owner_id2)),
        (vec![/*node1.clone(),*/ node2.clone()], count));

    assert_eq!(
        ddc_bucket.node_list(0, 100, Some(owner_id3)),
        (vec![], count));
}

#[ink::test]
fn contract_fee_works() {
    let mut ddc_bucket = DdcBucket::new();
    let accounts = get_accounts();
    let owner_id = accounts.alice;
    let alice_before = balance_of(accounts.alice);
    let cluster_id = 0;

    push_caller_value(owner_id, CONTRACT_FEE_LIMIT);
    let bucket_id = ddc_bucket.bucket_create("123".to_string(), cluster_id);

    let bucket = ddc_bucket.bucket_get(bucket_id)?;
    let expect_fee = FEE_PER_BYTE * (SIZE_PER_RECORD + bucket.encoded_size() + Account::new().encoded_size()) as Balance;
    let got_fee = balance_of(contract_id());
    assert!(expect_fee <= got_fee, "A sufficient contract fee should be taken.");
    assert!(got_fee <= 2 * expect_fee, "The contract fee should not be excessive.");
    assert!(got_fee < CONTRACT_FEE_LIMIT, "Value beyond the contract fee should be refunded.");
    let alice_after = balance_of(accounts.alice);
    assert_eq!(alice_after + got_fee, alice_before, "Accounts should be balanced.");
}
