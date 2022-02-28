use ink_lang as ink;

use super::*;
use super::test_utils::*;

type Event = <DdcBucket as ink::BaseEvent>::Type;


#[ink::test]
fn ddc_bucket_works() {
    let accounts = get_accounts();
    let provider_id0 = accounts.alice;
    let provider_id1 = accounts.bob;
    let consumer_id = accounts.charlie;

    let mut ddc_bucket = DdcBucket::new();
    set_balance(contract_id(), 1000); // For contract subsistence.

    // Create a Cluster.
    let cluster_params = "{}";
    push_caller(provider_id0);
    let cluster_id = ddc_bucket.cluster_create(cluster_params.to_string())?;
    pop_caller();

    // Provide a Service.
    let rent_per_month: Balance = 10 * CURRENCY;
    let service_params0 = "{\"url\":\"https://ddc.cere.network/bucket/{BUCKET_ID}\"}";
    let service_id0 = ddc_bucket.service_create(cluster_id, rent_per_month, service_params0.to_string())?;

    // Provide another Service.
    push_caller(provider_id1);
    let service_params1 = "{\"url\":\"https://ddc-2.cere.network/bucket/{BUCKET_ID}\"}";
    let service_id1 = ddc_bucket.service_create(cluster_id, rent_per_month, service_params1.to_string())?;
    pop_caller();
    assert_ne!(service_id0, service_id1);

    // Consumer discovers the Cluster with the 2 Services.
    let cluster = ddc_bucket.cluster_get(cluster_id)?;
    assert_eq!(cluster, Cluster {
        cluster_id,
        cluster_params: cluster_params.to_string(),
        service_ids: vec![service_id0, service_id1],
    });
    let service0 = ddc_bucket.service_get(service_id0)?;
    assert_eq!(service0, Service {
        service_id: service_id0,
        provider_id: provider_id0,
        rent_per_month,
        service_params: service_params0.to_string(),
    });
    let service1 = ddc_bucket.service_get(service_id1)?;
    assert_eq!(service1, Service {
        service_id: service_id1,
        provider_id: provider_id1,
        rent_per_month,
        service_params: service_params1.to_string(),
    });

    // Create a bucket.
    push_caller_value(consumer_id, 0);
    let bucket_params = "{}".to_string();
    let bucket_id = ddc_bucket.bucket_create(bucket_params.clone())?;
    pop_caller();

    // Add a deal to the bucket, also depositing some value.
    push_caller_value(consumer_id, 10 * CURRENCY);
    ddc_bucket.bucket_add_cluster(bucket_id, cluster_id)?;
    pop_caller();

    // Check the structure of the bucket including all deal IDs.
    let bucket = ddc_bucket.bucket_get(bucket_id)?;
    let deal_id0 = 0;
    let deal_id1 = 1;
    let deal_params = "".to_string();
    assert_eq!(bucket, Bucket {
        owner_id: consumer_id,
        cluster_ids: vec![cluster_id],
        deal_ids: vec![deal_id0, deal_id1],
        bucket_params: bucket_params.to_string(),
    });

    // Provider checks the status of the deal.
    let deal_status0 = ddc_bucket.deal_get_status(deal_id0)?;
    assert_eq!(deal_status0, DealStatus {
        service_id: service_id0,
        estimated_rent_end_ms: 1339200000, // TODO: calculate this value.
        deal_params: deal_params.clone(),
    });

    // Deposit more value into the account.
    push_caller_value(consumer_id, 100 * CURRENCY);
    ddc_bucket.deposit()?;
    pop_caller();

    // The end time increased because there is more deposit.
    let deal_status0 = ddc_bucket.deal_get_status(deal_id0)?;
    assert_eq!(deal_status0, DealStatus {
        service_id: service_id0,
        estimated_rent_end_ms: 14731200000, // TODO: calculate this value.
        deal_params: deal_params.clone(),
    });

    // The end time of the second deal is the same because it is paid from the same account.
    let deal_status1 = ddc_bucket.deal_get_status(deal_id1)?;
    assert_eq!(deal_status1, DealStatus {
        service_id: service_id1,
        estimated_rent_end_ms: 14731200000, // TODO: calculate this value.
        deal_params: deal_params.clone(),
    });

    // Check the status of the bucket recursively including all deal statuses.
    let bucket_status = ddc_bucket.bucket_get_status(bucket_id)?;
    assert_eq!(bucket_status, BucketStatus {
        bucket_id,
        bucket,
        writer_ids: vec![consumer_id],
        deal_statuses: vec![deal_status0, deal_status1],
    });

    // Provider withdraws in the future.
    advance_block::<DefaultEnvironment>().unwrap();
    push_caller(provider_id0);
    ddc_bucket.provider_withdraw(deal_id0)?;
    pop_caller();

    push_caller(provider_id1);
    ddc_bucket.provider_withdraw(deal_id1)?;
    pop_caller();

    let mut evs = get_events(10);
    evs.reverse();

    // Cluster setup.
    assert!(matches!(evs.pop().unwrap(), Event::ClusterCreated(ev) if ev ==
        ClusterCreated { cluster_id, cluster_params: cluster_params.to_string() }));

    // Provider setup.
    assert!(matches!(evs.pop().unwrap(), Event::ServiceCreated(ev) if ev ==
        ServiceCreated { service_id: service_id0, provider_id: provider_id0, rent_per_month, service_params: service_params0.to_string() }));

    // Provider setup 2.
    assert!(matches!(evs.pop().unwrap(), Event::ServiceCreated(ev) if ev ==
        ServiceCreated { service_id: service_id1, provider_id: provider_id1, rent_per_month, service_params: service_params1.to_string() }));

    // Create bucket.
    assert!(matches!(evs.pop().unwrap(), Event::BucketCreated(ev) if ev ==
        BucketCreated {  bucket_id, owner_id: consumer_id }));

    // Add a cluster with 2 deals and an initial deposit.
    assert!(matches!(evs.pop().unwrap(), Event::Deposit(ev) if ev ==
        Deposit { account_id: consumer_id, value: 10 * CURRENCY }));
    assert!(matches!(evs.pop().unwrap(), Event::DealCreated(ev) if ev ==
        DealCreated { deal_id: deal_id0, bucket_id, service_id: service_id0 }));
    assert!(matches!(evs.pop().unwrap(), Event::DealCreated(ev) if ev ==
        DealCreated { deal_id: deal_id1, bucket_id, service_id: service_id1 }));

    // Deposit more.
    assert!(matches!(evs.pop().unwrap(), Event::Deposit(ev) if ev ==
        Deposit { account_id: consumer_id, value: 100 * CURRENCY }));

    // Provider withdrawals.
    assert!(matches!(evs.pop().unwrap(), Event::ProviderWithdraw(ev) if ev ==
        ProviderWithdraw { provider_id: provider_id0, deal_id: deal_id0, value: 186 }));
    assert!(matches!(evs.pop().unwrap(), Event::ProviderWithdraw(ev) if ev ==
        ProviderWithdraw { provider_id: provider_id1, deal_id: deal_id1, value: 186 }));
}


