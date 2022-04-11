use ink_lang as ink;

use crate::ddc_bucket::*;

use super::env_utils::*;

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
fn admin_change_works() {
    let mut contract = DdcBucket::new();
    let admin = get_accounts().alice;
    set_balance(contract_id(), 10);

    let admin0 = get_accounts().alice;
    let admin1 = get_accounts().bob;

    push_caller_value(admin0, CONTRACT_FEE_LIMIT);
    contract.admin_grant(admin1, Perm::SuperAdmin);
    pop_caller();

    push_caller(admin);
    contract.admin_withdraw(9);
    pop_caller();
}

#[ink::test]
#[should_panic]
fn admin_change_only_admin() {
    let mut contract = DdcBucket::new();
    let not_admin = get_accounts().bob;

    push_caller_value(not_admin, CONTRACT_FEE_LIMIT);
    contract.admin_grant(get_accounts().charlie, Perm::SuperAdmin); // panic.
    pop_caller();
}
