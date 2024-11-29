use anchor_lang::prelude::*;

#[account]
pub struct MintWhitelist {
    pub mints: Vec<Pubkey>,
}

#[account]
pub struct FeeConfig {
    pub fee_amount: u64,
    pub fee_address: Pubkey,
}

#[account]
pub struct WhitelistConfig {
    pub require_whitelist: bool,
}
