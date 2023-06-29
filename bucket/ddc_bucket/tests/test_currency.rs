use ink_lang as ink;

use crate::ddc_bucket::*;
use super::env_utils::*;
use super::setup_utils::*;


#[ink::test]
fn currency_conversion_init_ok() {
    let contract = setup_contract();
    let usd_amount = contract.account_get_usd_per_cere();
    println!("{}", usd_amount);
    assert_eq!(
        contract.account_get_usd_per_cere(),
        1 * TOKEN,
        "conversion rate must be 1 initially"
    );
}


#[ink::test]
fn currency_conversion_set_rate_ok() {
    let mut contract = setup_contract();
    let usd_per_cere = TOKEN / 10;
    println!("{}", usd_per_cere);

    set_caller(admin_id());
    contract.account_set_usd_per_cere(usd_per_cere);

    assert_eq!(
        contract.account_get_usd_per_cere(),
        usd_per_cere,
        "conversion rate must be changed"
    );
}


#[ink::test]
#[should_panic]
fn currency_conversion_set_rate_err_if_not_admin() {
    let mut contract = setup_contract();
    let not_admin = get_accounts().bob;

    set_caller(not_admin);
    contract.account_set_usd_per_cere(9);
}


#[ink::test]
fn converter_ok() {
    // todo: this test scenario must be revised as it does pure printing without any assertion
    println!("Creating new cdn cluster");

    let mut ctx = setup_cluster();

    // The provider stops trusting the manager_id.
    println!("Cdn cluster id is {}", ctx.cluster_id);
    set_caller(ctx.manager_id);
    ctx.contract.cdn_set_rate(ctx.cluster_id, 3_750_000_000);
    set_caller(ctx.provider_id0);
    let rate = ctx.contract.cdn_get_rate(ctx.cluster_id);

    let usd_per_cere = TOKEN / 100;
    set_caller(admin_id());
    ctx.contract.account_set_usd_per_cere(usd_per_cere);

    let usd_amount = ctx.contract.account_get_usd_per_cere();
    println!("Current usd amount is {}", usd_amount);

    println!("The current rate is {}", rate);

    let usd_per_kb = rate / KB_PER_GB;
    println!("The current rate per kb {}", usd_per_kb);

    let cere_per_kb = ctx.contract.protocol.curr_converter.to_cere(usd_per_kb);
    println!("The current cere rate per kb {}", cere_per_kb);
}
