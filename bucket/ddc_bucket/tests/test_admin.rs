use ink_lang as ink;

use super::env_utils::*;
use super::setup_utils::*;
use crate::ddc_bucket::Error::*;
use crate::ddc_bucket::*;

fn not_admin_id() -> AccountId {
    get_accounts().bob
}

#[ink::test]
fn admin_init_ok() {
    let contract = setup_contract();

    // The deployer is SuperAdmin.
    assert!(contract.has_permission(admin_id(), Permission::SuperAdmin));
    assert!(!contract.has_permission(not_admin_id(), Permission::SuperAdmin));

    // The SuperAdmin has all other permissions, too.
    assert!(contract.has_permission(admin_id(), Permission::SetExchangeRate));
    assert!(!contract.has_permission(not_admin_id(), Permission::SetExchangeRate));
}

#[ink::test]
fn admin_withdraw_ok() {
    let mut contract = setup_contract();
    assert_eq!(balance_of(contract_id()), 10);

    set_caller(admin_id());

    contract.admin_withdraw(9).unwrap();

    assert_eq!(balance_of(contract_id()), 1);
}

#[ink::test]
#[should_panic]
fn admin_withdraw_err_if_not_admin() {
    let mut contract = setup_contract();

    set_caller(not_admin_id());

    contract.admin_withdraw(9).unwrap(); // panic.
}

#[ink::test]
fn admin_grant_ok() {
    let mut contract = setup_contract();
    let permission = Permission::SuperAdmin;

    set_caller_value(admin_id(), CONTRACT_FEE_LIMIT);

    let new_admin_id = not_admin_id();

    contract.admin_grant_permission(new_admin_id, permission)?;

    // Check the last event.
    let ev = get_events().pop().unwrap();
    assert!(matches!(ev, Event::PermissionGranted(ev) if ev ==
        PermissionGranted { account_id: not_admin_id(), permission }));

    assert!(contract.has_permission(new_admin_id, permission));

    set_caller(new_admin_id);

    contract.admin_withdraw(9).unwrap();
}

#[ink::test]
fn admin_grant_err_if_not_admin() {
    let mut contract = setup_contract();

    set_caller_value(not_admin_id(), CONTRACT_FEE_LIMIT);

    assert_eq!(
        contract.admin_grant_permission(get_accounts().charlie, Permission::SuperAdmin),
        Err(OnlySuperAdmin)
    );
}

#[ink::test]
#[should_panic]
fn admin_revoke_ok() {
    let mut contract = setup_contract();
    let permission = Permission::SuperAdmin;

    set_caller(admin_id());

    contract.admin_revoke_permission(admin_id(), permission)?;

    // Check the last event.
    let ev = get_events().pop().unwrap();
    assert!(matches!(ev, Event::PermissionRevoked(ev) if ev ==
        PermissionRevoked { account_id: not_admin_id(), permission }));

    assert!(!contract.has_permission(admin_id(), permission));

    // Cannot withdraw because no more permission.
    set_caller(admin_id());

    contract.admin_withdraw(9)?;
}

#[ink::test]
fn admin_revoke_err_if_not_admin() {
    let mut contract = setup_contract();

    set_caller_value(not_admin_id(), CONTRACT_FEE_LIMIT);

    assert_eq!(
        contract.admin_revoke_permission(admin_id(), Permission::SuperAdmin),
        Err(OnlySuperAdmin)
    );
}

