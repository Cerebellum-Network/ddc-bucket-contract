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
    let location = "https://ddc.cere.network";
    ddc_bucket.provider_set_info(rent_per_month, location.to_string())?;

    // Consumer discovers the Provider.
    let provider = ddc_bucket.provider_get_info(provider_id)?;
    println!("GET {:?}", provider);

    // Consumer setup.
    push_caller_value(consumer_id, 100 * CURRENCY);
    let bucket_id = ddc_bucket.create_bucket(provider_id)?;
    ddc_bucket.topup_bucket(bucket_id)?;
    pop_caller();

    // Provider checks the status of the bucket.
    let status = ddc_bucket.get_bucket_status(bucket_id)?;
    println!("GET {:?}", status);

    // Provider withdraws in the future.
    advance_block::<DefaultEnvironment>().unwrap();
    ddc_bucket.provider_withdraw(bucket_id)?;

    let evs = get_events(5);
    print_events(&evs);
    assert!(matches!(&evs[0], Event::ProviderSetInfo(ev) if ev.rent_per_month == rent_per_month));
    assert!(matches!(&evs[1], Event::CreateBucket(ev) if ev.bucket_id == 0));
    assert!(matches!(&evs[2], Event::TopupBucket(ev) if ev.value == 100 * CURRENCY));
    assert!(matches!(&evs[3], Event::TopupBucket(ev) if ev.value == 100 * CURRENCY));
    assert!(matches!(&evs[4], Event::ProviderWithdraw(ev) if ev.value == 186));
}

fn print_events(events: &[Event]) {
    for ev in events.iter() {
        match ev {
            Event::ProviderSetInfo(ev) => println!("EVENT {:?}", ev),
            Event::CreateBucket(ev) => println!("EVENT {:?}", ev),
            Event::TopupBucket(ev) => println!("EVENT {:?}", ev),
            Event::ProviderWithdraw(ev) => println!("EVENT {:?}", ev),
        }
    }
}