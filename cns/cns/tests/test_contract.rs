use ink_lang as ink;

use crate::cns::*;

use super::env_utils::*;

#[ink::test]
fn new_works() {
    let mut contract = CNS::new();

    let owner_id = get_accounts().alice;
    let name = "me";

    push_caller_value(owner_id, 0);
    contract.claim_name(name.to_string());
    pop_caller();

    // Check the last event.
    let ev = get_events().pop().unwrap();
    assert!(matches!(ev, Event::AllocateName(ev) if ev ==
        AllocateName {
            owner_id,
            name: name.to_string(),
        }));
}
