//! The store to create and access Attachments.

use ink_storage::Mapping;
use ink_storage::traits::{SpreadAllocate, SpreadLayout, StorageLayout};

use crate::ddc_nft_registry::{AccountId, Error, Error::*};
use crate::ddc_nft_registry::attachment::entity::{AssetId, NftId, Proof};

use super::entity::Attachment;

#[derive(SpreadAllocate, SpreadLayout, Default)]
#[cfg_attr(feature = "std", derive(StorageLayout, Debug))]
pub struct AttachmentStore(pub Mapping<NftId, Attachment>);

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
        if let Some(previous) = self.0.get(&attachment.nft_id) {
            if previous.reporter_id != reporter_id {
                return Err(UnauthorizedUpdate);
            }
        }

        self.0.insert(attachment.nft_id.clone(), &attachment);
        Ok(attachment)
    }

    pub fn get_by_nft_id(&mut self, nft_id: NftId) -> Result<Attachment, Error> {
        return self.0.get(&nft_id).map(|a| a.clone()).ok_or(AttachmentDoesNotExist);
    }
}
