//! The data structure of Clusters.

use ink_prelude::vec::Vec;
use ink_storage::traits::{SpreadAllocate, SpreadLayout, PackedLayout, PackedAllocate};
use scale::{Decode, Encode};
use ink_primitives::Key;
use crate::ddc_bucket::{AccountId, Balance, Result};
use crate::ddc_bucket::cash::{Cash, Payable};
use crate::ddc_bucket::Error::{UnauthorizedClusterManager, InsufficientBalance};
use crate::ddc_bucket::cdn_node::entity::{Resource, CdnNodeKey};
use crate::ddc_bucket::params::store::Params;

pub type ClusterId = u32;
pub type ClusterParams = Params;

#[derive(Clone, PartialEq, Encode, Decode, SpreadAllocate, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct CdnCluster {
    pub manager_id: AccountId,
    pub cdn_nodes: Vec<CdnNodeKey>,
    pub resources_used: Resource,
    pub revenues: Cash,
    pub usd_per_gb: Balance,
}

// https://use.ink/3.x/ink-vs-solidity#nested-mappings--custom--advanced-structures
#[allow(unconditional_recursion)]
impl ink_storage::traits::PackedAllocate for CdnCluster {
    fn allocate_packed(&mut self, at: &Key) {
        PackedAllocate::allocate_packed(&mut *self, at)
    }
}

#[derive(Clone, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct CdnClusterStatus {
    pub cluster_id: ClusterId,
    pub cluster: CdnCluster,
}

impl CdnCluster {
    pub fn new(
        manager_id: AccountId,
        cdn_nodes: Vec<CdnNodeKey>,
    ) -> Self {
        CdnCluster {
            manager_id,
            cdn_nodes,
            // usd_per_gb: 100_000_000, // setting initially to 1 cent per GB
            usd_per_gb: 104_857_600, // setting initially to 1 cent per GB
            resources_used: 0,
            revenues: Cash(0),
        }
    }

    pub fn get_revenue_cere(&self) -> Cash {
        self.revenues
    }

    pub fn set_rate(&mut self, usd_per_gb: Balance) -> Result<()> {
        self.usd_per_gb = usd_per_gb;
        Ok(())
    }

    pub fn get_rate(&self) -> Balance {
        self.usd_per_gb
    }

    pub fn put_revenues(&mut self, amount: Cash) {
        self.revenues.increase(amount);
    }

    pub fn take_revenues(&mut self, amount: Payable) -> Result<()> {
        if amount.peek() > self.revenues.peek() {
            return Err(InsufficientBalance);
        }
        self.revenues.pay_unchecked(amount);
        Ok(())
    }

    pub fn only_manager(&self, caller: AccountId) -> Result<()> {
        if self.manager_id == caller { Ok(()) } else { Err(UnauthorizedClusterManager) }
    }
}
