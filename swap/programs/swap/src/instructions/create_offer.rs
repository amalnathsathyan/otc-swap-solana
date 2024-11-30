use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};
use crate::state::*;

#[derive(Accounts)]
#[instruction(offer_id: u64, bump: u8)]
pub struct CreateOffer<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    #[account(
        init,
        payer = maker,
        space = 8 + 32 + 32 + 8 + 8 + 8 + 1,
        seeds = [
            b"offer",
            maker.key().as_ref(),
            offer_id.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub offer: Account<'info, Offer>,
    pub token_mint: Account<'info, token::Mint>,
    #[account(mut)]
    pub maker_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,
    /// CHECK: PDA for vault authority
    #[account(
        seeds = [b"vault", token_mint.key().as_ref()],
        bump
    )]
    pub vault_authority: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateOffer<'info> {
    pub fn process(
        &mut self,
        id: u64,
        token_mint: Pubkey,
        token_amount: u64,
        expected_total_amount: u64,
        deadline: i64,
        bump: u8,
    ) -> Result<()> {
        let offer = &mut self.offer;
        offer.offer_id = id;
        offer.maker = self.maker.key();
        offer.token_mint = token_mint;
        offer.token_amount = token_amount;
        offer.expected_total_amount = expected_total_amount;
        offer.deadline = deadline;
        offer.status = OfferStatus::Ongoing;
        offer.bump = bump;

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