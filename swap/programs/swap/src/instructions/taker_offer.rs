use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};
use crate::state::*;
use crate::error::*;

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct TakeOffer<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,
    
    #[account(
        mut,
        constraint = offer.status == OfferStatus::Ongoing @ SwapError::InvalidOfferStatus,
        seeds = [
            b"offer",
            offer.maker.as_ref(),
            offer.offer_id.to_le_bytes().as_ref()
        ],
        bump = offer.bump
    )]
    pub offer: Account<'info, Offer>,

    #[account(
        mut,
        constraint = maker_token_account.owner == offer.maker
    )]
    pub maker_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = taker_token_account.owner == taker.key()
    )]
    pub taker_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub fee_wallet: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = vault_token_account.mint == offer.token_mint
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    /// CHECK: This is the PDA that signs for vault transfers and is validated by the seeds constraint
    #[account(
        seeds = [b"vault", offer.token_mint.as_ref()],
        bump
    )]
    pub vault_authority: AccountInfo<'info>,

    pub fee_config: Account<'info, FeeConfig>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> TakeOffer<'info> {
    pub fn process(
        &mut self,
        token_amount: u64,
        vault_authority_bump: u8,
    ) -> Result<()> {
        let offer = &mut self.offer;

        require!(
            Clock::get()?.unix_timestamp <= offer.deadline,
            SwapError::OfferExpired
        );

        require!(
            token_amount <= offer.token_amount,
            SwapError::InsufficientAmount
        );

        let expected_payment = (token_amount as u128)
            .checked_mul(offer.expected_total_amount as u128)
            .unwrap()
            .checked_div(offer.token_amount as u128)
            .unwrap() as u64;

        let fee_amount = expected_payment
            .checked_mul(self.fee_config.fee_amount)
            .unwrap()
            .checked_div(10000)
            .unwrap();
        let payment_after_fee = expected_payment.checked_sub(fee_amount).unwrap();

        token::transfer(
            CpiContext::new(
                self.token_program.to_account_info(),
                token::Transfer {
                    from: self.taker_token_account.to_account_info(),
                    to: self.fee_wallet.to_account_info(),
                    authority: self.taker.to_account_info(),
                },
            ),
            fee_amount,
        )?;

        token::transfer(
            CpiContext::new(
                self.token_program.to_account_info(),
                token::Transfer {
                    from: self.taker_token_account.to_account_info(),
                    to: self.maker_token_account.to_account_info(),
                    authority: self.taker.to_account_info(),
                },
            ),
            payment_after_fee,
        )?;

        token::transfer(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                token::Transfer {
                    from: self.vault_token_account.to_account_info(),
                    to: self.taker_token_account.to_account_info(),
                    authority: self.vault_authority.to_account_info(),
                },
                &[&[&b"vault"[..], &offer.token_mint.as_ref(), &[vault_authority_bump]]],
            ),
            token_amount,
        )?;

        offer.token_amount = offer.token_amount.checked_sub(token_amount).unwrap();
        if offer.token_amount == 0 {
            offer.status = OfferStatus::Completed;
        }

        Ok(())
    }
}