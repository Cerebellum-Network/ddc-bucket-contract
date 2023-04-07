#![allow(unused_variables, dead_code)]

pub use ink::env::{
    call,
    DefaultEnvironment,
    test,
    test::{advance_block, default_accounts, DefaultAccounts, recorded_events},
};
use scale::Decode;

use crate::ddc_nft_registry::*;

pub type Event = <DdcNftRegistry as ::ink::reflect::ContractEventBase>::Type;

/// Recommended contract fee for all operations with reasonable data amounts.
pub const CONTRACT_FEE_LIMIT: Balance = 10 * TOKEN;

pub fn get_accounts() -> DefaultAccounts<DefaultEnvironment> {
  test::default_accounts::<DefaultEnvironment>()
}

pub fn set_caller_value(account: AccountId, value: Balance) {
  set_caller(account);
  set_value(value);
}

pub fn set_value(value: Balance) {
  test::set_value_transferred::<DefaultEnvironment>(value);
}

pub fn set_callee(account: AccountId) {
  test::set_callee::<DefaultEnvironment>(account);
}

pub fn set_caller(account: AccountId) {
  test::set_caller::<DefaultEnvironment>(account);
}

pub fn balance_of(account: AccountId) -> Balance {
  test::get_account_balance::<DefaultEnvironment>(account).unwrap()
}

pub fn set_balance(account: AccountId, balance: Balance) {
  test::set_account_balance::<DefaultEnvironment>(account, balance);
}

pub fn decode_event<Event: Decode>(event: &ink::env::test::EmittedEvent) -> Event {
  <Event as Decode>::decode(&mut &event.data[..])
      .expect("encountered invalid contract event data buffer")
}

pub fn get_events<Event: Decode>() -> Vec<Event> {
    let raw_events = recorded_events().collect::<Vec<_>>();
    raw_events.iter().map(decode_event).collect()
}

pub fn admin_id() -> AccountId {
    get_accounts().alice
}

pub fn contract_id() -> AccountId {
    AccountId::from([0x09; 32])
}
