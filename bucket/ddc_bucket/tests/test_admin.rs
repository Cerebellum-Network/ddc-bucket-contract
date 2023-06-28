use ink_lang as ink;

use crate::ddc_bucket::*;
use crate::ddc_bucket::Error::*;
use super::env_utils::*;
use super::setup_utils::*;


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

    contract.admin_withdraw(9);

    assert_eq!(balance_of(contract_id()), 1);
}

#[ink::test]
#[should_panic]
fn admin_withdraw_err_if_not_admin() {
    let mut contract = setup_contract();

    set_caller(not_admin_id());
    
    contract.admin_withdraw(9); // panic.
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

    contract.admin_withdraw(9);
}


#[ink::test]
fn admin_grant_err_if_not_admin() {
    let mut contract = setup_contract();

    set_caller_value(not_admin_id(), CONTRACT_FEE_LIMIT);

    assert_eq!(
        contract.admin_grant_permission(
            get_accounts().charlie, 
            Permission::SuperAdmin
        ),
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

    contract.admin_withdraw(9); // panic.
}


#[ink::test]
fn admin_revoke_err_if_not_admin() {
    let mut contract = setup_contract();

    set_caller_value(not_admin_id(), CONTRACT_FEE_LIMIT);

    assert_eq!(
        contract.admin_revoke_permission(
            admin_id(), 
            Permission::SuperAdmin
        ),
        Err(OnlySuperAdmin)
    );
}