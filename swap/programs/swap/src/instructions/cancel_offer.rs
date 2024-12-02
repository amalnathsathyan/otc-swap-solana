use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};
use crate::state::*;
use crate::error::*;

/// Accounts struct for canceling an offer in the token swap program
/// This structure defines all the accounts needed to cancel an existing offer
#[derive(Accounts)]
pub struct CancelOffer<'info> {
    /// The maker (creator) of the offer who is requesting cancellation
    /// Must be the signer of the transaction
    #[account(mut)]
    pub maker: Signer<'info>,

    /// The token account belonging to the maker that will receive the returned tokens
    /// Must be mutable as it will receive tokens from the vault
    #[account(mut)]
    pub maker_token_account: Account<'info, TokenAccount>,

    /// The vault's token account that currently holds the locked tokens
    /// Must be mutable as tokens will be withdrawn from it
    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,

    /// The offer account that contains the details of the offer being cancelled
    /// Constraints ensure:
    /// 1. Only the original maker can cancel the offer
    /// 2. The offer must be in Ongoing status
    #[account(
        mut,
        constraint = offer.maker == maker.key(),
        constraint = offer.status == OfferStatus::Ongoing
    )]
    pub offer: Account<'info, Offer>,

    /// The PDA that has authority over the vault
    /// This account is used to sign the transfer of tokens back to the maker
    /// CHECK: PDA for vault authority
    #[account(
        seeds = [b"vault"],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,

    /// The SPL Token program account
    pub token_program: Program<'info, Token>,
}

impl<'info> CancelOffer<'info> {
    /// Process the cancellation of an offer
    /// 
    /// This function:
    /// 1. Verifies the offer is in the correct status
    /// 2. Transfers the locked tokens back to the maker's token account
    /// 3. Updates the offer status to Cancelled
    ///
    /// # Errors
    /// Returns an error if:
    /// - The offer is not in Ongoing status
    /// - The token transfer fails
    pub fn process(&mut self) -> Result<()> {
        let offer = &mut self.offer;

        // Verify offer status is Ongoing
        require!(offer.status == OfferStatus::Ongoing, SwapError::InvalidOfferStatus);
        
        // Transfer tokens from vault back to maker's account
        // Uses PDA signing with "vault" seeds
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

        // Update offer status to Cancelled
        offer.status = OfferStatus::Cancelled;
        Ok(())
    }
}
