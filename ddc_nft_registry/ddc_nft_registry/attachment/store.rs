//! The store to create and access Attachments.

use ink_storage::{
    collections::Vec as InkVec,
    traits,
};
use crate::ddc_nft_registry::attachment::entity::{AssetId, NftId, Proof};
use crate::ddc_nft_registry::{Error};
use crate::ddc_nft_registry::Error::{AttachmentDoesNotExist, AttachmentIntegrityError};

use super::entity::{Attachment};

#[derive(traits::SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(traits::StorageLayout, Debug))]
pub struct AttachmentStore(pub InkVec<Attachment>);

impl AttachmentStore {
    #[must_use]
    pub fn create(&mut self, owner_id: ink_env::AccountId, nft_id: NftId, asset_id: AssetId, proof: Proof) -> Result<Attachment, Error> {
        let attachment = Attachment {
            owner_id,
            nft_id,
            asset_id,
            proof
        };
        if !self.verify_integrity(&attachment) {
            return Err(AttachmentIntegrityError)
        }
        self.0.push(attachment.clone());
        Ok(attachment)
    }

    pub fn get_by_nft_id(&self, nft_id: NftId) -> Result<Attachment, Error> {
        self.0.iter().find(|&a| a.nft_id == nft_id).map(|a| a.clone()).ok_or(AttachmentDoesNotExist)
    }

    pub fn get_by_asset_id(&self, asset_id: AssetId) -> Result<Attachment, Error> {
        self.0.iter().find(|&a| a.asset_id == asset_id).map(|a| a.clone()).ok_or(AttachmentDoesNotExist)
    }

    fn verify_integrity(&self, attachment: &Attachment) -> bool {
        self.0.iter().all(|a| a.nft_id != attachment.nft_id
            && a.asset_id != attachment.asset_id)
    }
}
