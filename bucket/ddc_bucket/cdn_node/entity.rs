//! The data structure of Nodes.

use ink_storage::traits::{SpreadAllocate, PackedLayout, SpreadLayout, PackedAllocate};
use scale::{Decode, Encode};
use ink_primitives::Key;
use crate::ddc_bucket::{AccountId, Balance, Error::*, Result};
use crate::ddc_bucket::params::store::Params;

pub type ProviderId = AccountId;
pub type NodeId = u32;
pub type CdnNodePublicKey = AccountId;
pub type Resource = u32;

#[derive(Clone, PartialEq, Encode, Decode, SpreadAllocate, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct CdnNode {
    pub provider_id: ProviderId,
    pub undistributed_payment: Balance,
    pub node_pub_key: CdnNodePublicKey,
}

// https://use.ink/3.x/ink-vs-solidity#nested-mappings--custom--advanced-structures
#[allow(unconditional_recursion)]
impl ink_storage::traits::PackedAllocate for CdnNode {
    fn allocate_packed(&mut self, at: &Key) {
        PackedAllocate::allocate_packed(&mut *self, at)
    }
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