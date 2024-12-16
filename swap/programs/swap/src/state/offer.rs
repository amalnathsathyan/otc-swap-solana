use anchor_lang::prelude::*;

/// Account structure representing a token swap offer
/// Stores all details about an offer including its current state and parameters
#[account]
pub struct Offer {
    /// Unique identifier for the offer
    pub offer_id: u64,

    /// The public key of the account that created this offer
    /// Controls permissions for offer modification and cancellation
    pub maker: Pubkey,

    /// The mint address of the token being offered
    /// Must be a whitelisted token mint if whitelist is enabled
    pub input_token_mint: Pubkey,

    /// The amount of tokens being offered
    /// Decreases as partial fills occur, reaches 0 when fully filled
    pub token_amount: u64,

    /// The mint address of the token being offered
    /// Must be a whitelisted token mint if whitelist is enabled
    pub output_token_mint: Pubkey,

    /// The total amount of payment tokens expected in return
    /// Used to calculate the exchange rate for partial fills
    pub expected_total_amount: u64,

    ///token amount_a remaining after each trade
    pub token_amount_remaining: u64,

    /// token_b fullfilled
    pub expected_fulfilled_amount: u64,

    /// Unix timestamp when this offer expires
    /// Offer cannot be taken after this time
    pub deadline: i64,

    /// Current status of the offer
    /// Controls what operations are permitted
    pub status: OfferStatus,

    pub fee_percentage: u64,        
    pub fee_wallet: Pubkey, 
}

/// Enum representing the possible states of an offer
/// Used to track the offer's lifecycle and control permitted operations
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum OfferStatus {
    /// Initial state after creation
    Initialized,
    
    /// State after vault setup and token transfer
    VaultInitialized,

    /// Offer is active and can be taken
    Ongoing,
    
    /// Offer has been fully filled
    /// All tokens have been exchanged
    Completed,
    
    /// Offer was cancelled by the maker
    /// Remaining tokens were returned
    Cancelled,
    
    /// Offer reached its deadline without being fully filled
    /// No further actions can be taken
    Expired,
}