use ink_lang::StaticEnv;

use crate::ddc_bucket::{Balance, DdcBucket, Result, TOKEN};
use crate::ddc_bucket::cash::Payable;

pub const FEE_PER_BYTE: Balance = TOKEN / 100;

/// The minimum size assumed for a new object to cover the size of the ID or storage key.
/// In addition to the size of the object itself.
pub const SIZE_PER_RECORD: usize = 32;

impl DdcBucket {
    pub fn capture_fee_and_refund(record_size: usize) -> Result<()> {
        let mut received_value = Self::receive_cash();
        let fee = calculate_contract_fee(record_size);
        received_value.pay(fee)?;
        let caller = Self::env().caller();
        Self::send_cash(caller, received_value)
    }
}

fn calculate_contract_fee(record_size: usize) -> Payable {
    Payable::new(FEE_PER_BYTE * record_size as Balance)
}
