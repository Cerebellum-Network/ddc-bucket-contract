use ink_lang as ink;

use crate::ddc_bucket::*;
use crate::ddc_bucket::account::entity::Account;
use crate::ddc_bucket::contract_fee::{calculate_contract_fee, FEE_PER_BYTE, SIZE_PER_RECORD};
use crate::ddc_bucket::flow::Flow;
use crate::ddc_bucket::schedule::{MS_PER_MONTH, Schedule};

use super::env_utils::*;

fn admin_id() -> AccountId {
    get_accounts().alice
}

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
    vnode_count: u32,
    node_params0: &'static str,
    node_params1: &'static str,
    node_params2: &'static str,
    capacity: u32,
    reserved: u32,
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

    for provider_id in [provider_id0, provider_id1, provider_id2] {
        push_caller_value(provider_id, CONTRACT_FEE_LIMIT);
        contract.node_trust_manager(manager);
        let expected_perm = Permission::ManagerTrustedBy(provider_id);
        assert!(contract.has_permission(manager, expected_perm));
        pop_caller();
    }

    push_caller_value(provider_id0, CONTRACT_FEE_LIMIT);
    let node_id0 = contract.node_create(rent_per_vnode, node_params0.to_string(), capacity);
    pop_caller();

    // Provide another Node.
    let node_params1 = "{\"url\":\"https://ddc-1.cere.network/bucket/{BUCKET_ID}\"}";
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
    let vnode_count = 6;
    push_caller_value(manager, CONTRACT_FEE_LIMIT);
    let cluster_id = contract.cluster_create(manager, vnode_count, vec![node_id0, node_id1, node_id2], cluster_params.to_string());
    pop_caller();

    push_caller_value(manager, 0);
    let reserved = 10;
    contract.cluster_reserve_resource(cluster_id, reserved);
    pop_caller();

    TestCluster { contract, manager, cluster_id, provider_id0, provider_id1, provider_id2, node_id0, node_id1, node_id2, rent_per_vnode, vnode_count, node_params0, node_params1, node_params2, capacity, reserved }
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

    push_caller_value(owner_id, CONTRACT_FEE_LIMIT);
    let bucket_id = ctx.contract.bucket_create("{}".to_string(), ctx.cluster_id);
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

    TestBucket { bucket_id, owner_id, resource }
}


#[ink::test]
fn cluster_create_works() {
    let ctx = new_cluster();
    let provider_ids = &[ctx.provider_id0, ctx.provider_id1, ctx.provider_id2];
    let node_ids = &[ctx.node_id0, ctx.node_id1, ctx.node_id2];
    let node_params = &[ctx.node_params0, ctx.node_params1, ctx.node_params2];

    assert_eq!(ctx.cluster_id, 1, "cluster_id must start at 1");
    assert_eq!(ctx.node_id0, 1, "node_id must start at 1");
    assert_ne!(ctx.node_id0, ctx.node_id1, "nodes must have unique IDs");

    // Check the nodes.
    {
        let node0 = ctx.contract.node_get(ctx.node_id0)?;
        assert_eq!(node0, NodeStatus {
            node_id: ctx.node_id0,
            node: Node {
                provider_id: ctx.provider_id0,
                rent_per_month: ctx.rent_per_vnode,
                free_resource: ctx.capacity - ctx.reserved * 2,
            },
            params: ctx.node_params0.to_string(),
        });

        let node1 = ctx.contract.node_get(ctx.node_id1)?;
        assert_eq!(node1, NodeStatus {
            node_id: ctx.node_id1,
            node: Node {
                provider_id: ctx.provider_id1,
                rent_per_month: ctx.rent_per_vnode,
                free_resource: ctx.capacity - ctx.reserved * 2,
            },
            params: ctx.node_params1.to_string(),
        });

        let node2 = ctx.contract.node_get(ctx.node_id2)?;
        assert_eq!(node2, NodeStatus {
            node_id: ctx.node_id2,
            node: Node {
                provider_id: ctx.provider_id2,
                rent_per_month: ctx.rent_per_vnode,
                free_resource: ctx.capacity - ctx.reserved * 2,
            },
            params: ctx.node_params2.to_string(),
        });
    }

    // Check the initial state of the cluster.
    {
        let cluster = ctx.contract.cluster_get(ctx.cluster_id)?;
        assert_eq!(cluster, ClusterStatus {
            cluster_id: ctx.cluster_id,
            cluster: Cluster {
                manager_id: ctx.manager,
                vnodes: vec![
                    ctx.node_id0, ctx.node_id1, ctx.node_id2,
                    ctx.node_id0, ctx.node_id1, ctx.node_id2],
                resource_per_vnode: ctx.reserved,
                resource_used: 0,
                revenues: Cash(0),
                total_rent: ctx.rent_per_vnode * ctx.vnode_count as Balance,
            },
            params: "{}".to_string(),
        });
    }

    // Check the events.
    let mut evs = get_events();
    evs.reverse(); // Work with pop().

    // Providers trust Manager.
    for provider_id in provider_ids {
        assert!(matches!(evs.pop().unwrap(), Event::GrantPermission(ev) if ev ==
            GrantPermission { account_id: ctx.manager, permission: Permission::ManagerTrustedBy(*provider_id) }));
    }

    // Nodes created.
    for i in 0..3 {
        assert!(matches!(evs.pop().unwrap(), Event::NodeCreated(ev) if ev ==
            NodeCreated {
                node_id: node_ids[i],
                provider_id: provider_ids[i],
                rent_per_month: ctx.rent_per_vnode,
                node_params: node_params[i].to_string() }));
    }

    // Cluster setup.
    assert!(matches!(evs.pop().unwrap(), Event::ClusterCreated(ev) if ev ==
        ClusterCreated { cluster_id: ctx.cluster_id, manager: ctx.manager, cluster_params: "{}".to_string() }));

    assert!(matches!(evs.pop().unwrap(), Event::ClusterReserveResource(ev) if ev ==
        ClusterReserveResource { cluster_id: ctx.cluster_id, resource: ctx.reserved }));

    assert_eq!(evs.len(), 0, "all events must be checked");
}


