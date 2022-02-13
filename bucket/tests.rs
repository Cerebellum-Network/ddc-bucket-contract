/// Imports `ink_lang` so we can use `#[ink::test]`.
use ink_lang as ink;

/// Imports all the definitions from the outer scope so we can use them here.
use super::ddc_bucket::*;

/// We test if the default constructor does its job.
#[ink::test]
fn new_works() {
    let _ddc_bucket = DdcBucket::new();
}