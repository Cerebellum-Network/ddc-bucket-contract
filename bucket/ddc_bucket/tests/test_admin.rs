use ink_lang as ink;

use crate::ddc_bucket::*;

use super::env_utils::*;

#[ink::test]
fn admin_init_works() {
    let contract = DdcBucket::new();
    let admin = get_accounts().alice;
    let not_admin = get_accounts().bob;

    assert!(contract.perm_has(admin, Perm::SuperAdmin));
    assert!(!contract.perm_has(not_admin, Perm::SuperAdmin));
}

#[ink::test]
fn admin_withdraw_works() {
    let mut contract = DdcBucket::new();
    let admin = get_accounts().alice;
    set_balance(contract_id(), 10);

    push_caller(admin);
    contract.admin_withdraw(9);
    pop_caller();

    assert_eq!(balance_of(contract_id()), 1);
}

#[ink::test]
#[should_panic]
fn admin_withdraw_only_admin() {
    let mut contract = DdcBucket::new();
    let not_admin = get_accounts().bob;
    set_balance(contract_id(), 10);

    push_caller(not_admin);
    contract.admin_withdraw(9); // panic.
    pop_caller();
}

#[ink::test]
fn admin_grant_works() {
    let mut contract = DdcBucket::new();
    let admin = get_accounts().alice;
    set_balance(contract_id(), 10);

    let admin0 = get_accounts().alice;
    let admin1 = get_accounts().bob;

    push_caller_value(admin0, CONTRACT_FEE_LIMIT);
    contract.admin_grant_perm(admin1, Perm::SuperAdmin);
    pop_caller();

    assert!(contract.perm_has(admin1, Perm::SuperAdmin));

    push_caller(admin);
    contract.admin_withdraw(9);
    pop_caller();
}

#[ink::test]
#[should_panic]
fn admin_grant_only_admin() {
    let mut contract = DdcBucket::new();
    let not_admin = get_accounts().bob;

    push_caller_value(not_admin, CONTRACT_FEE_LIMIT);
    contract.admin_grant_perm(get_accounts().charlie, Perm::SuperAdmin); // panic.
    pop_caller();
}

#[ink::test]
#[should_panic]
fn admin_revoke_only_admin() {
    let mut contract = DdcBucket::new();
    let not_admin = get_accounts().bob;

    push_caller_value(not_admin, CONTRACT_FEE_LIMIT);
    contract.admin_revoke_perm(get_accounts().alice, Perm::SuperAdmin); // panic.
    pop_caller();
}

#[ink::test]
#[should_panic]
fn admin_revoke_works() {
    let mut contract = DdcBucket::new();
    let admin = get_accounts().alice;
    set_balance(contract_id(), 10);

    // Revoke the permission.
    push_caller(admin);
    contract.admin_revoke_perm(admin, Perm::SuperAdmin);
    pop_caller();

    assert!(!contract.perm_has(admin, Perm::SuperAdmin));

    // Cannot withdraw because no more permission.
    push_caller(admin);
    contract.admin_withdraw(9); // panic.
    pop_caller();
}
