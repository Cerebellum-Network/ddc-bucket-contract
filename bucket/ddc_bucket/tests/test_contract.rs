use ink_lang as ink;

use crate::ddc_bucket::*;
use crate::ddc_bucket::account::entity::Account;
use crate::ddc_bucket::contract_fee::{calculate_contract_fee, FEE_PER_BYTE, SIZE_PER_RECORD};

use super::env_utils::*;

struct Context {
    contract: DdcBucket,
    manager: AccountId,
    cluster_id: ClusterId,
    provider_id0: AccountId,
    node_id0: NodeId,
    node_id1: NodeId,
    node_id2: NodeId,
}

fn new_cluster() -> Context {
    let accounts = get_accounts();
    set_balance(accounts.charlie, 1000 * TOKEN);
    let provider_id0 = accounts.alice;
    let provider_id1 = accounts.bob;
    let provider_id2 = accounts.charlie;
    let manager = accounts.charlie;

    let mut contract = DdcBucket::new();

    // Provide a Node.
    let rent_per_month: Balance = 10 * TOKEN;
    let node_params0 = "{\"url\":\"https://ddc.cere.network/bucket/{BUCKET_ID}\"}";
    let capacity = 100;
    push_caller_value(provider_id0, CONTRACT_FEE_LIMIT);
    let node_id0 = contract.node_create(rent_per_month, node_params0.to_string(), capacity);
    pop_caller();

    // Provide another Node.
    let node_params1 = "{\"url\":\"https://ddc-2.cere.network/bucket/{BUCKET_ID}\"}";
    push_caller_value(provider_id1, CONTRACT_FEE_LIMIT);
    let node_id1 = contract.node_create(rent_per_month, node_params1.to_string(), capacity);
    pop_caller();

    // Provide another Node.
    let node_params2 = "{\"url\":\"https://ddc-2.cere.network/bucket/{BUCKET_ID}\"}";
    push_caller_value(provider_id2, CONTRACT_FEE_LIMIT);
    let node_id2 = contract.node_create(rent_per_month, node_params2.to_string(), capacity);
    pop_caller();

    // Create a Cluster.
    let cluster_params = "{}";
    push_caller_value(manager, CONTRACT_FEE_LIMIT);
    let cluster_id = contract.cluster_create(manager, 6, vec![node_id0, node_id1, node_id2], cluster_params.to_string());
    pop_caller();

    push_caller_value(manager, 0);
    contract.cluster_reserve_resource(cluster_id, 10);
    pop_caller();

    Context { contract, manager, cluster_id, provider_id0, node_id0, node_id1, node_id2 }
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

    // Consumer discovers the Cluster with the 2 Nodes.
    let cluster = ddc_bucket.cluster_get(cluster_id)?;
    assert_eq!(cluster, Cluster {
        cluster_id,
        manager_id: cluster_manager,
        cluster_params: cluster_params.to_string(),
        vnodes: vec![node_id0, node_id1],
        resource_per_vnode: 0,
        resource_used: 0,
    });
    let node0 = ddc_bucket.node_get(node_id0)?;
    assert_eq!(node0, Node {
        node_id: node_id0,
        provider_id: provider_id0,
        rent_per_month,
        node_params: node_params0.to_string(),
        free_resource: capacity,
    });
    let node1 = ddc_bucket.node_get(node_id1)?;
    assert_eq!(node1, Node {
        node_id: node_id1,
        provider_id: provider_id1,
        rent_per_month,
        node_params: node_params1.to_string(),
        free_resource: capacity,
    });

    // Deposit some value to pay for buckets.
    push_caller_value(consumer_id, 10 * TOKEN);
    ddc_bucket.deposit();
    pop_caller();

    // Create a bucket.
    push_caller_value(consumer_id, CONTRACT_FEE_LIMIT);
    let bucket_params = "{}".to_string();
    let bucket_id = ddc_bucket.bucket_create(bucket_params.clone());
    pop_caller();

    // Allocate the bucket to the cluster.
    push_caller_value(consumer_id, CONTRACT_FEE_LIMIT);
    ddc_bucket.bucket_alloc_into_cluster(bucket_id, cluster_id);
    pop_caller();

    // Check the structure of the bucket including all deal IDs.
    let bucket = ddc_bucket.bucket_get(bucket_id)?;
    let deal_id0 = 0;
    let deal_id1 = 1;
    assert_eq!(bucket, Bucket {
        owner_id: consumer_id,
        cluster_ids: vec![cluster_id],
        deal_ids: vec![deal_id0, deal_id1],
        bucket_params: bucket_params.to_string(),
        resource_reserved: 0,
    });

    // Check the status of the deal.
    let deal_status0 = ddc_bucket.deal_get_status(deal_id0)?;
    assert_eq!(deal_status0, DealStatus {
        node_id: node_id0,
        estimated_rent_end_ms: 1167782400, // TODO: calculate this value.
    });

    // Deposit more value into the account.
    push_caller_value(consumer_id, 100 * TOKEN);
    ddc_bucket.deposit();
    pop_caller();

    // The end time increased because there is more deposit.
    let deal_status0 = ddc_bucket.deal_get_status(deal_id0)?;
    assert_eq!(deal_status0, DealStatus {
        node_id: node_id0,
        estimated_rent_end_ms: 14388364800, // TODO: calculate this value.
    });

    // The end time of the second deal is the same because it is paid from the same account.
    let deal_status1 = ddc_bucket.deal_get_status(deal_id1)?;
    assert_eq!(deal_status1, DealStatus {
        node_id: node_id1,
        estimated_rent_end_ms: 14388364800, // TODO: calculate this value.
    });

    // Check the status of the bucket recursively including all deal statuses.
    let bucket_status = ddc_bucket.bucket_get_status(bucket_id)?;
    assert_eq!(bucket_status, BucketStatus {
        bucket_id,
        bucket,
        writer_ids: vec![consumer_id],
        deal_statuses: vec![deal_status0, deal_status1],
    });

    // A provider is looking for the status of his deal with a bucket.
    let deal_of_provider = bucket_status.deal_statuses
        .iter().find(|deal|
        deal.node_id == node_id1);
    assert!(deal_of_provider.is_some());

    // Provider withdraws in the future.
    advance_block::<DefaultEnvironment>().unwrap();
    push_caller(provider_id0);
    ddc_bucket.provider_withdraw(deal_id0);
    pop_caller();

    push_caller(provider_id1);
    ddc_bucket.provider_withdraw(deal_id1);
    pop_caller();

    let mut evs = get_events(11);
    evs.reverse();

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

    // Add a cluster with 2 deals and an initial deposit.
    assert!(matches!(evs.pop().unwrap(), Event::BucketAllocated(ev) if ev ==
        BucketAllocated { bucket_id, cluster_id }));
    assert!(matches!(evs.pop().unwrap(), Event::DealCreated(ev) if ev ==
        DealCreated { deal_id: deal_id0, bucket_id, node_id: node_id0 }));
    assert!(matches!(evs.pop().unwrap(), Event::DealCreated(ev) if ev ==
        DealCreated { deal_id: deal_id1, bucket_id, node_id: node_id1 }));

    // Deposit more.
    let net_deposit = 100 * TOKEN - deposit_contract_fee;
    assert!(matches!(evs.pop().unwrap(), Event::Deposit(ev) if ev ==
        Deposit { account_id: consumer_id, value: net_deposit }));

    // Provider withdrawals.
    assert!(matches!(evs.pop().unwrap(), Event::ProviderWithdraw(ev) if ev ==
        ProviderWithdraw { provider_id: provider_id0, deal_id: deal_id0, value: 186 }));
    assert!(matches!(evs.pop().unwrap(), Event::ProviderWithdraw(ev) if ev ==
        ProviderWithdraw { provider_id: provider_id1, deal_id: deal_id1, value: 186 }));
}