#[ink::test]
fn bucket_list_works() {
    let accounts = get_accounts();
    let owner_id1 = accounts.alice;
    let owner_id2 = accounts.bob;
    let owner_id3 = accounts.charlie;

    let mut ddc_bucket = DdcBucket::new();

    push_caller(owner_id1);
    let bucket_id1 = ddc_bucket.bucket_create("".to_string())?;
    let bucket_status1 = ddc_bucket.bucket_get_status(bucket_id1)?;
    pop_caller();

    push_caller(owner_id2);
    let bucket_id2 = ddc_bucket.bucket_create("".to_string())?;
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
fn service_list_works() {
    let accounts = get_accounts();
    let owner_id1 = accounts.alice;
    let owner_id2 = accounts.bob;
    let owner_id3 = accounts.charlie;
    let rent_per_month: Balance = 10 * CURRENCY;

    let mut ddc_bucket = DdcBucket::new();

    // Create a Cluster.
    push_caller(owner_id1);
    let cluster_params = "{}";
    let cluster_id = ddc_bucket.cluster_create(cluster_params.to_string())?;
    pop_caller();

    // Create two Services.
    push_caller(owner_id1);
    let service_params1 = "{\"url\":\"https://ddc-1.cere.network/bucket/{BUCKET_ID}\"}";
    let service_id1 = ddc_bucket.service_create(cluster_id, rent_per_month, service_params1.to_string())?;
    pop_caller();

    push_caller(owner_id2);
    let service_params2 = "{\"url\":\"https://ddc-2.cere.network/bucket/{BUCKET_ID}\"}";
    let service_id2 = ddc_bucket.service_create(cluster_id, rent_per_month, service_params2.to_string())?;
    pop_caller();

    assert_ne!(service_id1, service_id2);
    let count = 2;

    let service1 = Service {
        service_id: service_id1,
        provider_id: owner_id1,
        rent_per_month,
        service_params: service_params1.to_string(),
    };

    let service2 = Service {
        service_id: service_id2,
        provider_id: owner_id2,
        rent_per_month,
        service_params: service_params2.to_string(),
    };

    assert_eq!(
        ddc_bucket.service_list(0, 100, None),
        (vec![service1.clone(), service2.clone()], count));

    assert_eq!(
        ddc_bucket.service_list(0, 2, None),
        (vec![service1.clone(), service2.clone()], count));

    assert_eq!(
        ddc_bucket.service_list(0, 1, None),
        (vec![service1.clone() /*, service2.clone()*/], count));

    assert_eq!(
        ddc_bucket.service_list(1, 1, None),
        (vec![/*service1.clone(),*/ service2.clone()], count));

    assert_eq!(
        ddc_bucket.service_list(20, 20, None),
        (vec![], count));

    // Filter by owner.
    assert_eq!(
        ddc_bucket.service_list(0, 100, Some(owner_id1)),
        (vec![service1.clone() /*, service2.clone()*/], count));

    assert_eq!(
        ddc_bucket.service_list(0, 100, Some(owner_id2)),
        (vec![/*service1.clone(),*/ service2.clone()], count));

    assert_eq!(
        ddc_bucket.service_list(0, 100, Some(owner_id3)),
        (vec![], count));
}


fn _print_events(events: &[Event]) {
    for ev in events.iter() {
        match ev {
            Event::ClusterCreated(ev) => println!("EVENT {:?}", ev),
            Event::ServiceCreated(ev) => println!("EVENT {:?}", ev),
            Event::BucketCreated(ev) => println!("EVENT {:?}", ev),
            Event::DealCreated(ev) => println!("EVENT {:?}", ev),
            Event::Deposit(ev) => println!("EVENT {:?}", ev),
            Event::ProviderWithdraw(ev) => println!("EVENT {:?}", ev),
        }
    }
}