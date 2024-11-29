use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::*;

#[derive(Accounts)]
pub struct ModifyWhitelist<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    #[account(
        mut,
        constraint = whitelist.offer == offer.key()
    )]
    pub whitelist: Account<'info, Whitelist>,
    pub offer: Account<'info, Offer>,
    pub system_program: Program<'info, System>,
}

impl<'info> ModifyWhitelist<'info> {
    pub fn add_taker(&mut self, taker: Pubkey) -> Result<()> {
        require!(!self.whitelist.takers.contains(&taker), SwapError::TakerAlreadyWhitelisted);
        self.whitelist.takers.push(taker);
        Ok(())
    }

    pub fn remove_taker(&mut self, taker: Pubkey) -> Result<()> {
        let position = self.whitelist.takers.iter().position(|x| x == &taker)
            .ok_or(SwapError::TakerNotWhitelisted)?;
        self.whitelist.takers.remove(position);
        Ok(())
    }
}