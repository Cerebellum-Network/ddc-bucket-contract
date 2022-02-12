use ink_env;
use ink_lang as ink;

use super::payments::*;

#[ink::test]
fn payments_works() {
    let accounts =
        ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()?;

    let mut payments = Payments::default();

    let tab_id1 = payments.create_tab(accounts.bob)?;
    let tab_id2 = payments.create_tab(accounts.bob)?;
    assert_ne!(tab_id1, tab_id2);

    payments.deposit()?;

    payments.increase_flow(accounts.alice, tab_id1.clone(), 3)?;
    payments.decrease_flow(accounts.alice, tab_id1, 1)?;

    payments.withdraw(1)?;
}
