use ink_lang as ink;

use crate::cns::*;

use super::env_utils::*;

fn setup() -> CNS {
    let mut contract = CNS::new();

    push_caller_value(owner_id(), 0);
    contract.claim_name(name());
    pop_caller();

    contract
}


#[ink::test]
fn claim_name_works() {
    let contract = setup();

    // Unknown name.
    assert_eq!(contract.get_by_name("unknown".to_string()), Err(NameDoesNotExist));

    // Existing name with an empty payload.
    let record = contract.get_by_name(name())?;

    assert_eq!(record, Record {
        owner_id: owner_id(),
        payload: "".to_string(),
    });

    // Check the last event.
    let ev = get_events().pop().unwrap();
    assert!(matches!(ev, Event::AllocateName(ev) if ev ==
        AllocateName {
            name: name(),
            owner_id: owner_id(),
        }));
}


#[ink::test]
#[should_panic]
fn claim_name_already_taken() {
    let mut contract = setup();

    push_caller_value(get_accounts().bob, 0);
    contract.claim_name(name());
    pop_caller();
}


#[ink::test]
#[should_panic]
fn claim_name_invalid() {
    let mut contract = setup();

    push_caller_value(owner_id(), 0);
    contract.claim_name("0f".to_string());
    pop_caller();
}


#[ink::test]
#[should_panic]
fn claim_name_empty() {
    let mut contract = setup();

    push_caller_value(owner_id(), 0);
    contract.claim_name("".to_string());
    pop_caller();
}


#[ink::test]
fn set_payload_works() {
    let mut contract = setup();

    push_caller_value(owner_id(), 0);
    contract.set_payload(name(), payload());
    pop_caller();

    let record = contract.get_by_name(name())?;

    assert_eq!(record, Record {
        owner_id: owner_id(),
        payload: payload(),
    });

    // Check the last event.
    let ev = get_events().pop().unwrap();
    assert!(matches!(ev, Event::SetPayload(ev) if ev ==
        SetPayload {
            name: name(),
            payload: payload(),
        }));
}


#[ink::test]
#[should_panic]
fn set_payload_only_owner() {
    let mut contract = setup();

    push_caller_value(get_accounts().bob, 0);
    contract.set_payload(name(), payload());
    pop_caller();
}


fn owner_id() -> AccountId { get_accounts().alice }

fn name() -> String { "me".to_string() }

fn payload() -> String { "my something".to_string() }
