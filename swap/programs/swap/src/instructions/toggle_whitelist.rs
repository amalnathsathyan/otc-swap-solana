use anchor_lang::prelude::*;
use crate::state::admin::*;

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct ToggleRequireWhitelist<'info> {
    #[account(
        mut,
        seeds = [b"authority"],
        bump,
        constraint = authority.key() == admin.key()
    )]
    /// CHECK: PDA check
    pub authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [b"whitelist_config"],
        bump,
    )]
    pub whitelist_config: Account<'info, WhitelistConfig>,
    pub system_program: Program<'info, System>,
}

impl<'info> ToggleRequireWhitelist<'info> {
    pub fn process(&mut self) -> Result<()> {
        self.whitelist_config.require_whitelist = !self.whitelist_config.require_whitelist;
        Ok(())
    }
}