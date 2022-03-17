#![allow(unused_variables, dead_code)]

pub use ink_env::{
    call,
    DefaultEnvironment,
    test,
    test::{advance_block, default_accounts, DefaultAccounts, initialize_or_reset_as_default, recorded_events},
};
use ink_lang as ink;
use scale::Decode;

use crate::ddc_bucket::*;

/// Recommended contract fee for all operations with reasonable data amounts.
pub const CONTRACT_FEE: Balance = 10 * TOKEN;

pub fn get_accounts() -> DefaultAccounts<DefaultEnvironment> {
    // The default account is "alice"
    default_accounts::<DefaultEnvironment>().unwrap()
}

pub fn push_caller(caller: AccountId) {
    push_caller_value(caller, 0);
}

pub fn push_caller_value(caller: AccountId, transferred_value: Balance) {
    let callee = ink_env::account_id::<DefaultEnvironment>().unwrap_or([0x0; 32].into());

    test::push_execution_context::<DefaultEnvironment>(
        caller,
        callee,
        1000000,
        transferred_value,                                          // transferred balance
        test::CallData::new(call::Selector::new([0x00; 4])), // dummy
    );

    transfer(caller, callee, transferred_value);
}

pub fn pop_caller() {
    test::pop_execution_context();
}

pub fn transfer(from: AccountId, to: AccountId, amount: Balance) {
    if amount == 0 { return; }
    let balance_of_from = balance_of(from);
    assert!(balance_of_from >= amount, "Insufficient balance in test account {:?}", from);
    set_balance(from, balance_of_from - amount);
    set_balance(to, balance_of(to) + amount);
}

pub fn balance_of(account: AccountId) -> Balance {
    test::get_account_balance::<DefaultEnvironment>(account).unwrap()
}

pub fn set_balance(account: AccountId, balance: Balance) {
    ink_env::test::set_account_balance::<DefaultEnvironment>(account, balance).unwrap();
}

pub fn contract_id() -> AccountId {
    ink_env::test::get_current_contract_account_id::<DefaultEnvironment>().unwrap()
}

pub fn decode_event<Event: Decode>(event: &ink_env::test::EmittedEvent) -> Event {
    <Event as Decode>::decode(&mut &event.data[..])
        .expect("encountered invalid contract event data buffer")
}

pub fn get_events<Event: Decode>(expected_count: usize) -> Vec<Event> {
    let raw_events = recorded_events().collect::<Vec<_>>();
    assert_eq!(raw_events.len(), expected_count);
    raw_events.iter().map(decode_event).collect()
}


pub type Event = <DdcBucket as ink::BaseEvent>::Type;

fn _print_events(events: &[Event]) {
    for ev in events.iter() {
        match ev {
            Event::ClusterCreated(ev) => println!("EVENT {:?}", ev),
            Event::VNodeCreated(ev) => println!("EVENT {:?}", ev),
            Event::BucketCreated(ev) => println!("EVENT {:?}", ev),
            Event::BucketAllocated(ev) => println!("EVENT {:?}", ev),
            Event::DealCreated(ev) => println!("EVENT {:?}", ev),
            Event::Deposit(ev) => println!("EVENT {:?}", ev),
            Event::ProviderWithdraw(ev) => println!("EVENT {:?}", ev),
        }
    }
}