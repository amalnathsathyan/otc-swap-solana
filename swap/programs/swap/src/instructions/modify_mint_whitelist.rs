use anchor_lang::prelude::*;
use crate::{state::admin::*, SwapError};

/// Account structure for modifying the whitelist of permitted token mints
/// Takes a bump seed as an instruction argument for PDA validation
#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct ModifyMintWhitelist<'info> {
    /// The authority PDA that controls whitelist modifications
    /// Must match the admin's public key for security
    /// Seeds: ["authority"]
    #[account(
       mut,
       seeds = [b"authority"],
       bump,
       constraint = authority.key() == admin.key()
    )]
    /// CHECK: PDA check
    pub authority: UncheckedAccount<'info>,

    /// The admin account that can modify the whitelist
    /// Must be the signer of the transaction
    #[account(mut)]
    pub admin: Signer<'info>,

    // The account storing the whitelist of permitted token mints
    /// PDA with seeds: ["mint_whitelist"]
    #[account(
       mut,
       seeds = [b"mint_whitelist"],
       bump,
    )]
    pub mint_whitelist: Account<'info, MintWhitelist>,

    /// The Solana System Program
    pub system_program: Program<'info, System>,
}

impl<'info> ModifyMintWhitelist<'info> {
    /// Adds a new token mint to the whitelist
    ///
    /// # Arguments
    /// * `mint` - The public key of the token mint to add
    ///
    /// # Errors
    /// Returns SwapError::MintAlreadyWhitelisted if the mint is already in the whitelist
    ///
    /// # Returns
    /// * `Result<()>` - Ok if the mint was successfully added
    pub fn add_mint(&mut self, mint: Pubkey) -> Result<()> {
        
        // Check if mint already exists in whitelist
        require!(!self.mint_whitelist.mints.contains(&mint), SwapError::MintAlreadyWhitelisted);

        // Add the new mint to the whitelist
        self.mint_whitelist.mints.push(mint);
        Ok(())
    }

    /// Removes a token mint from the whitelist
    ///
    /// # Arguments
    /// * `mint` - The public key of the token mint to remove
    ///
    /// # Errors
    /// Returns SwapError::MintNotWhitelisted if the mint is not found in the whitelist
    ///
    /// # Returns
    /// * `Result<()>` - Ok if the mint was successfully removed
    pub fn remove_mint(&mut self, mint: Pubkey) -> Result<()> {

        // Find the position of the mint in the whitelist  
        let position = self.mint_whitelist.mints.iter()
           .position(|x| x == &mint)
           .ok_or(SwapError::MintNotWhitelisted)?;

        // Remove the mint from the whitelist
        self.mint_whitelist.mints.remove(position);
        Ok(())
    }
} 