use ink_prelude::{
    string::String,
    vec::Vec,
};
use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};

use crate::ddc_bucket::{Balance, ServiceId};

pub type ClusterId = u32;
pub type ClusterParams = String;

#[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct Cluster {
    pub cluster_id: ClusterId,
    pub rent_per_month: Balance,
    pub cluster_params: ClusterParams,
    pub service_ids: Vec<ServiceId>,
}
