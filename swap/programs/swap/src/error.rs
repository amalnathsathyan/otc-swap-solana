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
    #[msg("Offer has expired")]
    OfferExpired,
    #[msg("Insufficient amount available")]
    InsufficientAmount,
    #[msg("Cannot Cancel Offer")]
    CannotCancelOffer,
    #[msg("Invalid Maker")]
    UnauthorizedMaker,
    #[msg("Unauthorized admin")]
    UnauthorizedAdmin,
    #[msg("Offer has not expired yet")]
    OfferNotExpired,
    #[msg("Maximum number of whitelisted mints reached")]
    TooManyMints,
    #[msg("Fee percentage cannot exceed 100%")]
    InvalidFeePercentage,
    #[msg("Admin has not been initialized")]
    AdminNotInitialized,
    #[msg("Fee configuration has not been initialized")]
    FeeConfigNotInitialized,
    #[msg("Zero Address not allowed")]
    InvalidAddress,
    #[msg("Invalid token account owner")]
    InvalidTokenAccount,
    #[msg("Token mint mismatch")]
    InvalidTokenMint,
    #[msg("Deadline must be in the future")]
    InvalidDeadline,
    #[msg("Whitelist is full")]
    WhitelistFull,
    #[msg("Takers list cannot be empty")]
    EmptyTakersList,
    #[msg("Token amount must be greater than 0")]
    InvalidAmount,
    #[msg("Sequence Overflow")]
    SequenceOverflow,
    #[msg("Calculation error occurred.")]
    CalculationError,
    #[msg("Invalid vault owner")]
    InvalidVaultOwner,
}