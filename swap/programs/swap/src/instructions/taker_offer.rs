use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};
use crate::state::*;
use crate::error::*;

/// Account structure for taking (accepting) an existing offer
/// Takes a bump seed as an instruction argument for PDA validation
#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct TakeOffer<'info> {
    /// The taker (acceptor) of the offer
    /// Must be the signer of the transaction
    #[account(mut)]
    pub taker: Signer<'info>,
    
    /// The offer account being taken/accepted
    /// Must be in Ongoing status and validated using PDA seeds
    /// Seeds: ["offer", maker pubkey, offer_id bytes]
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

    /// Token account belonging to the maker that will receive payment
    /// Must be owned by the offer's maker
    #[account(
        mut,
        constraint = maker_token_account.owner == offer.maker
    )]
    pub maker_token_account: Account<'info, TokenAccount>,

    /// Token account belonging to the taker that will:
    /// 1. Send payment to maker
    /// 2. Receive tokens from vault
    #[account(
        mut,
        constraint = taker_token_account.owner == taker.key()
    )]
    pub taker_token_account: Account<'info, TokenAccount>,

    /// Token account that will receive the transaction fees
    #[account(mut)]
    pub fee_wallet: Account<'info, TokenAccount>,

    /// The vault's token account holding the offered tokens
    /// Must match the token mint specified in the offer
    #[account(
        mut,
        constraint = vault_token_account.mint == offer.token_mint
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    /// The PDA that has authority over vault transfers
    /// Seeds: ["vault", token_mint]
    /// CHECK: Validated by seeds constraint
    #[account(
        seeds = [b"vault", offer.token_mint.as_ref()],
        bump
    )]
    pub vault_authority: AccountInfo<'info>,

    /// Configuration account containing fee settings
    pub fee_config: Account<'info, FeeConfig>,

    /// The SPL Token program account
    pub token_program: Program<'info, Token>,

    /// The Solana System program account
    pub system_program: Program<'info, System>,
}

impl<'info> TakeOffer<'info> {
    /// Process the taking/acceptance of an offer
    ///
    /// # Arguments
    /// * `token_amount` - Amount of tokens to take from the offer
    /// * `vault_authority_bump` - Bump seed for vault authority PDA
    ///
    /// # Steps
    /// 1. Validates offer hasn't expired
    /// 2. Checks requested amount is available
    /// 3. Calculates payment amount and fees
    /// 4. Transfers fees to fee wallet
    /// 5. Transfers payment to maker
    /// 6. Transfers tokens from vault to taker
    /// 7. Updates offer state
    ///
    /// # Errors
    /// Returns error if:
    /// - Offer has expired
    /// - Requested amount exceeds available amount
    /// - Any token transfer fails
    pub fn process(
        &mut self,
        token_amount: u64,
        vault_authority_bump: u8,
    ) -> Result<()> {
        let offer = &mut self.offer;

        // Check if offer has expired
        require!(
            Clock::get()?.unix_timestamp <= offer.deadline,
            SwapError::OfferExpired
        );

        // Verify requested amount is available
        require!(
            token_amount <= offer.token_amount,
            SwapError::InsufficientAmount
        );

        // Calculate expected payment based on proportion of tokens being taken
        let expected_payment = (token_amount as u128)
            .checked_mul(offer.expected_total_amount as u128)
            .unwrap()
            .checked_div(offer.token_amount as u128)
            .unwrap() as u64;

        // Calculate fee amount and payment after fee
        let fee_amount = expected_payment
            .checked_mul(self.fee_config.fee_amount)
            .unwrap()
            .checked_div(10000)
            .unwrap();
        let payment_after_fee = expected_payment.checked_sub(fee_amount).unwrap();

        // Transfer fee to fee wallet
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

        // Transfer payment to maker
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

        // Transfer tokens from vault to taker
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

        // Update offer state
        offer.token_amount = offer.token_amount.checked_sub(token_amount).unwrap();
        if offer.token_amount == 0 {
            offer.status = OfferStatus::Completed;
        }

        Ok(())
    }
}