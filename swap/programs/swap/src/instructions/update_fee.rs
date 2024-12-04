use anchor_lang::prelude::*;
use crate::state::admin::*;

/// Account structure for updating the protocol fee amount
/// Takes a bump seed as an instruction argument for PDA validation
#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct UpdateFee<'info> {
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

    /// The admin account that can update the fee amount
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

    /// Updates the protocol fee amount
    /// 
    /// # Arguments
    /// * `new_fee` - The new fee amount in basis points (1/100th of a percent)
    ///               e.g., 100 = 1%, 50 = 0.5%, 25 = 0.25%
    ///
    /// # Function Operation
    /// Updates the fee_amount in fee_config with the provided new_fee value
    ///
    /// # Returns
    /// * `Result<()>` - Ok if the fee amount was successfully updated
    pub fn fee_update(ctx: Context<UpdateFee>, new_fee: u64) -> Result<()> {
        // Update the fee amount in basis points
        ctx.accounts.fee_config.fee_percentage = new_fee;
        Ok(())
    }
