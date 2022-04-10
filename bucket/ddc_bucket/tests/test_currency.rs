use ink_lang as ink;

use crate::ddc_bucket::*;

use super::env_utils::*;

fn setup() -> DdcBucket {
    let contract = DdcBucket::new();
    contract
}

fn admin_id() -> AccountId {
    get_accounts().alice
}

#[ink::test]
fn currency_init_works() {
    let contract = setup();

    assert_eq!(contract.currency_get_conversion_rate(), 1,
               "conversion rate must be 1 initially");
}

#[ink::test]
fn currency_set_works() {
    let mut contract = setup();

    push_caller(admin_id());
    contract.currency_set_conversion_rate(9);
    pop_caller();

    assert_eq!(contract.currency_get_conversion_rate(), 9,
               "conversion rate must be changed");
}

#[ink::test]
#[should_panic]
fn currency_set_only_admin() {
    let mut contract = setup();
    let not_admin = get_accounts().bob;

    push_caller(not_admin);
    contract.currency_set_conversion_rate(9);
    pop_caller();
}
