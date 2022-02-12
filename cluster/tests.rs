/// Imports `ink_lang` so we can use `#[ink::test]`.
use ink_lang as ink;

/// Imports all the definitions from the outer scope so we can use them here.
use super::cluster::*;

/// We test if the default constructor does its job.
#[ink::test]
fn default_works() {
    let _cluster = Cluster::default();
}
