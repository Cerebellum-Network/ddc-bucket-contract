//! The data structure of Nodes.

use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};

use crate::ddc_bucket::{AccountId, Balance, Error::*, Result};
use crate::ddc_bucket::params::store::Params;

pub type ProviderId = AccountId;
pub type NodeId = u32;
pub type Resource = u32;

#[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct CdnNode {
    pub provider_id: ProviderId,
    pub undistributed_payment: Balance,
}

#[derive(Clone, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct CdnNodeStatus {
    pub node_id: NodeId,
    pub node: CdnNode,
    pub params: Params,
}

impl CdnNode {
    pub fn cdn_account_id(&self) -> AccountId {
        self.provider_id
    }

    pub fn only_owner(&self, provider_id: AccountId) -> Result<()> {
        if self.provider_id == provider_id { Ok(()) } else { Err(UnauthorizedProvider) }
    }

    pub fn put_payment(&mut self, amount: Balance) {
        self.undistributed_payment += amount;
    }

    pub fn take_payment(&mut self, amount: Balance) -> Result<()> {
        if self.undistributed_payment >= amount {
            self.undistributed_payment -= amount;
            Ok(())
        } else {
            Err(InsufficientBalance)
        }
    }
}