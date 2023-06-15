use ink_lang as ink;

use crate::ddc_bucket::*;

use super::env_utils::*;

#[ink::test]
fn admin_init_works() {
    let contract = setup();

    // The deployer is SuperAdmin.
    assert!(contract.has_permission(admin_id(), Permission::SuperAdmin));
    assert!(!contract.has_permission(not_admin_id(), Permission::SuperAdmin));

    // The SuperAdmin has all other permissions, too.
    assert!(contract.has_permission(admin_id(), Permission::SetExchangeRate));
    assert!(!contract.has_permission(not_admin_id(), Permission::SetExchangeRate));
}

#[ink::test]
fn admin_withdraw_works() {
    let mut contract = setup();
    assert_eq!(balance_of(contract_id()), 10);
    
    set_caller(admin_id());

    contract.admin_withdraw(9);

    assert_eq!(balance_of(contract_id()), 1);
}

#[ink::test]
#[should_panic]
fn admin_withdraw_only_admin() {
    let mut contract = setup();

    set_caller(not_admin_id());
    
    contract.admin_withdraw(9); // panic.
}

#[ink::test]
fn admin_grant_works() {
    let mut contract = setup();
    let permission = Permission::SuperAdmin;

    set_caller_value(admin_id(), CONTRACT_FEE_LIMIT);

    let new_admin_id = not_admin_id();

    contract.admin_grant_permission(new_admin_id, permission);

    // Check the last event.
    let ev = get_events().pop().unwrap();
    assert!(matches!(ev, Event::PermissionGranted(ev) if ev ==
        PermissionGranted { account_id: not_admin_id(), permission }));

    assert!(contract.has_permission(new_admin_id, permission));

    set_caller(new_admin_id);

    contract.admin_withdraw(9);
}

#[ink::test]
#[should_panic]
fn admin_grant_only_admin() {
    let mut contract = setup();

    set_caller_value(not_admin_id(), CONTRACT_FEE_LIMIT);

    contract.admin_grant_permission(get_accounts().charlie, Permission::SuperAdmin); // panic.
}

#[ink::test]
#[should_panic]
fn admin_revoke_works() {
    let mut contract = setup();
    let permission = Permission::SuperAdmin;

    set_caller(admin_id());

    contract.admin_revoke_permission(admin_id(), permission);

    // Check the last event.
    let ev = get_events().pop().unwrap();
    assert!(matches!(ev, Event::PermissionRevoked(ev) if ev ==
        PermissionRevoked { account_id: not_admin_id(), permission }));

    assert!(!contract.has_permission(admin_id(), permission));

    // Cannot withdraw because no more permission.
    set_caller(admin_id());

    contract.admin_withdraw(9); // panic.
}

#[ink::test]
#[should_panic]
fn admin_revoke_only_admin() {
    let mut contract = setup();

    set_caller_value(not_admin_id(), CONTRACT_FEE_LIMIT);

    contract.admin_revoke_permission(admin_id(), Permission::SuperAdmin); // panic.
}

fn setup() -> DdcBucket {
    set_caller(admin_id());
    set_callee(contract_id());
    let contract = DdcBucket::new();
    set_balance(contract_id(), 10);
    contract
}

fn not_admin_id() -> AccountId {
    get_accounts().bob
}