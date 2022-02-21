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
    let rent_per_month: Balance = 10 * CURRENCY;
    let location = "https://ddc.cere.network/bucket/{BUCKET_ID}";
    ddc_bucket.provider_set_info(rent_per_month, location.to_string())?;

    // Consumer setup.
    let bucket_id = {
        // Consumer discovers the Provider.
        let provider = ddc_bucket.provider_get_info(provider_id)?;
        assert_eq!(provider, Provider {
            rent_per_month,
            location: location.to_string(),
        });

        // Create a bucket, including some value.
        push_caller_value(consumer_id, 10 * CURRENCY);
        let bucket_id = ddc_bucket.bucket_create(provider_id)?;
        pop_caller();

        // Add more value into the bucket.
        push_caller_value(consumer_id, 100 * CURRENCY);
        ddc_bucket.bucket_topup(bucket_id)?;
        pop_caller();

        bucket_id
    };

    // Provider checks the status of the bucket.
    let status = ddc_bucket.bucket_get_status(bucket_id)?;
    assert_eq!(status, BucketStatus {
        provider_id,
        estimated_rent_end_ms: 29462400000,
        writer_ids: vec![consumer_id],
    });

    // Provider withdraws in the future.
    advance_block::<DefaultEnvironment>().unwrap();
    ddc_bucket.provider_withdraw(bucket_id)?;

    let evs = get_events(5);
    assert!(matches!(&evs[0], Event::ProviderSetInfo(ev) if *ev ==
        ProviderSetInfo { provider_id, rent_per_month, location: location.to_string() }));
    assert!(matches!(&evs[1], Event::BucketCreated(ev) if *ev ==
        BucketCreated { bucket_id }));
    assert!(matches!(&evs[2], Event::BucketTopup(ev) if *ev ==
        BucketTopup { bucket_id, value: 10 * CURRENCY }));
    assert!(matches!(&evs[3], Event::BucketTopup(ev) if *ev ==
        BucketTopup { bucket_id, value: 100 * CURRENCY }));
    assert!(matches!(&evs[4], Event::ProviderWithdraw(ev) if *ev ==
        ProviderWithdraw { provider_id, bucket_id, value: 186 }));
}

fn _print_events(events: &[Event]) {
    for ev in events.iter() {
        match ev {
            Event::ProviderSetInfo(ev) => println!("EVENT {:?}", ev),
            Event::BucketCreated(ev) => println!("EVENT {:?}", ev),
            Event::BucketTopup(ev) => println!("EVENT {:?}", ev),
            Event::ProviderWithdraw(ev) => println!("EVENT {:?}", ev),
        }
    }
}