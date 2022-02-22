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
    let service_id = (provider_id, 0);
    let rent_per_month: Balance = 10 * CURRENCY;
    let description = "{\"url\":\"https://ddc.cere.network/bucket/{BUCKET_ID}\"}";
    ddc_bucket.service_set_info(service_id, rent_per_month, description.to_string())?;

    // Provide another Service.
    push_caller(provider_id2);
    let service_id2 = (provider_id2, 1);
    let description2 = "{\"url\":\"https://ddc-2.cere.network/bucket/{BUCKET_ID}\"}";
    ddc_bucket.service_set_info(service_id2, rent_per_month, description2.to_string())?;
    pop_caller();

    // Consumer discovers the Provider.
    let service = ddc_bucket.service_get_info(service_id)?;
    assert_eq!(service, Service {
        provider_id,
        rent_per_month,
        description: description.to_string(),
    });

    // Create a replicated-bucket.
    push_caller_value(consumer_id, 0);
    let repbuck_id = ddc_bucket.repbuck_create()?;
    pop_caller();

    // Create a bucket, also depositing some value.
    push_caller_value(consumer_id, 10 * CURRENCY);
    let bucket_id = ddc_bucket.repbuck_attach_service(repbuck_id, service_id)?;
    pop_caller();

    // Deposit more value into the account.
    push_caller_value(consumer_id, 100 * CURRENCY);
    ddc_bucket.deposit()?;
    pop_caller();

    // Provider checks the status of the bucket.
    let status = ddc_bucket.bucket_get_status(bucket_id)?;
    assert_eq!(status, BucketStatus {
        service_id,
        estimated_rent_end_ms: 29462400000,
        writer_ids: vec![consumer_id],
    });

    // Create another bucket, making the consumer pay a more expensive rate.
    push_caller_value(consumer_id, 10 * CURRENCY);
    let bucket_id2 = ddc_bucket.repbuck_attach_service(repbuck_id, service_id)?;
    assert_ne!(bucket_id, bucket_id2);
    pop_caller();

    // The end time of the first bucket is earlier because the deposit is being depleted faster.
    let status1 = ddc_bucket.bucket_get_status(bucket_id)?;
    assert_eq!(status1, BucketStatus {
        service_id,
        estimated_rent_end_ms: 16070400000, // TODO: this value looks wrong.
        writer_ids: vec![consumer_id],
    });

    // The end time of the second bucket is the same because it is paid from the same account.
    let status2 = ddc_bucket.bucket_get_status(bucket_id2)?;
    assert_eq!(status2, BucketStatus {
        service_id,
        estimated_rent_end_ms: 16070400000, // TODO: this value looks wrong.
        writer_ids: vec![consumer_id],
    });

    // Check the status of the replicated-bucket to discover the two sub-buckets.
    let repbuck = ddc_bucket.repbuck_get(repbuck_id)?;
    assert_eq!(repbuck, RepBuck {
        owner_id: consumer_id,
        bucket_ids: vec![bucket_id, bucket_id2],
    });

    // Provider withdraws in the future.
    advance_block::<DefaultEnvironment>().unwrap();
    ddc_bucket.provider_withdraw(bucket_id)?;

    let evs = get_events(8);
    // Provider setup.
    assert!(matches!(&evs[0], Event::ServiceSetInfo(ev) if *ev ==
        ServiceSetInfo { provider_id, service_id, rent_per_month, description: description.to_string() }));

    // Provider setup 2.
    assert!(matches!(&evs[1], Event::ServiceSetInfo(ev) if *ev ==
        ServiceSetInfo { provider_id:provider_id2, service_id:service_id2, rent_per_month, description: description2.to_string() }));

    // Create bucket 1 with an initial deposit.
    assert!(matches!(&evs[2], Event::Deposit(ev) if *ev ==
        Deposit { account_id: consumer_id, value: 10 * CURRENCY }));
    assert!(matches!(&evs[3], Event::BucketCreated(ev) if *ev ==
        BucketCreated { bucket_id, service_id }));

    // Deposit more.
    assert!(matches!(&evs[4], Event::Deposit(ev) if *ev ==
        Deposit { account_id: consumer_id, value: 100 * CURRENCY }));

    // Create bucket 2 with an additional deposit.
    assert!(matches!(&evs[5], Event::Deposit(ev) if *ev ==
        Deposit { account_id: consumer_id, value: 10 * CURRENCY }));
    assert!(matches!(&evs[6], Event::BucketCreated(ev) if *ev ==
        BucketCreated { bucket_id: bucket_id2, service_id }));

    // Provider withdrawaw.
    assert!(matches!(&evs[7], Event::ProviderWithdraw(ev) if *ev ==
        ProviderWithdraw { provider_id, bucket_id, value: 186 }));
}

fn _print_events(events: &[Event]) {
    for ev in events.iter() {
        match ev {
            Event::ServiceSetInfo(ev) => println!("EVENT {:?}", ev),
            Event::BucketCreated(ev) => println!("EVENT {:?}", ev),
            Event::Deposit(ev) => println!("EVENT {:?}", ev),
            Event::ProviderWithdraw(ev) => println!("EVENT {:?}", ev),
        }
    }
}