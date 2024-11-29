use anchor_lang::prelude::*;
use crate::state::admin::*;

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct UpdateFeeAddress<'info> {
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
        seeds = [b"fee"],
        bump,
    )]
    pub fee_config: Account<'info, FeeConfig>,
    pub system_program: Program<'info, System>,
}

impl<'info> UpdateFeeAddress<'info> {
    pub fn process(&mut self, new_address: Pubkey) -> Result<()> {
        self.fee_config.fee_address = new_address;
        Ok(())
    }
}