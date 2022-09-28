//! The data structure of Clusters.

use ink_prelude::vec::Vec;
use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};

use crate::ddc_bucket::{AccountId, NodeId, Result};
use crate::ddc_bucket::cash::{Cash, Payable};
use crate::ddc_bucket::contract_fee::{SIZE_ACCOUNT_ID, SIZE_BALANCE, SIZE_PER_RECORD, SIZE_RESOURCE, SIZE_VEC};
use crate::ddc_bucket::Error::{UnauthorizedClusterManager, InsufficientBalance};
use crate::ddc_bucket::cdn_node::entity::Resource;
use crate::ddc_bucket::params::store::Params;

pub type ClusterId = u32;
pub type ClusterParams = Params;
pub type VNodeIndex = u32;
pub type VNodeId = (ClusterId, VNodeIndex);

#[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct CdnCluster {
    pub manager_id: AccountId,
    pub cdn_nodes: Vec<NodeId>,
    pub resources_used: Resource,
    pub revenues: Cash,
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
        cdn_nodes: Vec<NodeId>,
    ) -> Self {
        CdnCluster {
            manager_id,
            cdn_nodes,
            resources_used: 0,
            revenues: Cash(0),
        }
    }

    pub fn new_size(&self) -> usize {
        SIZE_PER_RECORD
            + SIZE_ACCOUNT_ID
            + SIZE_VEC 
            + SIZE_RESOURCE
            + SIZE_BALANCE
        // Or to be more precise:    SIZE_PER_RECORD + self.encoded_size()
    }

    pub fn get_revenue_cere(&self) -> Cash {
        self.revenues
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

pub const USD_PER_GB: u128 = 1;