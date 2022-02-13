use ink_lang as ink;

use super::*;
use super::test_utils::*;

type Event = <DdcBucket as ink::BaseEvent>::Type;

#[ink::test]
fn ddc_bucket_works() {
    let accounts = get_accounts();
    push_caller(accounts.alice);

    let mut ddc_bucket = DdcBucket::new();

    // Provider setup.
    let rent_per_month: Balance = 10;
    ddc_bucket.provider_set_info(rent_per_month)?;

    // Consumer setup.
    push_caller(accounts.bob);
    let bid = ddc_bucket.create_bucket(accounts.alice)?;
    push_caller_value(accounts.bob, 100);
    ddc_bucket.topup_bucket(bid)?;
    pop_caller();
    let status = ddc_bucket.get_bucket_status(bid)?;
    pop_caller();

    // Provider withdraw.
    ddc_bucket.provider_withdraw(bid)?;

    let evs = get_events(4);
    assert!(matches!(&evs[0], Event::ProviderSetInfo(ev) if ev.rent_per_month == rent_per_month));
    assert!(matches!(&evs[1], Event::CreateBucket(ev) if ev.bucket_id == 0));
    assert!(matches!(&evs[2], Event::TopupBucket(ev) if ev.value == 100));
    assert!(matches!(&evs[3], Event::ProviderWithdraw(ev) if ev.value == 0));
}
