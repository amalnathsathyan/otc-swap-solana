use anchor_lang::prelude::*;

/// Account structure storing the whitelist of permitted token mints
/// This controls which tokens can be used in the protocol
#[account]
pub struct MintWhitelist {
    /// Vector of public keys representing allowed token mints
    /// Only tokens from these mints can be used in the protocol when whitelist is enabled
    pub mints: Vec<Pubkey>,
}

/// Account structure storing fee configuration for the protocol
/// Controls both the fee amount and where fees are sent
#[account]
pub struct FeeConfig {
    /// The fee amount in basis points (1/100th of a percent)
    /// e.g., 100 = 1%, 50 = 0.5%, 25 = 0.25%
    pub fee_percentage: u64,

    /// The public key of the account that receives protocol fees
    /// All fees collected from trades will be sent to this address
    pub fee_address: Pubkey,
}

/// Account structure controlling whether token mint whitelist is enforced
/// Provides global toggle for whitelist functionality
#[account]
pub struct WhitelistConfig {
    /// Boolean flag indicating if whitelist checking is required
    /// true = only whitelisted token mints can be used
    /// false = any token mint can be used
    pub require_whitelist: bool,
}

/// Configuration account for admin operations and offer tracking
#[account]
pub struct AdminConfig {
    /// Admin's public key for authorization
    pub admin: Pubkey,
}

