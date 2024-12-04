use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, CloseAccount};
use crate::state::*;
use crate::error::*;

/// CancelOffer instruction account structure
/// Allows maker to cancel their offer and recover tokens if:
/// 1. The offer has expired, or
/// 2. The maker wants to cancel their own active offer
#[derive(Accounts)]
pub struct CancelOffer<'info> {
    /// The maker of the offer who will receive back their tokens
    #[account(mut)]
    pub maker: Signer<'info>,
    
    /// The offer account to be cancelled
    /// Will be closed and rent returned to maker
    #[account(
        mut,
        constraint = offer.maker == maker.key(),
        constraint = offer.status == OfferStatus::Ongoing @ SwapError::InvalidOfferStatus,
        seeds = [
            b"offer",
            maker.key().as_ref()
        ],
        bump,
        close = maker
    )]
    pub offer: Account<'info, Offer>,

    /// The whitelist PDA to be closed
    #[account(
        mut,
        seeds = [
            b"whitelist",
            offer.key().as_ref()
        ],
        bump,
        constraint = whitelist.offer == offer.key(),
        close = maker
    )]
    pub whitelist: Account<'info, Whitelist>,

    /// The fee config PDA to be closed
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

    /// The maker's token account to receive returned tokens
    #[account(
        mut,
        constraint = maker_token_account.owner == maker.key(),
        constraint = maker_token_account.mint == offer.input_token_mint
    )]
    pub maker_token_account: Account<'info, TokenAccount>,

    /// The vault's token account holding the offered tokens
    #[account(
        mut,
        constraint = vault_token_account.mint == offer.input_token_mint
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    /// The PDA with authority over the vault
    /// CHECK: Validated by seeds constraint
    #[account(
        seeds = [b"vault", offer.input_token_mint.as_ref()],
        bump
    )]
    pub vault_authority: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

/// Processes the cancellation of a token offer. This function:
/// 1. Verifies cancellation conditions (offer expired or maker cancelling)
/// 2. Returns tokens from vault to maker
/// 3. Closes vault token account
/// 4. Updates offer status to Cancelled
///
/// # Arguments
/// * `ctx` - The context containing all accounts required for offer cancellation
///
/// # Returns
/// * `Result<()>` - Success or error
///
/// # Errors
/// * `SwapError::CannotCancelOffer` - If cancellation conditions aren't met
pub fn update_cancel_offer(ctx: Context<CancelOffer>) -> Result<()> {
    let offer = &mut ctx.accounts.offer;
    let current_time = Clock::get()?.unix_timestamp;

    require!(
        current_time > offer.deadline || offer.maker == ctx.accounts.maker.key(),
        SwapError::CannotCancelOffer
    );

    let vault_auth_bump = ctx.bumps.vault_authority;
    let seeds = &[
        b"vault",
        offer.input_token_mint.as_ref(),
        &[vault_auth_bump],
    ];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.vault_token_account.to_account_info(),
                to: ctx.accounts.maker_token_account.to_account_info(),
                authority: ctx.accounts.vault_authority.to_account_info(),
            },
            &[seeds]
        ),
        offer.token_amount,
    )?;

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

    offer.status = OfferStatus::Cancelled;

    Ok(())
}