#[ink::test]
fn admin_transfer_node_ownership_err_if_not_admin() {
    let mut contract = setup_contract();

    let new_node_key = AccountId::from([
        0xc4, 0xcd, 0xaa, 0xfa, 0xf1, 0x30, 0x7d, 0x23, 0xf4, 0x99, 0x84, 0x71, 0xdf, 0x78, 0x59,
        0xce, 0x06, 0x3d, 0xce, 0x78, 0x59, 0xc4, 0x3a, 0xe8, 0xef, 0x12, 0x0a, 0xbc, 0x43, 0xc4,
        0x84, 0x31,
    ]);
    let not_admin_id = AccountId::from([
        0x00, 0xc9, 0x91, 0xf1, 0x63, 0x0f, 0xb4, 0x51, 0xf6, 0x6c, 0x9e, 0xa5, 0xc6, 0xdd, 0xf3,
        0x33, 0xd8, 0x48, 0x75, 0xc6, 0x22, 0xf5, 0xd3, 0xde, 0x4a, 0x39, 0xe7, 0x71, 0x6f, 0x74,
        0xf0, 0x49,
    ]);
    set_balance(not_admin_id, 1000 * TOKEN);

    set_caller_value(not_admin_id, CONTRACT_FEE_LIMIT);
    contract.node_create(
        new_node_key,
        NodeParams::from("{\"url\":\"https://ddc-1.cere.network/storage/new\"}"),
        100,
        10 * TOKEN,
    )?;

    let node_info = contract.node_get(new_node_key)?;
    assert_eq!(node_info.node.provider_id, not_admin_id);

    let new_owner_id = AccountId::from([
        0xf8, 0x9e, 0xfb, 0x5c, 0x80, 0x72, 0x8e, 0x2a, 0x69, 0x54, 0x73, 0x32, 0x52, 0x8b, 0x03,
        0xb7, 0x9d, 0x2c, 0xd5, 0x06, 0xed, 0x38, 0x72, 0x95, 0x19, 0x9c, 0x6b, 0x8f, 0x7e, 0xa3,
        0x47, 0x16,
    ]);
    set_balance(new_owner_id, 1000 * TOKEN);

    set_caller(not_admin_id);

    assert_eq!(
        contract.admin_transfer_node_ownership(new_node_key, new_owner_id),
        Err(OnlySuperAdmin)
    );
}

#[ink::test]
fn admin_transfer_node_ownership_err_if_provider_is_not_admin() {
    let mut contract = setup_contract();

    let new_node_key = AccountId::from([
        0xc4, 0xcd, 0xaa, 0xfa, 0xf1, 0x30, 0x7d, 0x23, 0xf4, 0x99, 0x84, 0x71, 0xdf, 0x78, 0x59,
        0xce, 0x06, 0x3d, 0xce, 0x78, 0x59, 0xc4, 0x3a, 0xe8, 0xef, 0x12, 0x0a, 0xbc, 0x43, 0xc4,
        0x84, 0x31,
    ]);
    let not_admin_id = AccountId::from([
        0x00, 0xc9, 0x91, 0xf1, 0x63, 0x0f, 0xb4, 0x51, 0xf6, 0x6c, 0x9e, 0xa5, 0xc6, 0xdd, 0xf3,
        0x33, 0xd8, 0x48, 0x75, 0xc6, 0x22, 0xf5, 0xd3, 0xde, 0x4a, 0x39, 0xe7, 0x71, 0x6f, 0x74,
        0xf0, 0x49,
    ]);
    set_balance(not_admin_id, 1000 * TOKEN);

    set_caller_value(not_admin_id, CONTRACT_FEE_LIMIT);
    contract.node_create(
        new_node_key,
        NodeParams::from("{\"url\":\"https://ddc-1.cere.network/storage/new\"}"),
        100,
        10 * TOKEN,
    )?;

    let node_info = contract.node_get(new_node_key)?;
    assert_eq!(node_info.node.provider_id, not_admin_id);

    let new_owner_id = AccountId::from([
        0xf8, 0x9e, 0xfb, 0x5c, 0x80, 0x72, 0x8e, 0x2a, 0x69, 0x54, 0x73, 0x32, 0x52, 0x8b, 0x03,
        0xb7, 0x9d, 0x2c, 0xd5, 0x06, 0xed, 0x38, 0x72, 0x95, 0x19, 0x9c, 0x6b, 0x8f, 0x7e, 0xa3,
        0x47, 0x16,
    ]);
    set_balance(new_owner_id, 1000 * TOKEN);

    set_caller(admin_id());

    assert_eq!(
        contract.admin_transfer_node_ownership(new_node_key, new_owner_id),
        Err(NodeProviderIsNotSuperAdmin)
    );
}