#[ink::test]
fn cluster_replace_node_works() {
    let mut ctx = new_cluster();
    push_caller_value(ctx.manager, 0);

    // Reassign a vnode from node1 to node2.
    ctx.contract.cluster_replace_node(ctx.cluster_id, 1, ctx.node_id2);

    // Check the last event.
    let ev = get_events().pop().unwrap();
    assert!(matches!(ev, Event::ClusterNodeReplaced(ev) if ev ==
        ClusterNodeReplaced { cluster_id: ctx.cluster_id, node_id: ctx.node_id2, vnode_index: 1 }));

    // Check the changed state of the cluster.
    let cluster = ctx.contract.cluster_get(ctx.cluster_id)?.cluster;
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
            ctx.contract.node_get(node_id)?.node.free_resource,
            available, "resources must have shifted between nodes");
    }
}


#[ink::test]
fn cluster_reserve_works() {
    let mut ctx = new_cluster();
    push_caller_value(ctx.manager, 0);

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
        (ctx.node_id0, 100 - 10 - 10 - 5 - 5),
        (ctx.node_id1, 100 - 10 - 10 - 5 - 5),
        (ctx.node_id2, 100 - 10 - 10 - 5 - 5),
    ];
    for (node_id, available) in expected_resources {
        assert_eq!(
            ctx.contract.node_get(node_id)?.node.free_resource,
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
    do_bucket_pays_cluster(ctx, test_bucket, 1).unwrap();
}

#[ink::test]
fn bucket_pays_cluster_at_new_rate() {
    let ctx = &mut new_cluster();
    let test_bucket = &new_bucket(ctx);

    // Set up an exchange rate manager.
    push_caller_value(admin_id(), CONTRACT_FEE_LIMIT);
    ctx.contract.admin_grant_permission(admin_id(), Permission::SetExchangeRate);
    pop_caller();

    // Change the currency exchange rate.
    let usd_per_cere = 2;
    push_caller(admin_id());
    ctx.contract.account_set_usd_per_cere(usd_per_cere * TOKEN);
    pop_caller();

    do_bucket_pays_cluster(ctx, test_bucket, usd_per_cere).unwrap();
}

fn do_bucket_pays_cluster(ctx: &mut TestCluster, test_bucket: &TestBucket, usd_per_cere: Balance) -> Result<()> {
    let expected_rent = ctx.rent_per_vnode * ctx.vnode_count as Balance;

    // Check the state before payment.
    let before = ctx.contract
        .account_get(test_bucket.owner_id)?
        .deposit.peek();
    let bucket = ctx.contract.bucket_get(test_bucket.bucket_id)?.bucket;
    assert_eq!(bucket.owner_id, test_bucket.owner_id);
    /* TODO: Not testable at the moment, see struct BucketInStatus.
    assert_eq!(bucket.flow,
               Flow {
                   from: test_bucket.owner_id,
                   schedule: Schedule::new(0, expected_rent),
               });
    */

    bucket_settle_payment(ctx, &test_bucket);

    // Check the last event.
    let ev = get_events().pop().unwrap();
    assert!(matches!(ev, Event::BucketSettlePayment(ev) if ev ==
        BucketSettlePayment {  bucket_id: test_bucket.bucket_id, cluster_id: ctx.cluster_id }));

    // Check the state after payment.
    let after = ctx.contract
        .account_get(test_bucket.owner_id)?
        .deposit.peek();
    let spent = before - after;
    /* TODO: Not testable at the moment, see struct BucketInStatus.
    let bucket = ctx.contract.bucket_get(test_bucket.bucket_id)?.bucket;
    assert_eq!(bucket.flow,
               Flow {
                   from: test_bucket.owner_id,
                   schedule: Schedule::new(BLOCK_TIME, expected_rent),
               });
    */

    let expect_revenues_usd = expected_rent * BLOCK_TIME as u128 / MS_PER_MONTH as u128;
    let expect_revenues = expect_revenues_usd / usd_per_cere;
    assert!(expect_revenues > 0);
    assert_eq!(expect_revenues, spent, "revenues must come from the bucket owner");

    let cluster = ctx.contract.cluster_get(ctx.cluster_id)?.cluster;
    assert_eq!(cluster.revenues.peek(), expect_revenues, "must get revenues into the cluster");

    Ok(())
}


#[ink::test]
fn cluster_pays_providers() {
    let ctx = &mut new_cluster();
    let test_bucket = &new_bucket(ctx);
    bucket_settle_payment(ctx, &test_bucket);

    // Get state before the distribution.
    let to_distribute = ctx.contract.cluster_get(ctx.cluster_id)?
        .cluster.revenues.peek();
    let before0 = balance_of(ctx.provider_id0);
    let before1 = balance_of(ctx.provider_id1);
    let before2 = balance_of(ctx.provider_id2);

    let skip_events = get_events::<Event>().len();

    // Distribute the revenues of the cluster to providers.
    ctx.contract.cluster_distribute_revenues(ctx.cluster_id);

    // Check the last events.
    let mut evs = get_events();
    evs.reverse(); // Work with pop().
    evs.truncate(evs.len() - skip_events);
    let expected_recipients = vec![
        ctx.provider_id0, ctx.provider_id1, ctx.provider_id2,
        ctx.provider_id0, ctx.provider_id1, ctx.provider_id2,
    ];
    for provider_id in expected_recipients {
        assert!(matches!(evs.pop().unwrap(), Event::ClusterDistributeRevenues(ev) if ev ==
            ClusterDistributeRevenues { cluster_id: ctx.cluster_id, provider_id }));
    }
    assert_eq!(evs.len(), 0, "all events must be checked");

    // Get state after the distribution.
    let left_after_distribution = ctx.contract.cluster_get(ctx.cluster_id)?
        .cluster.revenues.peek();
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
fn bucket_reserve_0_works() {
    let contract = DdcBucket::new();

    assert_eq!(
        contract.bucket_list(0, 10, None),
        (vec![BucketStatus {
            bucket_id: 0,
            bucket: BucketInStatus {
                owner_id: AccountId::default(),
                cluster_id: 0,
                resource_reserved: 0,
            },
            params: "".to_string(),
            writer_ids: vec![AccountId::default()],
            rent_covered_until_ms: 18446744073709551615,
        }], 1));

    assert_eq!(
        contract.cluster_list(0, 10, None),
        (vec![ClusterStatus {
            cluster_id: 0,
            cluster: Cluster {
                manager_id: AccountId::default(),
                vnodes: vec![],
                resource_per_vnode: 0,
                resource_used: 0,
                revenues: Cash(0),
                total_rent: 0,
            },
            params: "".to_string(),
        }], 1));

    assert_eq!(
        contract.node_list(0, 10, None),
        (vec![NodeStatus {
            node_id: 0,
            node: Node {
                provider_id: AccountId::default(),
                rent_per_month: 0,
                free_resource: 0,
            },
            params: "".to_string(),
        }], 1));
}


#[ink::test]
fn bucket_create_works() {
    let ctx = &mut new_cluster();
    let test_bucket = &new_bucket(ctx);

    assert_eq!(test_bucket.bucket_id, 1, "bucket_id must start at 1");

    // Check the structure of the bucket including the payment flow.
    let total_rent = ctx.rent_per_vnode * ctx.vnode_count as Balance;
    let expect_bucket = Bucket {
        owner_id: test_bucket.owner_id,
        cluster_id: ctx.cluster_id,
        flow: Flow {
            from: test_bucket.owner_id,
            schedule: Schedule::new(0, total_rent),
        },
        resource_reserved: test_bucket.resource,
    };

    // Check the status of the bucket.
    let bucket_status = ctx.contract.bucket_get(test_bucket.bucket_id)?;
    assert_eq!(bucket_status, BucketStatus {
        bucket_id: test_bucket.bucket_id,
        bucket: expect_bucket.into(),
        params: "{}".to_string(),
        writer_ids: vec![test_bucket.owner_id],
        rent_covered_until_ms: 446400000, // TODO: check this value.
    });

    let mut evs = get_events();
    evs.reverse(); // Work with pop().
    evs.truncate(8 - 3 - 2); // Skip 3 NodeCreated and 2 cluster setup events.

    // Create bucket.
    assert!(matches!(evs.pop().unwrap(), Event::BucketCreated(ev) if ev ==
        BucketCreated {  bucket_id: test_bucket.bucket_id, owner_id: test_bucket.owner_id }));

    assert!(matches!(evs.pop().unwrap(), Event::BucketAllocated(ev) if ev ==
        BucketAllocated { bucket_id: test_bucket.bucket_id, cluster_id: ctx.cluster_id, resource: test_bucket.resource }));

    // Deposit more.
    let net_deposit = 10 * TOKEN;
    assert!(matches!(evs.pop().unwrap(), Event::Deposit(ev) if ev ==
        Deposit { account_id: test_bucket.owner_id, value: net_deposit }));

    assert_eq!(evs.len(), 0, "all events must be checked");
}


#[ink::test]
fn account_deposit_works() {
    let account_id = get_accounts().alice;
    let mut contract = DdcBucket::new();

    assert_eq!(
        contract.account_get(account_id),
        Err(AccountDoesNotExist), "must not get a non-existent account");

    let deposit = 10 * TOKEN;
    let deposit_contract_fee = calculate_contract_fee(Account::RECORD_SIZE).peek();
    let deposit_after_fee = deposit - deposit_contract_fee;

    // Deposit some value.
    push_caller_value(account_id, deposit);
    contract.account_deposit();
    pop_caller();

    let account = contract.account_get(account_id)?;
    assert_eq!(account, Account {
        deposit: Cash(deposit_after_fee),
        payable_schedule: Schedule::empty(),
    }, "must take deposit minus creation fee");

    // Deposit more value.
    push_caller_value(account_id, deposit);
    contract.account_deposit();
    pop_caller();

    let account = contract.account_get(account_id)?;
    assert_eq!(account, Account {
        deposit: Cash(deposit_after_fee + deposit),
        payable_schedule: Schedule::empty(),
    }, "must take more deposits without creation fee");

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
    push_caller_value(ctx.provider_id0, CONTRACT_FEE_LIMIT);
    ctx.contract.node_change_params(ctx.node_id0, "new params".to_string());
    pop_caller();

    // Check the changed params.
    let status = ctx.contract.node_get(ctx.node_id0)?;
    assert_eq!(status.params, "new params");
}

#[ink::test]
#[should_panic]
fn node_change_params_only_owner() {
    let ctx = &mut new_cluster();

    // Change params.
    push_caller_value(get_accounts().bob, CONTRACT_FEE_LIMIT);
    ctx.contract.node_change_params(ctx.node_id0, "new params".to_string());
    // Panic.
}


#[ink::test]
fn cluster_change_params_works() {
    let ctx = &mut new_cluster();

    // Change params.
    push_caller_value(ctx.manager, CONTRACT_FEE_LIMIT);
    ctx.contract.cluster_change_params(ctx.cluster_id, "new params".to_string());
    pop_caller();

    // Check the changed params.
    let status = ctx.contract.cluster_get(ctx.cluster_id)?;
    assert_eq!(status.params, "new params");
}

#[ink::test]
#[should_panic]
fn cluster_change_params_only_owner() {
    let ctx = &mut new_cluster();

    // Change params.
    push_caller_value(get_accounts().bob, CONTRACT_FEE_LIMIT);
    ctx.contract.cluster_change_params(ctx.cluster_id, "new params".to_string());
    // Panic.
}


#[ink::test]
fn bucket_change_params_works() {
    let ctx = &mut new_cluster();
    let test_bucket = &new_bucket(ctx);

    // Change params.
    push_caller_value(test_bucket.owner_id, CONTRACT_FEE_LIMIT);
    ctx.contract.bucket_change_params(test_bucket.bucket_id, "new params".to_string());
    pop_caller();

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
    push_caller_value(get_accounts().bob, CONTRACT_FEE_LIMIT);
    ctx.contract.bucket_change_params(test_bucket.bucket_id, "new params".to_string());
    // Panic.
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
    let bucket_status1 = ddc_bucket.bucket_get(bucket_id1).unwrap();
    pop_caller();

    push_caller_value(owner_id2, CONTRACT_FEE_LIMIT);
    let bucket_id2 = ddc_bucket.bucket_create("".to_string(), cluster_id);
    let bucket_status2 = ddc_bucket.bucket_get(bucket_id2)?;
    pop_caller();

    assert_ne!(bucket_id1, bucket_id2);
    let count = 3;

    assert_eq!(
        ddc_bucket.bucket_list(1, 100, None),
        (vec![bucket_status1.clone(), bucket_status2.clone()], count));

    assert_eq!(
        ddc_bucket.bucket_list(1, 2, None),
        (vec![bucket_status1.clone(), bucket_status2.clone()], count));

    assert_eq!(
        ddc_bucket.bucket_list(1, 1, None),
        (vec![bucket_status1.clone() /*, bucket_status2.clone()*/], count));
    assert_eq!(
        ddc_bucket.bucket_list(2, 1, None),
        (vec![/*bucket_status1.clone(),*/ bucket_status2.clone()], count));

    assert_eq!(
        ddc_bucket.bucket_list(21, 20, None),
        (vec![], count));

    // Filter by owner.
    assert_eq!(
        ddc_bucket.bucket_list(1, 100, Some(owner_id1)),
        (vec![bucket_status1.clone() /*, bucket_status2.clone()*/], count));

    assert_eq!(
        ddc_bucket.bucket_list(1, 100, Some(owner_id2)),
        (vec![/*bucket_status1.clone(),*/ bucket_status2.clone()], count));

    assert_eq!(
        ddc_bucket.bucket_list(1, 100, Some(owner_id3)),
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
    let count = 3;

    let node1 = NodeStatus {
        node_id: node_id1,
        node: Node {
            provider_id: owner_id1,
            rent_per_month,
            free_resource: capacity,
        },
        params: node_params1.to_string(),
    };

    let node2 = NodeStatus {
        node_id: node_id2,
        node: Node {
            provider_id: owner_id2,
            rent_per_month,
            free_resource: capacity,
        },
        params: node_params2.to_string(),
    };

    assert_eq!(
        ddc_bucket.node_list(1, 100, None),
        (vec![node1.clone(), node2.clone()], count));

    assert_eq!(
        ddc_bucket.node_list(1, 2, None),
        (vec![node1.clone(), node2.clone()], count));

    assert_eq!(
        ddc_bucket.node_list(1, 1, None),
        (vec![node1.clone() /*, node2.clone()*/], count));

    assert_eq!(
        ddc_bucket.node_list(2, 1, None),
        (vec![/*node1.clone(),*/ node2.clone()], count));

    assert_eq!(
        ddc_bucket.node_list(21, 20, None),
        (vec![], count));

    // Filter by owner.
    assert_eq!(
        ddc_bucket.node_list(1, 100, Some(owner_id1)),
        (vec![node1.clone() /*, node2.clone()*/], count));

    assert_eq!(
        ddc_bucket.node_list(1, 100, Some(owner_id2)),
        (vec![/*node1.clone(),*/ node2.clone()], count));

    assert_eq!(
        ddc_bucket.node_list(1, 100, Some(owner_id3)),
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
