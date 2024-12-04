use anchor_lang::prelude::*;

/// Account structure representing a whitelist of allowed takers for a specific offer
/// Controls which addresses can take (accept) a particular offer
#[account]
pub struct Whitelist {
    /// Maker who controls the whitelist
    pub maker: Pubkey,
    
    /// The public key of the offer this whitelist is associated with
    /// Links the whitelist to its specific offer
    pub offer: Pubkey,

    /// Vector of public keys representing addresses allowed to take the offer
    /// Only these addresses can execute trades against the offer when whitelist is enabled
    /// Empty vector means no takers are currently whitelisted
    pub takers: Vec<Pubkey>,
}