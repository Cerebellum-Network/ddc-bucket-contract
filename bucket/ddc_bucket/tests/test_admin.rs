use ink_lang as ink;

use crate::ddc_bucket::*;

use super::env_utils::*;

#[ink::test]
fn admin_init_works() {
    let contract = setup();

    assert!(contract.perm_has(admin_id(), Perm::SuperAdmin));
    assert!(!contract.perm_has(not_admin_id(), Perm::SuperAdmin));
}


#[ink::test]
fn admin_withdraw_works() {
    let mut contract = setup();

    push_caller(admin_id());
    contract.admin_withdraw(9);
    pop_caller();

    assert_eq!(balance_of(contract_id()), 1);
}

#[ink::test]
#[should_panic]
fn admin_withdraw_only_admin() {
    let mut contract = setup();

    push_caller(not_admin_id());
    contract.admin_withdraw(9); // panic.
    pop_caller();
}


#[ink::test]
fn admin_grant_works() {
    let mut contract = setup();

    push_caller_value(admin_id(), CONTRACT_FEE_LIMIT);
    contract.admin_grant_perm(not_admin_id(), Perm::SuperAdmin);
    pop_caller();

    assert!(contract.perm_has(not_admin_id(), Perm::SuperAdmin));

    push_caller(not_admin_id());
    contract.admin_withdraw(9);
    pop_caller();
}

#[ink::test]
#[should_panic]
fn admin_grant_only_admin() {
    let mut contract = DdcBucket::new();

    push_caller_value(not_admin_id(), CONTRACT_FEE_LIMIT);
    contract.admin_grant_perm(get_accounts().charlie, Perm::SuperAdmin); // panic.
    pop_caller();
}


#[ink::test]
#[should_panic]
fn admin_revoke_works() {
    let mut contract = setup();

    // Revoke the permission.
    push_caller(admin_id());
    contract.admin_revoke_perm(admin_id(), Perm::SuperAdmin);
    pop_caller();

    assert!(!contract.perm_has(admin_id(), Perm::SuperAdmin));

    // Cannot withdraw because no more permission.
    push_caller(admin_id());
    contract.admin_withdraw(9); // panic.
    pop_caller();
}

#[ink::test]
#[should_panic]
fn admin_revoke_only_admin() {
    let mut contract = DdcBucket::new();

    push_caller_value(not_admin_id(), CONTRACT_FEE_LIMIT);
    contract.admin_revoke_perm(admin_id(), Perm::SuperAdmin); // panic.
    pop_caller();
}


fn setup() -> DdcBucket {
    let contract = DdcBucket::new();
    set_balance(contract_id(), 10);
    contract
}

fn admin_id() -> AccountId { get_accounts().alice }

fn not_admin_id() -> AccountId { get_accounts().bob }
