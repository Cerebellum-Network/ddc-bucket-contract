use ink_lang as ink;

use crate::ddc_nft_registry::*;

use super::env_utils::*;

#[ink::test]
fn new_works() {
    let mut contract = DdcNftRegistry::new();

    set_balance(get_accounts().alice, 1000 * TOKEN);
    let reporter_id = get_accounts().alice;
    let nft_id = "0000000000000030ABCD1234ABCD1234ABCD1234ABCD1234ABCD12340000003132333435";
    let asset_id = "4321DCBA4321DCBA4321DCBA4321DCBA4321DCBA";
    let proof = "certified by cere";

    // Attach asset_id to nft_id
    push_caller_value(reporter_id, 1000 * TOKEN);
    contract.attach(nft_id.to_string(), asset_id.to_string(), proof.to_string());
    pop_caller();

    // Verify attachment of asset_id to nft_id
    let ev = get_events().pop().unwrap();
    assert!(matches!(ev, Event::Attach(ev) if ev ==
        Attach {
            reporter_id,
            nft_id:     nft_id.to_string(),
            asset_id:   asset_id.to_string(),
            proof:      proof.to_string(),
        }));

    let attachment_status = contract.get_by_nft_id(nft_id.to_string());
    assert_eq!(attachment_status.attachment.nft_id, nft_id.to_string());
    assert_eq!(attachment_status.attachment.asset_id, asset_id.to_string());
    assert_eq!(attachment_status.attachment.proof, proof.to_string());

    // Attach different attachment to nft_id
    let new_asset_id = "beefbeefbeefbeefbeefbeefbeefbeefbeefbeef";
    push_caller_value(reporter_id, 900 * TOKEN);
    contract.attach(nft_id.to_string(), new_asset_id.to_string(), proof.to_string());
    pop_caller();

    // Verify attachment of new_asset_id to nft_id
    let ev = get_events().pop().unwrap();
    assert!(matches!(ev, Event::Attach(ev) if ev ==
        Attach {
            reporter_id,
            nft_id:     nft_id.to_string(),
            asset_id:   new_asset_id.to_string(),
            proof:      proof.to_string(),
        }));

    let new_attachment_status = contract.get_by_nft_id(nft_id.to_string());
    assert_eq!(new_attachment_status.attachment.nft_id, nft_id.to_string());
    assert_eq!(new_attachment_status.attachment.asset_id, new_asset_id.to_string());
    assert_eq!(new_attachment_status.attachment.proof, proof.to_string());
}
