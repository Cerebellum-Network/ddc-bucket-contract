//! This module captures fees for the entire network.

use crate::ddc_bucket::{AccountId, Balance, DdcBucket, Result};
use crate::ddc_bucket::cash::Cash;

/// Network fee on cluster revenues. In basis points (1% of 1%).
const NETWORK_FEE_BP: Balance = 0;
// 1%
const BP: Balance = 10_000; // 100%


impl DdcBucket {
    /// Take a network fee from the given revenues (in place).
    pub fn capture_network_fee(revenues: &mut Cash) -> Result<()> {
        let network_fee = revenues.peek() * NETWORK_FEE_BP / BP;
        let (payable, cash) = Cash::borrow_payable_cash(network_fee);
        revenues.pay(payable)?;
        let to_burn = AccountId::default();
        Self::send_cash(to_burn, cash)
    }
}
