use anchor_lang::prelude::*;

#[account]
pub struct Offer {
    pub offer_id: u64,
    pub maker: Pubkey,
    pub token_mint: Pubkey,
    pub token_amount: u64,
    pub expected_total_amount: u64,
    pub deadline: i64,
    pub status: OfferStatus,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]

pub enum OfferStatus {
    Ongoing,
    Completed,
    Cancelled,
    Expired,
}