//! The data structure of Nodes.

use ink_storage::traits::{SpreadAllocate, PackedLayout, SpreadLayout, PackedAllocate};
use scale::{Decode, Encode};
use ink_primitives::Key;
use crate::ddc_bucket::{AccountId, Balance, ClusterId, NodeStatusInCluster, Error::*, Result};
use ink_prelude::string::String;


pub type ProviderId = AccountId;
pub type CdnNodeKey = AccountId;
pub type CdnNodeParams = String;

#[derive(Clone, PartialEq, Encode, Decode, SpreadAllocate, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct CdnNode {
    pub provider_id: ProviderId,
    pub undistributed_payment: Balance,
    pub cdn_node_params: CdnNodeParams,
    pub cluster_id: Option<ClusterId>,
    pub status_in_cluster: Option<NodeStatusInCluster>,
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
pub struct CdnNodeInfo {
    pub cdn_node_key: CdnNodeKey,
    pub cdn_node: CdnNode,
}

pub const CDN_NODE_PARAMS_MAX_LEN: usize = 100_000;

impl CdnNode {

    pub fn new(
        provider_id: AccountId,
        cdn_node_params: CdnNodeParams,
        undistributed_payment: Balance
    ) -> Result<Self> {
        let mut cdn_node = CdnNode {
            provider_id,
            cdn_node_params: CdnNodeParams::default(),
            undistributed_payment,
            cluster_id: None,
            status_in_cluster: None,
        };
        
        cdn_node.set_params(cdn_node_params)?;
        Ok(cdn_node)
    }

    pub fn only_provider(&self, caller: AccountId) -> Result<()> {
        (self.provider_id == caller)
            .then(|| ())
            .ok_or(OnlyCdnNodeProvider)
    }

    pub fn only_without_cluster(&self) -> Result<()> {
        self.cluster_id
            .map_or(Ok(()), |cluster_id| Err(CdnNodeIsAddedToCluster(cluster_id)))
    }

    pub fn only_with_cluster(&self, cluster_id: ClusterId) -> Result<()> {
        self.cluster_id
            .is_some()
            .then(|| ())
            .ok_or(CdnNodeIsNotAddedToCluster(cluster_id))
    }

    pub fn set_params(&mut self, cdn_node_params: CdnNodeParams) -> Result<()> {
        if cdn_node_params.len() > CDN_NODE_PARAMS_MAX_LEN {
            return Err(ParamsSizeExceedsLimit);
        }
        self.cdn_node_params = cdn_node_params;
        Ok(())
    }

    pub fn set_cluster(&mut self, cluster_id: ClusterId, status: NodeStatusInCluster) {
        self.cluster_id = Some(cluster_id);
        self.status_in_cluster = Some(status);
    }

    pub fn unset_cluster(&mut self) {
        self.cluster_id = None;
        self.status_in_cluster = None;
    }

    pub fn change_status_in_cluster(&mut self, status: NodeStatusInCluster) {
        self.status_in_cluster = Some(status);
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