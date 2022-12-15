//! The data structure of Clusters.

use scale::{Decode, Encode};
use ink_storage::traits::{PackedLayout, SpreadLayout};

pub type CdnNodeId = u32;
pub type TierId = u32;

#[derive(Clone, PartialEq, Encode, Decode, PackedLayout, SpreadLayout, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct CdnNodeTier {
    pub cdn_node_id: CdnNodeId,
    pub tier_id: TierId,
}