//! The public interface to manage Protocol (fees included).

use crate::ddc_bucket::{DdcBucket, Result};
use crate::ddc_bucket::cash::{Cash, Payable};
use crate::ddc_bucket::perm::entity::Permission;

impl DdcBucket {

    pub fn message_get_protocol_fee_bp(&self) -> u128 {
        self.protocol.get_protocol_fee_bp()
    }

    pub fn message_set_protocol_fee_bp(&mut self, protocol_fee_bp: u128) -> Result<()> {
        self.only_with_permission(Permission::SuperAdmin)?;
        self.protocol.set_protocol_fee_bp(protocol_fee_bp);
        Ok(())
    }

    pub fn message_get_fee_revenues(&self) -> Cash {
        self.protocol.get_fee_revenues()
    }

    pub fn message_put_fee_revenues(&mut self, amount: Cash) -> Result<()> {
        self.protocol.put_revenues(amount);
        Ok(())
    }  

    pub fn message_withdraw_revenues(&mut self, amount: u128) -> Result<()> {
        self.only_with_permission(Permission::SuperAdmin)?;
        self.protocol.withdraw_revenues(Payable(amount))?;
        Self::send_cash(self.protocol.protocol_fee_destination, Cash(amount))?;
        Ok(())
    }

}