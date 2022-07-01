//! The store to create and access Attachments.

use ink_storage::{
    collections::HashMap as InkHashMap,
    traits,
};
use crate::ddc_nft_registry::attachment::entity::{AssetId, NftId, Proof};
use crate::ddc_nft_registry::{Error};
use crate::ddc_nft_registry::Error::*;

use super::entity::{Attachment};

#[derive(traits::SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout, Debug))]
pub struct AttachmentStore(pub InkHashMap<NftId, Attachment>);

impl AttachmentStore {
    #[must_use]
    pub fn create(&mut self, owner_id: ink_env::AccountId, nft_id: NftId, asset_id: AssetId, proof: Proof) -> Result<Attachment, Error> {
        let attachment = Attachment {
            owner_id,
            nft_id,
            asset_id,
            proof
        };

        // If exists, check that this is the same reporter.
        if let Some(previous) = self.0.get(&attachment.nft_id) {
            if previous.owner_id != owner_id {
                return Err(UnauthorizedUpdate);
            }
        }

        self.0.insert(attachment.nft_id.clone(), attachment.clone());
        Ok(attachment)
    }

    pub fn get_by_nft_id(&mut self, nft_id: NftId) -> Result<Attachment, Error> {
        return self.0.get(&nft_id).map(|a| a.clone()).ok_or(AttachmentDoesNotExist);
    }

}
