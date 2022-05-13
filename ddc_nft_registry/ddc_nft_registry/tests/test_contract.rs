use ink_lang as ink;

use crate::ddc_nft_registry::*;

use super::env_utils::*;

#[ink::test]
fn new_works() {
    let contract = DdcNftRegistry::new();
    contract.get()?;
}
