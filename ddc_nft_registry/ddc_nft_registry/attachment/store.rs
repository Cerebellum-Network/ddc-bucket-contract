//! The store to create and access Attachments.

use ink_storage::Mapping;

use crate::ddc_nft_registry::{AccountId, Error, Error::*};
use crate::ddc_nft_registry::attachment::entity::{AssetId, NftId, Proof};

use super::entity::Attachment;

pub const ATTACHMENTS_STORE_KEY: u32 = openbrush::storage_unique_key!(AttachmentStore);
#[openbrush::upgradeable_storage(ATTACHMENTS_STORE_KEY)]
#[derive(Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct AttachmentStore {
    pub attachments: Mapping<NftId, Attachment>,
    _reserved: Option<()>
}

impl AttachmentStore {
    #[must_use]
    pub fn create(&mut self, reporter_id: AccountId, nft_id: NftId, asset_id: AssetId, proof: Proof) -> Result<Attachment, Error> {
        let attachment = Attachment {
            reporter_id,
            nft_id,
            asset_id,
            proof,
        };

        // If exists, check that this is the same reporter.
        if let Some(previous) = self.attachments.get(&attachment.nft_id) {
            if previous.reporter_id != reporter_id {
                return Err(UnauthorizedUpdate);
            }
        }

        self.attachments.insert(attachment.nft_id.clone(), &attachment);
        Ok(attachment)
    }

    pub fn get_by_nft_id(&mut self, nft_id: NftId) -> Result<Attachment, Error> {
        return self.attachments.get(&nft_id).map(|a| a.clone()).ok_or(AttachmentDoesNotExist);
    }
}
