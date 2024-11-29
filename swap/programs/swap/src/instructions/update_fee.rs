use anchor_lang::prelude::*;
use crate::state::admin::*;

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct UpdateFee<'info> {
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

impl<'info> UpdateFee<'info> {
    pub fn process(&mut self, new_fee: u64) -> Result<()> {
        self.fee_config.fee_amount = new_fee;
        Ok(())
    }
}
