use ink_lang as ink;

use super::*;
use super::test_utils::*;

type Event = <DdcBucket as ink::BaseEvent>::Type;


#[ink::test]
fn ddc_bucket_works() {
    let accounts = get_accounts();
    let provider_id = accounts.alice;
    let provider_id2 = accounts.bob;
    let consumer_id = accounts.charlie;

    push_caller(provider_id);
    let mut ddc_bucket = DdcBucket::new();
    set_balance(contract_id(), 1000); // For contract subsistence.

    // Provide a Service.
    let rent_per_month: Balance = 10 * CURRENCY;
    let description = "{\"url\":\"https://ddc.cere.network/bucket/{BUCKET_ID}\"}";
    let service_id = ddc_bucket.service_create(rent_per_month, description.to_string())?;

    // Provide another Service.
    push_caller(provider_id2);
    let description2 = "{\"url\":\"https://ddc-2.cere.network/bucket/{BUCKET_ID}\"}";
    let service_id2 = ddc_bucket.service_create(rent_per_month, description2.to_string())?;
    pop_caller();
    assert_ne!(service_id, service_id2);

    // Consumer discovers the Provider.
    let service = ddc_bucket.service_get(service_id)?;
    assert_eq!(service, Service {
        service_id,
        provider_id,
        rent_per_month,
        description: description.to_string(),
    });

    // Create a bucket.
    push_caller_value(consumer_id, 0);
    let bucket_id = ddc_bucket.bucket_create()?;
    pop_caller();

    // Add a deal to the bucket, also depositing some value.
    push_caller_value(consumer_id, 10 * CURRENCY);
    let deal_id1 = ddc_bucket.bucket_add_deal(bucket_id, service_id)?;
    pop_caller();

    // Deposit more value into the account.
    push_caller_value(consumer_id, 100 * CURRENCY);
    ddc_bucket.deposit()?;
    pop_caller();

    // Provider checks the status of the deal.
    let deal_status1 = ddc_bucket.deal_get_status(deal_id1)?;
    assert_eq!(deal_status1, DealStatus {
        service_id,
        estimated_rent_end_ms: 29462400000,
    });

    // Add another deal, making the consumer pay a more expensive rate.
    push_caller_value(consumer_id, 10 * CURRENCY);
    let deal_id2 = ddc_bucket.bucket_add_deal(bucket_id, service_id)?;
    assert_ne!(deal_id1, deal_id2);
    pop_caller();

    // The end time of the first deal is earlier because the deposit is being depleted faster.
    let deal_status1 = ddc_bucket.deal_get_status(deal_id1)?;
    assert_eq!(deal_status1, DealStatus {
        service_id,
        estimated_rent_end_ms: 16070400000, // TODO: this value looks wrong.
    });

    // The end time of the second deal is the same because it is paid from the same account.
    let deal_status2 = ddc_bucket.deal_get_status(deal_id2)?;
    assert_eq!(deal_status2, DealStatus {
        service_id,
        estimated_rent_end_ms: 16070400000, // TODO: this value looks wrong.
    });

    // Check the structure of the bucket including all deal IDs.
    let bucket = ddc_bucket.bucket_get(bucket_id)?;
    assert_eq!(bucket, Bucket {
        owner_id: consumer_id,
        deal_ids: vec![deal_id1, deal_id2],
    });

    // Check the status of the bucket recursively including all deal statuses.
    let bucket_status = ddc_bucket.bucket_get_status(bucket_id)?;
    assert_eq!(bucket_status, BucketStatus {
        bucket_id,
        bucket,
        writer_ids: vec![consumer_id],
        deal_statuses: vec![deal_status1, deal_status2],
    });


    // Provider withdraws in the future.
    advance_block::<DefaultEnvironment>().unwrap();
    ddc_bucket.provider_withdraw(deal_id1)?;
    ddc_bucket.provider_withdraw(deal_id2)?;

    let evs = get_events(10);
    // Provider setup.
    assert!(matches!(&evs[0], Event::ServiceCreated(ev) if *ev ==
        ServiceCreated { service_id, provider_id, rent_per_month, description: description.to_string() }));

    // Provider setup 2.
    assert!(matches!(&evs[1], Event::ServiceCreated(ev) if *ev ==
        ServiceCreated { service_id: service_id2, provider_id: provider_id2, rent_per_month, description: description2.to_string() }));

    // Create bucket.
    assert!(matches!(&evs[2], Event::BucketCreated(ev) if *ev ==
        BucketCreated {  bucket_id, owner_id: consumer_id }));

    // Add deal 1 with an initial deposit.
    assert!(matches!(&evs[3], Event::Deposit(ev) if *ev ==
        Deposit { account_id: consumer_id, value: 10 * CURRENCY }));
    assert!(matches!(&evs[4], Event::DealCreated(ev) if *ev ==
        DealCreated { deal_id: deal_id1, bucket_id, service_id }));

    // Deposit more.
    assert!(matches!(&evs[5], Event::Deposit(ev) if *ev ==
        Deposit { account_id: consumer_id, value: 100 * CURRENCY }));

    // Add deal 2 with an additional deposit.
    assert!(matches!(&evs[6], Event::Deposit(ev) if *ev ==
        Deposit { account_id: consumer_id, value: 10 * CURRENCY }));
    assert!(matches!(&evs[7], Event::DealCreated(ev) if *ev ==
        DealCreated { deal_id: deal_id2, bucket_id, service_id }));

    // Provider withdrawals.
    assert!(matches!(&evs[8], Event::ProviderWithdraw(ev) if *ev ==
        ProviderWithdraw { provider_id, deal_id: deal_id1, value: 186 }));
    assert!(matches!(&evs[9], Event::ProviderWithdraw(ev) if *ev ==
        ProviderWithdraw { provider_id, deal_id: deal_id2, value: 186 }));
}


