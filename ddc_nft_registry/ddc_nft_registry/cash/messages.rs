use ink_lang::StaticEnv;

use crate::ddc_nft_registry::{AccountId, DdcNftRegistry, Result};

use super::entity::Cash;

impl DdcNftRegistry {
    pub fn receive_cash() -> Cash {
        Cash(Self::env().transferred_balance())
    }

    pub fn send_cash(destination: AccountId, cash: Cash) -> Result<()> {
        if cash.peek() == 0 { return Ok(()); }
        match Self::env().transfer(destination, cash.consume()) {
            Err(_e) => panic!("Transfer failed"), // Err(Error::TransferFailed),
            Ok(_v) => Ok(()),
        }
    }
}
