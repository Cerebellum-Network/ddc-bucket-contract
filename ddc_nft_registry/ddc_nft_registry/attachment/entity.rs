//! The data structure of Attachments.

use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};
use ink_prelude::string::String;
use crate::ddc_nft_registry::contract_fee::{SIZE_ACCOUNT_ID, SIZE_HASHMAP, SIZE_PER_RECORD, SIZE_VEC};
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

impl Attachment {
    pub fn record_size(&self) -> usize {
        return SIZE_PER_RECORD + SIZE_HASHMAP // Map overhead
            + SIZE_VEC // Key string
            + SIZE_VEC * 3 // Value strings
            + SIZE_ACCOUNT_ID // Reporter ID
            + self.nft_id.len() * 2 // NFT ID in key and in value
            + self.asset_id.len() // Asset ID
            + self.proof.len(); // Proof data
    }
}

#[derive(Clone, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo))]
pub struct AttachmentStatus {
    pub attachment: Attachment,
}
