use anchor_lang::prelude::*;

#[error_code]
pub enum SwapError {
    #[msg("Invalid Offer Status")]
    InvalidOfferStatus,
    #[msg("Taker not whitelisted")]
    TakerNotWhitelisted,
    #[msg("Invalid Maker")]
    InvalidMaker,
    #[msg("Taker already exists")]
    TakerAlreadyWhitelisted,
    #[msg("Token already whitelisted")]
    MintAlreadyWhitelisted,
    #[msg("Token not whitelisted")]
    MintNotWhitelisted,
    #[msg("Invalid Admin")]
    InvalidAdmin,
}