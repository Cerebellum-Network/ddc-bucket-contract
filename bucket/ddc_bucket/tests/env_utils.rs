#![allow(unused_variables, dead_code)]

pub use ink_env::{
    block_timestamp, call, test,
    test::{advance_block, DefaultAccounts},
    DefaultEnvironment,
};

use crate::ddc_bucket::*;
use scale::Decode;

pub type Event = <DdcBucket as ::ink_lang::reflect::ContractEventBase>::Type;

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

pub fn decode_event<Event: Decode>(event: &ink_env::test::EmittedEvent) -> Event {
    <Event as Decode>::decode(&mut &event.data[..])
        .expect("encountered invalid contract event data buffer")
}

pub fn get_events<Event: Decode>() -> Vec<Event> {
    let raw_events = test::recorded_events().collect::<Vec<_>>();
    raw_events.iter().map(decode_event).collect()
}

pub fn print_events(events: &[Event]) {
    for ev in events.iter() {
        match ev {
            Event::ClusterCreated(ev) => println!("EVENT {:?}", ev),
            Event::ClusterNodeReplaced(ev) => println!("EVENT {:?}", ev),
            Event::ClusterNodeReset(ev) => println!("EVENT {:?}", ev),
            Event::ClusterReserveResource(ev) => println!("EVENT {:?}", ev),
            Event::ClusterDistributeRevenues(ev) => println!("EVENT {:?}", ev),
            Event::NodeCreated(ev) => println!("EVENT {:?}", ev),
            Event::BucketCreated(ev) => println!("EVENT {:?}", ev),
            Event::BucketAllocated(ev) => println!("EVENT {:?}", ev),
            Event::BucketSettlePayment(ev) => println!("EVENT {:?}", ev),
            Event::BucketAvailabilityUpdated(ev) => println!("EVENT {:?}", ev),
            Event::Deposit(ev) => println!("EVENT {:?}", ev),
            Event::PermissionGranted(ev) => println!("EVENT {:?}", ev),
            Event::PermissionRevoked(ev) => println!("EVENT {:?}", ev),
            Event::ClusterDistributeCdnRevenues(ev) => println!("EVENT {:?}", ev),
            Event::CdnNodeCreated(ev) => println!("EVENT {:?}", ev),
            Event::ClusterNodeAdded(ev) => println!("EVENT {:?}", ev),
            Event::ClusterCdnNodeAdded(ev) => println!("{:?}", ev),
            Event::ClusterNodeRemoved(ev) => println!("EVENT {:?}", ev),
            Event::ClusterCdnNodeRemoved(ev) => println!("EVENT {:?}", ev),
            Event::ClusterParamsSet(ev) => println!("EVENT {:?}", ev),
            Event::ClusterRemoved(ev) => println!("EVENT {:?}", ev),
            Event::ClusterNodeStatusSet(ev) => println!("EVENT {:?}", ev),
            Event::ClusterCdnNodeStatusSet(ev) => println!("EVENT {:?}", ev),
            Event::CdnNodeRemoved(ev) => println!("EVENT {:?}", ev),
            Event::CdnNodeParamsSet(ev) => println!("EVENT {:?}", ev),
            Event::NodeRemoved(ev) => println!("EVENT {:?}", ev),
            Event::NodeParamsSet(ev) => println!("EVENT {:?}", ev),
            Event::NodeOwnershipTransferred(ev) => println!("EVENT {:?}", ev),
            Event::CdnNodeOwnershipTransferred(ev) => println!("EVENT {:?}", ev),
            Event::BucketParamsSet(ev) => println!("EVENT {:?}", ev),
        }
    }
}
