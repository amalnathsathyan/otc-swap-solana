use anchor_lang::prelude::*;
use crate::state::admin::*;

/// Account structure for updating the fee recipient address
/// Takes a bump seed as an instruction argument for PDA validation
#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct UpdateFeeAddress<'info> {
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

    /// The admin account that can update the fee address
    /// Must be the signer of the transaction
    #[account(mut)]
    pub admin: Signer<'info>,

    /// The configuration account storing fee-related settings
    /// PDA with seeds: ["fee"]
    #[account(
        mut,
        seeds = [b"fee"],
        bump,
    )]
    pub fee_config: Account<'info, FeeConfig>,

    /// The Solana System Program
    pub system_program: Program<'info, System>,
}

    /// Updates the address that receives protocol fees
    /// 
    /// # Arguments
    /// * `new_address` - The new public key that will receive protocol fees
    ///
    /// # Function Operation
    /// Replaces the current fee_address in fee_config with the provided new_address
    ///
    /// # Returns
    /// * `Result<()>` - Ok if the fee address was successfully updated
    pub fn fee_address_update(ctx: Context<UpdateFeeAddress>, new_address: Pubkey) -> Result<()> {
        // Update the fee recipient address
        ctx.accounts.fee_config.fee_address = new_address;
        Ok(())
    }
