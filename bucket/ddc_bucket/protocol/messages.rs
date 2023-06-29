//! The public interface to manage Protocol (fees included).

use crate::ddc_bucket::{DdcBucket, Result};
use crate::ddc_bucket::cash::{Cash, Payable};

impl DdcBucket {

    pub fn message_get_fee_bp(&self) -> u32 {
        self.protocol.get_fee_bp()
    }

    pub fn message_set_fee_bp(&mut self, fee_bp: u32) -> Result<()> {
        match self.protocol.set_fee_bp(fee_bp) {
            Err(_e) => panic!("Setting fee failed"), 
            Ok(_v) => Ok(()),
        }
    }

    pub fn message_get_fee_revenues(&self) -> Cash {
        self.protocol.get_fee_revenues()
    }

    pub fn message_put_fee_revenues(&mut self, amount: Cash) -> Result<()> {
        self.protocol.put_revenues(amount);

        Ok(())
    }  

    pub fn message_withdraw_revenues(&mut self, amount: u128) -> Result<()> {
        self.protocol.withdraw_revenues(Payable(amount))?;

        Self::send_cash(self.protocol.admin, Cash(amount))?;

        Ok(())
    }
    
}