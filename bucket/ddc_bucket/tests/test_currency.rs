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
fn currency_conversion_init_works() {
    let contract = setup();

    assert_eq!(contract.account_get_usd_per_cere(), 1 * TOKEN,
               "conversion rate must be 1 initially");
}

#[ink::test]
fn currency_conversion_set_rate_works() {
    let mut contract = setup();
    let usd_per_cere = TOKEN / 10;

    push_caller(admin_id());
    contract.account_set_usd_per_cere(usd_per_cere);
    pop_caller();

    assert_eq!(contract.account_get_usd_per_cere(), usd_per_cere,
               "conversion rate must be changed");
}

#[ink::test]
#[should_panic]
fn currency_conversion_set_rate_only_admin() {
    let mut contract = setup();
    let not_admin = get_accounts().bob;

    push_caller(not_admin);
    contract.account_set_usd_per_cere(9);
    pop_caller();
}
