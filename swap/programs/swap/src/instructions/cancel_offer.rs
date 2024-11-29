use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};
use crate::state::*;
use crate::error::*;

#[derive(Accounts)]
pub struct CancelOffer<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    #[account(mut)]
    pub maker_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = offer.maker == maker.key(),
        constraint = offer.status == OfferStatus::Ongoing
    )]
    pub offer: Account<'info, Offer>,
    /// CHECK: PDA for vault authority
    #[account(
        seeds = [b"vault"],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

impl<'info> CancelOffer<'info> {
    pub fn process(&mut self) -> Result<()> {
        let offer = &mut self.offer;
        require!(offer.status == OfferStatus::Ongoing, SwapError::InvalidOfferStatus);
        
        token::transfer(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                token::Transfer {
                    from: self.vault_token_account.to_account_info(),
                    to: self.maker_token_account.to_account_info(),
                    authority: self.vault_authority.to_account_info(),
                },
                &[&[b"vault"]],
            ),
            offer.token_amount,
        )?;

        offer.status = OfferStatus::Cancelled;
        Ok(())
    }
}
