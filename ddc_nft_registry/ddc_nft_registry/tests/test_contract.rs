use ink_lang as ink;

use crate::ddc_nft_registry::*;

use super::env_utils::*;

#[ink::test]
fn new_works() {
    let mut contract = DdcNftRegistry::new();

    let reporter_id = get_accounts().alice;
    let nft_id = "nft:polygon/freeport_1/2";
    let asset_id = "ddc:1234";
    let proof = "certified by cere";

    push_caller_value(reporter_id, 0);
    contract.attach(nft_id.to_string(), asset_id.to_string(), proof.to_string());
    pop_caller();

    // Check the last event.
    let ev = get_events().pop().unwrap();
    assert!(matches!(ev, Event::Attach(ev) if ev ==
        Attach {
            reporter_id,
            nft_id:nft_id.to_string(),
            asset_id:asset_id.to_string(),
            proof:proof.to_string(),
        }));
}
