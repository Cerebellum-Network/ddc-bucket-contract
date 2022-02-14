use ink_env;
use ink_lang as ink;

use super::ddc_billing::*;

#[ink::test]
fn billing_works() {
    let accounts =
        ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()?;

    let mut billing = DdcBilling::new();

    let tab_id1 = billing.create_tab(accounts.bob)?;
    let tab_id2 = billing.create_tab(accounts.bob)?;
    assert_ne!(tab_id1, tab_id2);

    billing.deposit()?;

    billing.increase_flow(accounts.alice, tab_id1.clone(), 3)?;
    billing.decrease_flow(accounts.alice, tab_id1, 1)?;

    billing.withdraw(1)?;
}
