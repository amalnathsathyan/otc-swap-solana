use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};
use crate::state::*;

/// Accounts structure for creating a new offer in the token swap system
/// Takes two instruction arguments:
/// - offer_id: Unique identifier for the offer
/// - bump: Bump seed for PDA derivation
#[derive(Accounts)]
#[instruction(offer_id: u64, bump: u8)]
pub struct CreateOffer<'info> {
    /// The maker (creator) of the offer who will deposit tokens
    /// Must be the signer and will pay for account creation
    #[account(mut)]
    pub maker: Signer<'info>,

    /// The offer account that will store all offer details
    /// This is a PDA derived from:
    /// - "offer" prefix
    /// - maker's public key
    /// - offer_id
    /// Space calculation:
    /// - 8 bytes for discriminator
    /// - 32 bytes for maker pubkey
    /// - 32 bytes for token mint
    /// - 8 bytes for token amount
    /// - 8 bytes for expected total amount
    /// - 8 bytes for deadline
    /// - 1 byte for status
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

    /// The mint of the token being offered
    pub token_mint: Account<'info, token::Mint>,

    /// The maker's token account that will send tokens to the vault
    #[account(mut)]
    pub maker_token_account: Account<'info, TokenAccount>,

    /// The vault's token account that will receive and hold the tokens
    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,

    /// The PDA that has authority over the vault
    /// Derived using the token mint as part of the seeds
    /// CHECK: PDA for vault authority
    #[account(
        seeds = [b"vault", token_mint.key().as_ref()],
        bump
    )]
    pub vault_authority: AccountInfo<'info>,

    /// The SPL Token program account
    pub token_program: Program<'info, Token>,

    /// The Solana System program account (needed for account creation)
    pub system_program: Program<'info, System>,
}

impl<'info> CreateOffer<'info> {
    /// Process the creation of a new offer
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the offer
    /// * `token_mint` - Public key of the token mint being offered
    /// * `token_amount` - Amount of tokens to be locked in the offer
    /// * `expected_total_amount` - Total amount expected in return
    /// * `deadline` - Unix timestamp for offer expiration
    /// * `bump` - Bump seed for PDA derivation
    ///
    /// # Steps
    /// 1. Initializes the offer account with provided parameters
    /// 2. Transfers tokens from maker's account to vault account
    ///
    /// # Returns
    /// * `Result<()>` - Ok if successful, Error if token transfer fails
    pub fn process(
        &mut self,
        id: u64,
        token_mint: Pubkey,
        token_amount: u64,
        expected_total_amount: u64,
        deadline: i64,
        bump: u8,
    ) -> Result<()> {

        // Initialize offer account with provided parameters
        let offer = &mut self.offer;
        offer.offer_id = id;
        offer.maker = self.maker.key();
        offer.token_mint = token_mint;
        offer.token_amount = token_amount;
        offer.expected_total_amount = expected_total_amount;
        offer.deadline = deadline;
        offer.status = OfferStatus::Ongoing;
        offer.bump = bump;

        // Transfer tokens from maker to vault
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