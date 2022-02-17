#![allow(unused_variables, dead_code)]

pub use ink_env::{
    AccountId, call,
    DefaultEnvironment,
    test,
    test::{advance_block, default_accounts, DefaultAccounts, initialize_or_reset_as_default, recorded_events},
};
use scale::Decode;

use super::*;

pub const CURRENCY: Balance = 10_000_000_000;

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
}

pub fn pop_caller() {
    test::pop_execution_context();
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