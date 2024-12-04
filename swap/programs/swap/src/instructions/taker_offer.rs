use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, CloseAccount};
use crate::state::*;
use crate::error::*;

/// TakeOffer instruction implementation
/// Allows a whitelisted taker to accept an existing offer by:
/// 1. Sending the requested output tokens (payment + fees)
/// 2. Receiving the offered input tokens from the vault
/// 3. Closes all related accounts when offer is fully taken
#[derive(Accounts)]
pub struct TakeOffer<'info> {
    /// Taker (transaction signer) who will:
    /// - Send output tokens as payment
    /// - Pay protocol fees
    /// - Receive input tokens from vault
    /// Must be whitelisted in the offer's whitelist PDA
    #[account(mut)]
    pub taker: Signer<'info>,
    
    /// The offer account storing all trade details
    /// - Validates trade is still active
    /// - Stores input/output token info
    /// - Tracks remaining token amount
    /// - Contains maker's address
    /// Will be closed and rent returned to maker when fully taken
    #[account(
        mut,
        constraint = offer.status == OfferStatus::Ongoing @ SwapError::InvalidOfferStatus,
        seeds = [
            b"offer",
            offer.maker.as_ref(),
            offer.offer_id.to_le_bytes().as_ref()
        ],
        bump,
        close = maker
    )]
    pub offer: Account<'info, Offer>,

    /// Original offer maker who will:
    /// - Receive payment in output tokens
    /// - Get rent refunds from closed accounts
    /// Validated to match the maker stored in offer
    /// CHECK: Maker account
    #[account(
        mut,
        constraint = maker.key() == offer.maker
    )]
    pub maker: AccountInfo<'info>,

    /// Whitelist PDA controlling who can take this offer
    /// - Stores array of allowed taker addresses
    /// - Validates taker is authorized
    /// - Closed when offer completes (rent to maker)
    /// PDA seeds: ["whitelist", offer pubkey]
    #[account(
        mut,
        seeds = [
            b"whitelist",
            offer.key().as_ref()
        ],
        bump,
        constraint = whitelist.offer == offer.key(),
        constraint = whitelist.takers.contains(&taker.key()) @ SwapError::TakerNotWhitelisted,
        close = maker
    )]
    pub whitelist: Account<'info, Whitelist>,

    /// Offer-specific fee configuration PDA
    /// - Stores fee percentage for this offer
    /// - Stores fee recipient address
    /// - Used to calculate protocol fees
    /// - Closed when offer completes (rent to maker)
    /// PDA seeds: ["fee_config", offer pubkey]
    #[account(
        mut,
        seeds = [
            b"fee_config",
            offer.key().as_ref()
        ],
        bump,
        close = maker
    )]
    pub offer_fee_config: Account<'info, FeeConfig>,

    /// Maker's token account to receive payment
    /// Must be owned by maker and accept output tokens
    #[account(
        mut,
        constraint = maker_receive_token_account.owner == offer.maker,
        constraint = maker_receive_token_account.mint == offer.output_token_mint
    )]
    pub maker_receive_token_account: Account<'info, TokenAccount>,

    /// Taker's token account to pay from
    /// Must be owned by taker and hold output tokens
    #[account(
        mut,
        constraint = taker_payment_token_account.owner == taker.key(),
        constraint = taker_payment_token_account.mint == offer.output_token_mint
    )]
    pub taker_payment_token_account: Account<'info, TokenAccount>,

    /// Taker's token account to receive offered tokens
    /// Must be owned by taker and accept input tokens
    #[account(
        mut,
        constraint = taker_receive_token_account.owner == taker.key(),
        constraint = taker_receive_token_account.mint == offer.input_token_mint
    )]
    pub taker_receive_token_account: Account<'info, TokenAccount>,

    /// Protocol fee receiving account
    /// Must be owned by fee_address from config
    /// Must accept output tokens
    #[account(
        mut,
        constraint = fee_token_account.owner == offer_fee_config.fee_address,
        constraint = fee_token_account.mint == offer.output_token_mint
    )]
    pub fee_token_account: Account<'info, TokenAccount>,

    /// Vault holding the offered input tokens
    /// Closed when offer fully taken (rent to maker)
    #[account(
        mut,
        constraint = vault_token_account.mint == offer.input_token_mint
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    /// PDA that controls the vault token account
    /// Used to sign transfers from vault
    /// PDA seeds: ["vault", input_token_mint]
    /// CHECK: Valut authority seeds are present
    #[account(
        seeds = [b"vault", offer.input_token_mint.as_ref()],
        bump
    )]
    pub vault_authority: AccountInfo<'info>,

    /// The SPL Token program for token operations
    pub token_program: Program<'info, Token>,
    
    /// The System program for PDA operations
    pub system_program: Program<'info, System>,
}

    /// Process a take offer instruction
    /// 
    /// # Arguments
    /// * `ctx` - Instruction context containing all account references
    /// * `token_amount` - Amount of input tokens to take from the offer
    ///
    /// # Flow
    /// 1. Validate offer expiry and available amount
    /// 2. Calculate proportional payment and fees
    /// 3. Transfer fees to protocol
    /// 4. Transfer payment to maker
    /// 5. Transfer input tokens to taker
    /// 6. Handle account closing if offer fully taken
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub fn process(        
        ctx: Context<TakeOffer>,
        token_amount: u64,
    ) -> Result<()> {
        let offer = &mut ctx.accounts.offer;

        // Verify offer hasn't expired
        require!(
            Clock::get()?.unix_timestamp <= offer.deadline,
            SwapError::OfferExpired
        );

        // Verify requested amount is available
        require!(
            token_amount <= offer.token_amount,
            SwapError::InsufficientAmount
        );

        // Calculate proportional payment based on taken amount
        // (taken_amount * total_expected) / total_amount
        let expected_payment = (token_amount as u128)
            .checked_mul(offer.expected_total_amount as u128)
            .unwrap()
            .checked_div(offer.token_amount as u128)
            .unwrap() as u64;

        // Calculate protocol fee and final payment
        // Fee is in basis points (1/100th of a percent)
        let fee_amount = expected_payment
            .checked_mul(ctx.accounts.offer_fee_config.fee_percentage)
            .unwrap()
            .checked_div(10000)
            .unwrap();
        let payment_after_fee = expected_payment.checked_sub(fee_amount).unwrap();

        // Transfer protocol fee to fee wallet
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.taker_payment_token_account.to_account_info(),
                    to: ctx.accounts.fee_token_account.to_account_info(),
                    authority: ctx.accounts.taker.to_account_info(),
                },
            ),
            fee_amount,
        )?;

        // Transfer payment to maker
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.taker_payment_token_account.to_account_info(),
                    to: ctx.accounts.maker_receive_token_account.to_account_info(),
                    authority: ctx.accounts.taker.to_account_info(),
                },
            ),
            payment_after_fee,
        )?;

        // Setup vault authority signing
        let vault_auth_bump = ctx.bumps.vault_authority;
        let seeds = &[
            b"vault",
            offer.input_token_mint.as_ref(),
            &[vault_auth_bump],
        ];

        // Transfer input tokens from vault to taker
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.vault_token_account.to_account_info(),
                    to: ctx.accounts.taker_receive_token_account.to_account_info(),
                    authority: ctx.accounts.vault_authority.to_account_info(),
                },
                &[seeds]
            ),
            token_amount,
        )?;

        // Handle full vs partial take
        if token_amount == offer.token_amount {
            // Close vault token account if fully taken
            token::close_account(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    CloseAccount {
                        account: ctx.accounts.vault_token_account.to_account_info(),
                        destination: ctx.accounts.maker.to_account_info(),
                        authority: ctx.accounts.vault_authority.to_account_info(),
                    },
                    &[seeds]
                )
            )?;

            // Mark as completed - triggers closing of PDAs
            offer.status = OfferStatus::Completed;
        } else {
            // Update remaining amount for partial take
            offer.token_amount = offer.token_amount.checked_sub(token_amount).unwrap();
        }

        Ok(())
    }
