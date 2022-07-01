//! The data structure of Attachments.

use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};
use ink_prelude::string::String;
use crate::ddc_nft_registry::AccountId;

pub type NftId = String;
pub type AssetId = String;
pub type Proof = String;

#[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct Attachment {
    pub reporter_id: AccountId,
    pub nft_id: NftId,
    pub asset_id: AssetId,
    pub proof: Proof,
}

#[derive(Clone, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct AttachmentStatus {
    pub attachment: Attachment,
}
