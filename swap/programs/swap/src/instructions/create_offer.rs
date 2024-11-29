use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};
use crate::state::*;

#[derive(Accounts)]
pub struct CreateOffer<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    #[account(
        init,
        payer = maker,
        space = 8 + 32 + 32 + 8 + 8 + 8 + 1
    )]
    pub offer: Account<'info, Offer>,
    #[account(mut)]
    pub maker_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,
    /// CHECK: PDA for vault authority
    #[account(
        seeds = [b"vault"],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateOffer<'info> {
    pub fn process(
        &mut self,
        token_mint: Pubkey,
        token_amount: u64,
        expected_total_amount: u64,
        deadline: i64,
    ) -> Result<()> {
        let offer = &mut self.offer;
        offer.maker = self.maker.key();
        offer.token_mint = token_mint;
        offer.token_amount = token_amount;
        offer.expected_total_amount = expected_total_amount;
        offer.deadline = deadline;
        offer.status = OfferStatus::Ongoing;

        token::transfer(
            CpiContext::new(
                self.token_program.to_account_info(),
                token::Transfer {
                    from: self.maker_token_account.to_account_info(),
                    to: self.vault_token_account.to_account_info(),
                    authority: self.maker.to_account_info(),
                },
            ),
            token_amount,
        )?;

        Ok(())
    }
}