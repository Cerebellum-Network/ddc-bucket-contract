use ink_lang as ink;

use crate::ddc_bucket::*;
use crate::ddc_bucket::Error::*;
use super::env_utils::*;
use super::setup_utils::*;
use crate::ddc_bucket::schedule::{Schedule};


#[ink::test]
fn account_deposit_success() {
    let account_id = AccountId::from([0x76, 0x95, 0x7c, 0xa6, 0xbe, 0xf5, 0xa3, 0x6d, 0x67, 0x0d, 0x3a, 0x84, 0xc6, 0x0a, 0xe2, 0xbb, 0xc9, 0x5e, 0xee, 0xde, 0x3a, 0x5f, 0x27, 0x0e, 0x26, 0xe3, 0x43, 0x4c, 0x46, 0xe2, 0x98, 0x10]);
    set_balance(account_id, 1000 * TOKEN);

    let mut contract = setup_contract();

    assert_eq!(
        contract.account_get(account_id),
        Err(AccountDoesNotExist),
        "must not get a non-existent account"
    );

    let deposit = 10 * TOKEN;
    let deposit_after_fee = deposit;

    // Deposit some value.
    set_caller_value(account_id, deposit);
    contract.account_deposit();

    let account = contract.account_get(account_id)?;
    assert_eq!(
        account,
        Account {
            deposit: Cash(deposit_after_fee),
            payable_schedule: Schedule::empty(),
            bonded: Cash(0),
            unbonded_amount: Cash(0),
            negative: Cash(0),
            unbonded_timestamp: 0,
        },
        "must take deposit minus creation fee"
    );

    // Deposit more value.
    set_caller_value(account_id, deposit);
    contract.account_deposit();

    let account = contract.account_get(account_id)?;
    assert_eq!(
        account,
        Account {
            deposit: Cash(deposit_after_fee + deposit),
            payable_schedule: Schedule::empty(),
            bonded: Cash(0),
            unbonded_amount: Cash(0),
            negative: Cash(0),
            unbonded_timestamp: 0,
        },
        "must take more deposits without creation fee"
    );

    // Check events.
    let mut events = get_events();
    events.reverse(); // Work with pop().

    // First deposit event.
    assert!(matches!(events.pop().unwrap(), Event::Deposit(ev) if ev ==
        Deposit { account_id, value: deposit_after_fee }));

    // Second deposit event. No deposit_contract_fee because the account already exists.
    assert!(matches!(events.pop().unwrap(), Event::Deposit(ev) if ev ==
        Deposit { account_id, value: deposit }));

    assert_eq!(events.len(), 0, "all events must be checked");
}
