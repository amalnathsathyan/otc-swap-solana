use anchor_lang::prelude::*;
use crate::{state::admin::*, SwapError};

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct ModifyMintWhitelist<'info> {
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
       seeds = [b"mint_whitelist"],
       bump,
   )]
   pub mint_whitelist: Account<'info, MintWhitelist>,
   pub system_program: Program<'info, System>,
}

impl<'info> ModifyMintWhitelist<'info> {
   pub fn add_mint(&mut self, mint: Pubkey) -> Result<()> {
       require!(!self.mint_whitelist.mints.contains(&mint), SwapError::MintAlreadyWhitelisted);
       self.mint_whitelist.mints.push(mint);
       Ok(())
   }

   pub fn remove_mint(&mut self, mint: Pubkey) -> Result<()> {
       let position = self.mint_whitelist.mints.iter()
           .position(|x| x == &mint)
           .ok_or(SwapError::MintNotWhitelisted)?;
       self.mint_whitelist.mints.remove(position);
       Ok(())
   }
} 