#[ink::test]
fn bucket_list_works() {
    let accounts = get_accounts();
    let owner_id1 = accounts.alice;
    let owner_id2 = accounts.bob;

    let mut ddc_bucket = DdcBucket::new();

    push_caller(owner_id1);
    let bucket_id1 = ddc_bucket.bucket_create()?;
    let bucket_status1 = ddc_bucket.bucket_get_status(bucket_id1)?;
    pop_caller();

    push_caller(owner_id2);
    let bucket_id2 = ddc_bucket.bucket_create()?;
    let bucket_status2 = ddc_bucket.bucket_get_status(bucket_id2)?;
    pop_caller();

    assert_ne!(bucket_id1, bucket_id2);
    let count = 2;

    assert_eq!(
        ddc_bucket.bucket_list_statuses(0, 100),
        (vec![bucket_status1.clone(), bucket_status2.clone()], count));

    assert_eq!(
        ddc_bucket.bucket_list_statuses(0, 2),
        (vec![bucket_status1.clone(), bucket_status2.clone()], count));

    assert_eq!(
        ddc_bucket.bucket_list_statuses(0, 1),
        (vec![bucket_status1.clone() /*, bucket_status2.clone()*/], count));

    assert_eq!(
        ddc_bucket.bucket_list_statuses(1, 1),
        (vec![/*bucket_status1.clone(),*/ bucket_status2.clone()], count));

    assert_eq!(
        ddc_bucket.bucket_list_statuses(20, 20),
        (vec![], count));
}


fn _print_events(events: &[Event]) {
    for ev in events.iter() {
        match ev {
            Event::ServiceCreated(ev) => println!("EVENT {:?}", ev),
            Event::BucketCreated(ev) => println!("EVENT {:?}", ev),
            Event::DealCreated(ev) => println!("EVENT {:?}", ev),
            Event::Deposit(ev) => println!("EVENT {:?}", ev),
            Event::ProviderWithdraw(ev) => println!("EVENT {:?}", ev),
        }
    }
}