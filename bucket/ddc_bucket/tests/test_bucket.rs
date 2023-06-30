use ink_lang as ink;

use crate::ddc_bucket::*;
use crate::ddc_bucket::flow::Flow;
use crate::ddc_bucket::schedule::{Schedule, MS_PER_MONTH};
use super::env_utils::*;
use super::setup_utils::*;


fn do_bucket_pays_cluster(
    ctx: &mut TestCluster,
    test_bucket: &TestBucket,
    usd_per_cere: Balance,
) -> Result<()> {
    let expected_rent = ctx.rent_per_month * ctx.cluster_v_nodes.len() as Balance;

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
    // Go to the future when some revenues are due.
    advance_block::<DefaultEnvironment>();
    // Pay the due thus far.
    set_caller_value(ctx.manager_id, CONTRACT_FEE_LIMIT);
    ctx.contract.bucket_settle_payment(test_bucket.bucket_id)?;
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
fn bucket_pays_cluster_ok() {
    let ctx = &mut setup_cluster();
    let test_bucket = &setup_bucket(ctx);
    do_bucket_pays_cluster(ctx, test_bucket, 1).unwrap();
}


#[ink::test]
fn bucket_pays_cluster_at_new_rate_ok() {
    let ctx = &mut setup_cluster();

    let test_bucket = &setup_bucket(ctx);
    // Set up an exchange rate manager_id.
    set_caller(admin_id());
    ctx.contract
        .admin_grant_permission(admin_id(), Permission::SetExchangeRate).unwrap();


    // Change the currency exchange rate.
    let usd_per_cere = 2;
    set_caller(admin_id());
    ctx.contract.account_set_usd_per_cere(usd_per_cere * TOKEN)?;

    do_bucket_pays_cluster(ctx, test_bucket, usd_per_cere).unwrap();
}


#[ink::test]
fn bucket_create_ok() {
    let ctx = &mut setup_cluster();
    let test_bucket = &setup_bucket(ctx);

    // Check the structure of the bucket including the payment flow.
    let total_rent = ctx.rent_per_month * ctx.cluster_v_nodes.len() as Balance;
    let bucket_params = BucketParams::from("");
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
        bucket_params: bucket_params
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
    events.truncate(8 - 3 - 2); // Skip 3 NodeCreated and 2 cluster setup_contract events.

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
fn bucket_change_params_ok() {
    let ctx = &mut setup_cluster();
    let test_bucket = &setup_bucket(ctx);

    // Change params.
    set_caller_value(test_bucket.owner_id, CONTRACT_FEE_LIMIT);
    ctx.contract
        .bucket_change_params(test_bucket.bucket_id, "new params".to_string())?;

    // Check the changed params.
    let status = ctx.contract.bucket_get(test_bucket.bucket_id)?;
    assert_eq!(status.params, "new params");
}

#[ink::test]
#[should_panic]
fn bucket_change_params_only_owner() {
    let ctx = &mut setup_cluster();
    let test_bucket = &setup_bucket(ctx);

    // Change params.
    set_caller_value(get_accounts().bob, CONTRACT_FEE_LIMIT);
    ctx.contract
        .bucket_change_params(test_bucket.bucket_id, "new params".to_string())?;
    // Panic.
}


#[ink::test]
fn bucket_list_ok() {
    let mut ddc_bucket = setup_contract();

    let owner_id1 = AccountId::from([0xd8, 0x69, 0x19, 0x54, 0xea, 0xdc, 0x9a, 0xc0, 0x3d, 0x37, 0x56, 0x9f, 0x2a, 0xe8, 0xdf, 0x59, 0x34, 0x3f, 0x32, 0x65, 0xba, 0xd4, 0x16, 0xac, 0x07, 0xdf, 0x06, 0xeb, 0x4d, 0xbc, 0x6a, 0x66]);
    set_balance(owner_id1, 1000 * TOKEN);
    let owner_id2 = AccountId::from([0x2a, 0x5f, 0xbc, 0xcf, 0x71, 0x0b, 0x65, 0x04, 0x88, 0x91, 0x12, 0x7e, 0x5e, 0xe3, 0x78, 0xdb, 0x48, 0x63, 0x09, 0x44, 0xcc, 0xc5, 0x75, 0xbd, 0xa5, 0xaa, 0xa5, 0x0e, 0x77, 0xab, 0x7b, 0x4e]);
    set_balance(owner_id2, 1000 * TOKEN);
    let owner_id3 = AccountId::from([0x64, 0xef, 0xd7, 0xb4, 0x41, 0xb2, 0x58, 0xb5, 0x56, 0x6b, 0xfc, 0x4b, 0x19, 0xb8, 0xe5, 0x09, 0x5d, 0x17, 0xb3, 0xc3, 0x44, 0x38, 0x58, 0xa9, 0x7d, 0x20, 0x49, 0x39, 0xbd, 0xbd, 0xb6, 0x48]);
    set_balance(owner_id3, 1000 * TOKEN);

    let cluster_id = 0;

    set_caller_value(owner_id1, CONTRACT_FEE_LIMIT);
    let bucket_id1 = ddc_bucket.bucket_create("".to_string(), cluster_id, None)?;
    let bucket_status1 = ddc_bucket.bucket_get(bucket_id1)?;

    set_caller_value(owner_id2, CONTRACT_FEE_LIMIT);
    let bucket_id2 = ddc_bucket.bucket_create("".to_string(), cluster_id, None)?;
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
