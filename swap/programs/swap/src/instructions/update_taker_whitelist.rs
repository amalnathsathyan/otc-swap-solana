use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::*;

/// Account structure for updating offer whitelist
#[derive(Accounts)]
pub struct UpdateWhitelist<'info> {
    /// The offer creator who controls the whitelist
    #[account(mut)]
    pub maker: Signer<'info>,
    
    /// The whitelist PDA to be modified
    /// Only modifiable by the original maker
    #[account(
        mut,
        constraint = whitelist.maker == maker.key() @ SwapError::UnauthorizedMaker
    )]
    pub whitelist: Account<'info, Whitelist>,
}

/// Adds multiple taker addresses to an offer's whitelist
///
/// # Arguments
/// * `ctx` - UpdateWhitelist context
/// * `new_takers` - Vector of addresses to add to whitelist
///
/// # Security
/// - Only callable by offer maker
/// - Automatically deduplicates addresses
/// - Maximum of 50 takers enforced by account space
pub fn add_takers(ctx: Context<UpdateWhitelist>, new_takers: Vec<Pubkey>) -> Result<()> {
    for taker in new_takers {
        if !ctx.accounts.whitelist.takers.contains(&taker) {
            ctx.accounts.whitelist.takers.push(taker);
        }
    }
    Ok(())
}

/// Removes multiple taker addresses from an offer's whitelist
///
/// # Arguments
/// * `ctx` - UpdateWhitelist context
/// * `remove_takers` - Vector of addresses to remove from whitelist
///
/// # Security
/// - Only callable by offer maker
/// - Safely handles removal of non-existent addresses
pub fn remove_takers(ctx: Context<UpdateWhitelist>, remove_takers: Vec<Pubkey>) -> Result<()> {
    ctx.accounts.whitelist.takers.retain(|taker| !remove_takers.contains(taker));
    Ok(())
}