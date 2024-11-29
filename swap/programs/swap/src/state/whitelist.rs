use anchor_lang::prelude::*;

#[account]
pub struct Whitelist {
    pub offer: Pubkey,
    pub takers: Vec<Pubkey>,
}