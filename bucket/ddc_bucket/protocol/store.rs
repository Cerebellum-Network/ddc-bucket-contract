use crate::ddc_bucket::{AccountId, Error::*, Result};
use crate::ddc_bucket::cash::{Cash, Payable};


#[ink::storage_item]
#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ProtocolStore { 
  pub admin: AccountId,
  pub fee_bp: u32,
  pub revenues: Cash,
}

impl ProtocolStore {
  pub fn new(
      admin: AccountId,
      fee_bp: u32,
  ) -> Self {
    ProtocolStore {
      admin,
      fee_bp,
      revenues: Cash(0),
    }
  }

  pub fn get_fee_bp(&self) -> u32 {
      self.fee_bp
  }

  pub fn set_fee_bp(&mut self, fee_bp: u32) -> Result<()> {
      self.fee_bp = fee_bp;
      Ok(())
  }

  pub fn get_fee_revenues(&self) -> Cash {
      self.revenues
  }

  pub fn put_revenues(&mut self, amount: Cash) {
      self.revenues.increase(amount);
  }

  pub fn withdraw_revenues(&mut self, amount: Payable) -> Result<()> {
      if amount.peek() > self.revenues.peek() {
          return Err(InsufficientBalance);
      }
      self.revenues.pay_unchecked(amount);
      Ok(())
  }
}