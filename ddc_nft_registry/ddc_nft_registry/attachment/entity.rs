//! The data structure of Attachments.

use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};
use ink_prelude::string::String;

use crate::ddc_nft_registry::contract_fee::{SIZE_ACCOUNT_ID, SIZE_INDEX, SIZE_NFT_ID, SIZE_PER_RECORD, SIZE_PROOF};

pub type NftId = String;
pub type AssetId = String;
pub type Proof = String;

#[derive(Clone, PartialEq, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct Attachment {
    pub owner_id: ink_env::AccountId,
    pub nft_id: NftId,
    pub asset_id: AssetId,
    pub proof: Proof,
}

#[derive(Clone, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct AttachmentStatus {
    pub attachment: Attachment,
}

impl Attachment {
    pub const RECORD_SIZE: usize = SIZE_PER_RECORD
        + SIZE_ACCOUNT_ID + SIZE_NFT_ID + SIZE_INDEX + SIZE_PROOF;
}
