use ink_lang as ink;

use super::*;
use super::test_utils::*;

type Event = <DdcBucket as ink::BaseEvent>::Type;


#[ink::test]
fn ddc_bucket_works() {
    let accounts = get_accounts();
    let provider_id = accounts.alice;
    let consumer_id = accounts.bob;
    push_caller(provider_id);

    let mut ddc_bucket = DdcBucket::new();
    set_balance(contract_id(), 1000); // For contract subsistence.

    // Provider setup.
    let service_id = provider_id;
    let rent_per_month: Balance = 10 * CURRENCY;
    let location = "https://ddc.cere.network/bucket/{BUCKET_ID}";
    ddc_bucket.service_set_info(service_id, rent_per_month, location.to_string())?;

    // Consumer discovers the Provider.
    let service = ddc_bucket.service_get_info(service_id)?;
    assert_eq!(service, Service {
        rent_per_month,
        location: location.to_string(),
    });

    // Create a bucket, including some value.
    push_caller_value(consumer_id, 10 * CURRENCY);
    let bucket_id = ddc_bucket.bucket_create(service_id)?;
    pop_caller();

    // Add more value into the bucket.
    push_caller_value(consumer_id, 100 * CURRENCY);
    ddc_bucket.deposit()?;
    pop_caller();

    // Provider checks the status of the bucket.
    let status = ddc_bucket.bucket_get_status(bucket_id)?;
    assert_eq!(status, BucketStatus {
        provider_id,
        estimated_rent_end_ms: 29462400000,
        writer_ids: vec![consumer_id],
    });

    // Create another bucket, making the consumer pay a more expensive rate.
    push_caller_value(consumer_id, 10 * CURRENCY);
    let bucket_id2 = ddc_bucket.bucket_create(provider_id)?;
    assert_ne!(bucket_id, bucket_id2);
    pop_caller();

    // The end time of the first bucket is earlier because the deposit is being depleted faster.
    let status1 = ddc_bucket.bucket_get_status(bucket_id)?;
    assert_eq!(status1, BucketStatus {
        provider_id,
        estimated_rent_end_ms: 16070400000, // TODO: this value looks wrong.
        writer_ids: vec![consumer_id],
    });

    // The end time of the second bucket is the same because it is paid from the same account.
    let status2 = ddc_bucket.bucket_get_status(bucket_id2)?;
    assert_eq!(status2, BucketStatus {
        provider_id,
        estimated_rent_end_ms: 16070400000, // TODO: this value looks wrong.
        writer_ids: vec![consumer_id],
    });

    // Provider withdraws in the future.
    advance_block::<DefaultEnvironment>().unwrap();
    ddc_bucket.provider_withdraw(bucket_id)?;

    let evs = get_events(7);
    // Provider setup.
    assert!(matches!(&evs[0], Event::ServiceSetInfo(ev) if *ev ==
        ServiceSetInfo { provider_id, service_id, rent_per_month, location: location.to_string() }));

    // Create bucket 1 with an initial deposit.
    assert!(matches!(&evs[1], Event::Deposit(ev) if *ev ==
        Deposit { account_id: consumer_id, value: 10 * CURRENCY }));
    assert!(matches!(&evs[2], Event::BucketCreated(ev) if *ev ==
        BucketCreated { bucket_id }));

    // Deposit more.
    assert!(matches!(&evs[3], Event::Deposit(ev) if *ev ==
        Deposit { account_id: consumer_id, value: 100 * CURRENCY }));

    // Create bucket 2 with an additional deposit.
    assert!(matches!(&evs[4], Event::Deposit(ev) if *ev ==
        Deposit { account_id: consumer_id, value: 10 * CURRENCY }));
    assert!(matches!(&evs[5], Event::BucketCreated(ev) if *ev ==
        BucketCreated { bucket_id: bucket_id2 }));

    // Provider withdrawaw.
    assert!(matches!(&evs[6], Event::ProviderWithdraw(ev) if *ev ==
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