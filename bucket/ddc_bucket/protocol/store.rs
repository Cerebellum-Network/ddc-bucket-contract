use crate::ddc_bucket::cash::{Cash, Payable};
use crate::ddc_bucket::currency::CurrencyConverter;
use crate::ddc_bucket::{
    AccountId, Balance, BasisPoints, DdcBucket, Error::*, Result, BASIS_POINTS,
};
use ink_storage::traits::{PackedLayout, SpreadAllocate, SpreadLayout};
use scale::{Decode, Encode};

/// The configuration of fees.
#[derive(Default, Clone, PartialEq, Encode, Decode, SpreadAllocate, SpreadLayout, PackedLayout)]
#[cfg_attr(
    feature = "std",
    derive(ink_storage::traits::StorageLayout, Debug, scale_info::TypeInfo)
)]
pub struct NetworkFeeConfig {
    /// The fee rate from cluster revenues to the overall network. In basis points (1% of 1%).
    pub network_fee_bp: BasisPoints,
    /// The destination account of network fees. Use the 0 account to burn the fees.
    pub network_fee_destination: AccountId,
    /// The fee rate from cluster revenues to the cluster manager. In basis points (1% of 1%).
    pub cluster_management_fee_bp: BasisPoints,
}

impl NetworkFeeConfig {
    pub fn new(
        network_fee_bp: BasisPoints,
        network_fee_destination: AccountId,
        cluster_management_fee_bp: BasisPoints,
    ) -> Self {
        Self {
            network_fee_bp,
            network_fee_destination,
            cluster_management_fee_bp,
        }
    }
}

#[derive(Default, Clone, PartialEq, Encode, Decode, SpreadAllocate, SpreadLayout, PackedLayout)]
#[cfg_attr(
    feature = "std",
    derive(ink_storage::traits::StorageLayout, Debug, scale_info::TypeInfo)
)]
pub struct ProtocolStore {
    pub protocol_fee_bp: BasisPoints,
    pub protocol_fee_destination: AccountId,
    pub revenues: Cash,
    pub curr_converter: CurrencyConverter,
    pub network_fee_config: NetworkFeeConfig,
}

impl ProtocolStore {
    pub fn init(
        &mut self,
        protocol_fee_bp: BasisPoints,
        protocol_fee_dest: AccountId,
        network_fee_bp: BasisPoints,
        network_fee_dest: AccountId,
        cluster_fee_bp: BasisPoints,
    ) {
        self.protocol_fee_bp = protocol_fee_bp;
        self.protocol_fee_destination = protocol_fee_dest;
        self.curr_converter = CurrencyConverter::new();
        self.network_fee_config =
            NetworkFeeConfig::new(network_fee_bp, network_fee_dest, cluster_fee_bp);
    }

    pub fn get_protocol_fee_bp(&self) -> BasisPoints {
        self.protocol_fee_bp
    }

    pub fn set_protocol_fee_bp(&mut self, protocol_fee_bp: BasisPoints) {
        self.protocol_fee_bp = protocol_fee_bp;
    }

    pub fn get_protocol_fee_dest(&self) -> AccountId {
        self.protocol_fee_destination
    }

    pub fn get_revenues(&self) -> Cash {
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

    pub fn get_network_fee_config(&self) -> NetworkFeeConfig {
        self.network_fee_config.clone()
    }

    pub fn set_network_fee_config(&mut self, config: NetworkFeeConfig) {
        self.network_fee_config = config;
    }

    pub fn get_network_fee_bp(&self) -> BasisPoints {
        self.network_fee_config.network_fee_bp
    }

    pub fn get_network_fee_dest(&self) -> AccountId {
        self.network_fee_config.network_fee_destination
    }

    pub fn get_cluster_management_fee_bp(&self) -> BasisPoints {
        self.network_fee_config.cluster_management_fee_bp
    }
}

impl DdcBucket {
    /// Take a network fee from the given revenues (in place).
    pub fn capture_network_fee(&mut self, revenues: &mut Cash) -> Result<()> {
        self.capture_fee(
            self.protocol.get_network_fee_bp(),
            self.protocol.get_network_fee_dest(),
            revenues,
        )
    }

    /// Take a fee from the given revenues (in place) and send it to the destination.
    pub fn capture_fee(
        &self,
        rate_bp: Balance,
        destination: AccountId,
        revenues: &mut Cash,
    ) -> Result<()> {
        let fee = revenues.peek() * rate_bp / BASIS_POINTS;
        let (payable, cash) = Cash::borrow_payable_cash(fee);
        revenues.pay(payable)?;
        Self::send_cash(destination, cash)
    }
}