#[ink::test]
fn admin_transfer_node_ownership_ok() {
    let mut contract = setup_contract();

    let new_node_key = AccountId::from([
        0xc4, 0xcd, 0xaa, 0xfa, 0xf1, 0x30, 0x7d, 0x23, 0xf4, 0x99, 0x84, 0x71, 0xdf, 0x78, 0x59,
        0xce, 0x06, 0x3d, 0xce, 0x78, 0x59, 0xc4, 0x3a, 0xe8, 0xef, 0x12, 0x0a, 0xbc, 0x43, 0xc4,
        0x84, 0x31,
    ]);
    set_caller_value(admin_id(), CONTRACT_FEE_LIMIT);
    contract.node_create(
        new_node_key,
        NodeParams::from("{\"url\":\"https://ddc-1.cere.network/storage/new\"}"),
        100,
        10 * TOKEN,
    )?;

    let node_info1 = contract.node_get(new_node_key)?;
    assert_eq!(node_info1.node.provider_id, admin_id());

    let new_owner_id = AccountId::from([
        0xf8, 0x9e, 0xfb, 0x5c, 0x80, 0x72, 0x8e, 0x2a, 0x69, 0x54, 0x73, 0x32, 0x52, 0x8b, 0x03,
        0xb7, 0x9d, 0x2c, 0xd5, 0x06, 0xed, 0x38, 0x72, 0x95, 0x19, 0x9c, 0x6b, 0x8f, 0x7e, 0xa3,
        0x47, 0x16,
    ]);
    set_balance(new_owner_id, 1000 * TOKEN);

    set_caller(admin_id());
    contract.admin_transfer_node_ownership(new_node_key, new_owner_id)?;

    let node_info2 = contract.node_get(new_node_key)?;
    assert_eq!(node_info2.node.provider_id, new_owner_id);

    assert!(
        matches!(get_events().pop().unwrap(), Event::NodeOwnershipTransferred(ev) if ev ==
            NodeOwnershipTransferred {
                account_id: new_owner_id,
                node_key: new_node_key
            }
        )
    );
}

#[ink::test]
fn admin_transfer_cdn_node_ownership_err_if_not_admin() {
    let mut contract = setup_contract();

    let new_cdn_node_key = AccountId::from([
        0xc4, 0xcd, 0xaa, 0xfa, 0xf1, 0x30, 0x7d, 0x23, 0xf4, 0x99, 0x84, 0x71, 0xdf, 0x78, 0x59,
        0xce, 0x06, 0x3d, 0xce, 0x78, 0x59, 0xc4, 0x3a, 0xe8, 0xef, 0x12, 0x0a, 0xbc, 0x43, 0xc4,
        0x84, 0x31,
    ]);
    let not_admin_id = AccountId::from([
        0x00, 0xc9, 0x91, 0xf1, 0x63, 0x0f, 0xb4, 0x51, 0xf6, 0x6c, 0x9e, 0xa5, 0xc6, 0xdd, 0xf3,
        0x33, 0xd8, 0x48, 0x75, 0xc6, 0x22, 0xf5, 0xd3, 0xde, 0x4a, 0x39, 0xe7, 0x71, 0x6f, 0x74,
        0xf0, 0x49,
    ]);
    set_balance(not_admin_id, 1000 * TOKEN);

    set_caller_value(not_admin_id, CONTRACT_FEE_LIMIT);
    contract.cdn_node_create(
        new_cdn_node_key,
        CdnNodeParams::from("{\"url\":\"https://ddc-1.cere.network/cdn/new\"}"),
    )?;

    let cdn_node_info = contract.cdn_node_get(new_cdn_node_key)?;
    assert_eq!(cdn_node_info.cdn_node.provider_id, not_admin_id);

    let new_owner_id = AccountId::from([
        0xf8, 0x9e, 0xfb, 0x5c, 0x80, 0x72, 0x8e, 0x2a, 0x69, 0x54, 0x73, 0x32, 0x52, 0x8b, 0x03,
        0xb7, 0x9d, 0x2c, 0xd5, 0x06, 0xed, 0x38, 0x72, 0x95, 0x19, 0x9c, 0x6b, 0x8f, 0x7e, 0xa3,
        0x47, 0x16,
    ]);
    set_balance(new_owner_id, 1000 * TOKEN);

    set_caller(not_admin_id);

    assert_eq!(
        contract.admin_transfer_cdn_node_ownership(new_cdn_node_key, new_owner_id),
        Err(OnlySuperAdmin)
    );
}

