//! The public interface to manage Attachments.

use ink_lang::{EmitEvent, StaticEnv};

use crate::ddc_nft_registry::{Attach, DdcNftRegistry, Result};
use crate::ddc_nft_registry::attachment::entity::{AssetId, Attachment, AttachmentStatus, NftId, Proof};

impl DdcNftRegistry {
    pub fn message_attach(&mut self, nft_id: NftId, asset_id: AssetId, proof: Proof) -> Result<()> {
        let reporter_id = Self::env().caller();
        let attachment = self.attachments.create(reporter_id, nft_id, asset_id, proof)?;
        Self::capture_fee_and_refund(Attachment::RECORD_SIZE)?;
        Self::env().emit_event(Attach { reporter_id, nft_id: attachment.nft_id, asset_id: attachment.asset_id, proof: attachment.proof });
        Ok(())
    }

    pub fn message_get_by_nft_id(&mut self, nft_id: NftId) -> Result<AttachmentStatus> {
        let attachment = self.attachments.get_by_nft_id(nft_id)?;
        Ok(AttachmentStatus { attachment })
    }

    pub fn message_get_by_asset_id(&mut self, asset_id: AssetId) -> Result<AttachmentStatus> {
        let attachment = self.attachments.get_by_asset_id(asset_id)?;
        Ok(AttachmentStatus { attachment })
    }
}