#[ink::test]
fn bucket_list_works() {
    let mut ddc_bucket = DdcBucket::new();
    let accounts = get_accounts();
    let owner_id1 = accounts.alice;
    let owner_id2 = accounts.bob;
    let owner_id3 = accounts.charlie;

    push_caller_value(owner_id1, CONTRACT_FEE_LIMIT);
    let bucket_id1 = ddc_bucket.bucket_create("".to_string());
    let bucket_status1 = ddc_bucket.bucket_get_status(bucket_id1)?;
    pop_caller();

    push_caller_value(owner_id2, CONTRACT_FEE_LIMIT);
    let bucket_id2 = ddc_bucket.bucket_create("".to_string());
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

    push_caller_value(owner_id, CONTRACT_FEE_LIMIT);
    let bucket_id = ddc_bucket.bucket_create("123".to_string());

    let bucket = ddc_bucket.bucket_get(bucket_id)?;
    let expect_fee = FEE_PER_BYTE * (SIZE_PER_RECORD + bucket.encoded_size()) as Balance;
    let got_fee = balance_of(contract_id());
    assert!(expect_fee <= got_fee, "A sufficient contract fee should be taken.");
    assert!(got_fee <= 2 * expect_fee, "The contract fee should not be excessive.");
    assert!(got_fee < CONTRACT_FEE_LIMIT, "Value beyond the contract fee should be refunded.");
    let alice_after = balance_of(accounts.alice);
    assert_eq!(alice_after + got_fee, alice_before, "Accounts should be balanced.");
}
