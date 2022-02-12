/// Imports `ink_lang` so we can use `#[ink::test]`.
use ink_lang as ink;

/// Imports all the definitions from the outer scope so we can use them here.
use super::payments::*;

#[ink::test]
fn payments_works() {
    let mut payments = Payments::default();
}
