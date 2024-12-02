use anchor_lang::prelude::*;
use crate::state::admin::*;

/// Account structure for toggling the global whitelist requirement setting
/// Takes a bump seed as an instruction argument for PDA validation
#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct ToggleRequireWhitelist<'info> {
    /// The authority PDA that controls admin operations
    /// Must match the admin's public key for security
    /// Seeds: ["authority"]
    #[account(
        mut,
        seeds = [b"authority"],
        bump,
        constraint = authority.key() == admin.key()
    )]
    /// CHECK: PDA validation is handled by seeds and bump
    pub authority: UncheckedAccount<'info>,

    /// The admin account that can toggle the whitelist requirement
    /// Must be the signer of the transaction
    #[account(mut)]
    pub admin: Signer<'info>,

    /// The configuration account storing the whitelist requirement setting
    /// PDA with seeds: ["whitelist_config"]
    #[account(
        mut,
        seeds = [b"whitelist_config"],
        bump,
    )]
    pub whitelist_config: Account<'info, WhitelistConfig>,

    /// The Solana System Program
    pub system_program: Program<'info, System>,
}

impl<'info> ToggleRequireWhitelist<'info> {
    /// Toggles the global whitelist requirement setting
    /// 
    /// # Function Operation
    /// Inverts the current value of require_whitelist:
    /// - If currently true, sets to false
    /// - If currently false, sets to true
    ///
    /// # Returns
    /// * `Result<()>` - Ok if the toggle was successful
    pub fn process(&mut self) -> Result<()> {
        // Toggle the whitelist requirement flag
        self.whitelist_config.require_whitelist = !self.whitelist_config.require_whitelist;
        Ok(())
    }
}