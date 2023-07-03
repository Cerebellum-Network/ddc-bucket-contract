//! The public interface to manage Protocol (fees included).

use crate::ddc_bucket::{DdcBucket, NetworkFeeConfig, BasisPoints};
use crate::ddc_bucket::cash::Cash;

impl DdcBucket {

    pub fn message_get_protocol_fee_bp(&self) -> BasisPoints {
        self.protocol.get_protocol_fee_bp()
    }

    pub fn message_get_network_fee_config(&self) -> NetworkFeeConfig {
        self.protocol.get_network_fee_config()
    }
    
    pub fn message_get_protocol_revenues(&self) -> Cash {
        self.protocol.get_revenues()
    }

}