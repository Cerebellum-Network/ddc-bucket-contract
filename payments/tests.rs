/// Imports `ink_lang` so we can use `#[ink::test]`.
use ink_lang as ink;
use ink_env;

/// Imports all the definitions from the outer scope so we can use them here.
use super::payments::*;

#[ink::test]
fn payments_works() {
    let accounts =
        ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()?;

    let mut payments = Payments::default();

    payments.deposit()?;

    payments.increase_flow(accounts.alice, accounts.bob, 3)?;
    payments.decrease_flow(accounts.alice, accounts.bob, 1)?;

    payments.withdraw(1)?;
}