#[ink::test]
fn admin_transfer_cdn_node_ownership_err_if_provider_is_not_admin() {
    let mut contract = setup_contract();

    let new_cdn_node_key = AccountId::from([
        0xc4, 0xcd, 0xaa, 0xfa, 0xf1, 0x30, 0x7d, 0x23, 0xf4, 0x99, 0x84, 0x71, 0xdf, 0x78, 0x59,
        0xce, 0x06, 0x3d, 0xce, 0x78, 0x59, 0xc4, 0x3a, 0xe8, 0xef, 0x12, 0x0a, 0xbc, 0x43, 0xc4,
        0x84, 0x31,
    ]);
    let not_admin_id = AccountId::from([
        0x00, 0xc9, 0x91, 0xf1, 0x63, 0x0f, 0xb4, 0x51, 0xf6, 0x6c, 0x9e, 0xa5, 0xc6, 0xdd, 0xf3,
        0x33, 0xd8, 0x48, 0x75, 0xc6, 0x22, 0xf5, 0xd3, 0xde, 0x4a, 0x39, 0xe7, 0x71, 0x6f, 0x74,
        0xf0, 0x49,
    ]);
    set_balance(not_admin_id, 1000 * TOKEN);

    set_caller_value(not_admin_id, CONTRACT_FEE_LIMIT);
    contract.cdn_node_create(
        new_cdn_node_key,
        CdnNodeParams::from("{\"url\":\"https://ddc-1.cere.network/cdn/new\"}"),
    )?;

    let cdn_node_info = contract.cdn_node_get(new_cdn_node_key)?;
    assert_eq!(cdn_node_info.cdn_node.provider_id, not_admin_id);

    let new_owner_id = AccountId::from([
        0xf8, 0x9e, 0xfb, 0x5c, 0x80, 0x72, 0x8e, 0x2a, 0x69, 0x54, 0x73, 0x32, 0x52, 0x8b, 0x03,
        0xb7, 0x9d, 0x2c, 0xd5, 0x06, 0xed, 0x38, 0x72, 0x95, 0x19, 0x9c, 0x6b, 0x8f, 0x7e, 0xa3,
        0x47, 0x16,
    ]);
    set_balance(new_owner_id, 1000 * TOKEN);

    set_caller(admin_id());

    assert_eq!(
        contract.admin_transfer_cdn_node_ownership(new_cdn_node_key, new_owner_id),
        Err(CdnNodeOwnerIsNotSuperAdmin)
    );
}

#[ink::test]
fn admin_transfer_cdn_node_ownership_ok() {
    let mut contract = setup_contract();

    let new_cdn_node_key = AccountId::from([
        0xc4, 0xcd, 0xaa, 0xfa, 0xf1, 0x30, 0x7d, 0x23, 0xf4, 0x99, 0x84, 0x71, 0xdf, 0x78, 0x59,
        0xce, 0x06, 0x3d, 0xce, 0x78, 0x59, 0xc4, 0x3a, 0xe8, 0xef, 0x12, 0x0a, 0xbc, 0x43, 0xc4,
        0x84, 0x31,
    ]);
    set_caller_value(admin_id(), CONTRACT_FEE_LIMIT);
    contract.cdn_node_create(
        new_cdn_node_key,
        CdnNodeParams::from("{\"url\":\"https://ddc-1.cere.network/cdn/new\"}"),
    )?;

    let cdn_node_info1 = contract.cdn_node_get(new_cdn_node_key)?;
    assert_eq!(cdn_node_info1.cdn_node.provider_id, admin_id());

    let new_owner_id = AccountId::from([
        0xf8, 0x9e, 0xfb, 0x5c, 0x80, 0x72, 0x8e, 0x2a, 0x69, 0x54, 0x73, 0x32, 0x52, 0x8b, 0x03,
        0xb7, 0x9d, 0x2c, 0xd5, 0x06, 0xed, 0x38, 0x72, 0x95, 0x19, 0x9c, 0x6b, 0x8f, 0x7e, 0xa3,
        0x47, 0x16,
    ]);
    set_balance(new_owner_id, 1000 * TOKEN);

    set_caller(admin_id());
    contract.admin_transfer_cdn_node_ownership(new_cdn_node_key, new_owner_id)?;

    let cdn_node_info2 = contract.cdn_node_get(new_cdn_node_key)?;
    assert_eq!(cdn_node_info2.cdn_node.provider_id, new_owner_id);

    assert!(
        matches!(get_events().pop().unwrap(), Event::CdnNodeOwnershipTransferred(ev) if ev ==
            CdnNodeOwnershipTransferred {
                account_id: new_owner_id,
                cdn_node_key: new_cdn_node_key
            }
        )
    );
}
