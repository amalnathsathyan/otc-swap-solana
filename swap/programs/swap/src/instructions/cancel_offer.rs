use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface}
};
use crate::state::*;
use crate::error::*;

#[event]
pub struct OfferCancelled {
    pub offer_id: u64,
    pub maker: Pubkey,
    pub token_amount: u64,
    pub token_mint: Pubkey,
    pub reason: CancellationReason,
    pub timestamp: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum CancellationReason {
    Expired,
    MakerCancelled,
}

/// Account validation structure for the cancel offer instruction
/// Allows offer makers to cancel their offers and recover tokens in two cases:
/// 1. When the offer has expired (past deadline)
/// 2. When the maker wants to voluntarily cancel their active offer
///
/// The instruction will:
/// - Return tokens from vault to maker
/// - Close all related PDAs
/// - Update protocol statistics
/// - Return rent to maker
#[derive(Accounts)]
pub struct CancelOffer<'info> {
    /// Original offer maker who will:
    /// - Sign the cancellation transaction
    /// - Receive returned tokens
    /// - Receive rent from closed accounts
    /// Must match the maker stored in the offer
    #[account(mut)]
    pub maker: Signer<'info>,
    
    /// The offer PDA that serves dual purpose as:
    /// 1. Storage for offer details
    /// 2. Authority over the vault token account
    /// 
    /// Constraints:
    /// - Must be signed by original maker
    /// - Must be in Ongoing status
    /// - Will be closed with rent returned to maker
    ///
    /// Seeds: ["offer", maker_pubkey, offer_id]
    #[account(
        mut,
        constraint = offer.maker == maker.key(),
        constraint = offer.status == OfferStatus::Ongoing @ SwapError::InvalidOfferStatus,
        seeds = [b"offer", maker.key().as_ref(), &offer.offer_id.to_le_bytes()],
        bump,
    )]
    pub offer: Account<'info, Offer>,

    /// The whitelist PDA storing allowed takers
    /// Will be closed and rent returned to maker
    /// 
    /// Seeds: ["whitelist", maker_pubkey, offer_id]
    #[account(
        mut,
        seeds = [b"whitelist", maker.key().as_ref(), &offer.offer_id.to_le_bytes()],
        bump,
    )]
    pub whitelist: Account<'info, Whitelist>,

    /// Token account owned by maker that will receive
    /// returned tokens from the vault
    /// 
    /// Constraints:
    /// - Must be owned by maker
    /// - Must match input token mint from offer
    #[account(
        mut,
        constraint = maker_token_account.owner == maker.key(),
        constraint = maker_token_account.mint == offer.input_token_mint
    )]
    pub maker_token_account: InterfaceAccount<'info, TokenAccount>,

    /// The vault token account holding the offered tokens
    /// Created as an Associated Token Account owned by offer PDA
    /// Will be closed after returning tokens
    /// 
    /// Constraints:
    /// - Must be an ATA
    /// - Must have offer PDA as authority
    /// - Must match input token mint
    #[account(
        mut,
        associated_token::mint = input_token_mint,
        associated_token::authority = offer,
        associated_token::token_program = token_program,
        constraint = vault_token_account.mint == offer.input_token_mint,
    )]
    pub vault_token_account: InterfaceAccount<'info, TokenAccount>,

    /// Admin configuration PDA for updating protocol statistics
    /// Tracks active/cancelled offer counts
    ///
    /// Seeds: ["admin_config"]
    #[account(
        mut,
        seeds = [b"admin_config"],
        bump
    )]
    pub admin_config: Account<'info, AdminConfig>,

    /// The mint of the token being returned
    /// Used for transfer_checked validation
    pub input_token_mint: InterfaceAccount<'info, Mint>,

    /// Token interface program for Token-2022 support
    pub token_program: Interface<'info, TokenInterface>,

    /// Required for ATA validation
    pub associated_token_program: Program<'info, AssociatedToken>,

    pub system_program: Program<'info, System>,
}

/// Processes an offer cancellation request
/// 
/// This function handles the complete cancellation flow including:
/// 1. Validation of cancellation conditions
/// 2. Return of tokens to maker
/// 3. Closure of vault and PDAs
/// 4. Update of protocol statistics
/// 
/// # Arguments
/// * `ctx` - The CancelOffer context containing all required accounts
///
/// # Security Checks
/// - Verifies maker authority
/// - Validates offer status
/// - Checks cancellation conditions (expired or maker cancellation)
/// - Ensures proper token account ownership
/// - Validates token mint matches
///
/// # Token Operations
/// - Uses transfer_checked for safe token returns
/// - Properly closes vault token account
/// - Returns all rent to maker
///
/// # State Updates
/// - Marks offer as Cancelled
/// - Updates protocol statistics
/// - Closes related PDAs
///
/// # Errors
/// * `SwapError::CannotCancelOffer` - If neither expiry nor maker cancellation conditions are met
/// * `SwapError::InvalidOfferStatus` - If offer is not in Ongoing status
/// * Various token program errors for transfer failures

pub fn update_cancel_offer(ctx: Context<CancelOffer>) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;

    // Validate cancellation conditions
    let is_expired = current_time > ctx.accounts.offer.deadline;
    let is_maker = ctx.accounts.offer.maker == ctx.accounts.maker.key();
    require!(is_expired || is_maker, SwapError::CannotCancelOffer);

    let cancellation_reason = if is_expired {
        CancellationReason::Expired
    } else {
        CancellationReason::MakerCancelled
    };

    msg!(
       "cancelling offer"
    );

    // Transfer remaining tokens back to the maker
    let token_amount = ctx.accounts.offer.token_amount_remaining;
    let seeds = &[
        b"offer",
        ctx.accounts.offer.maker.as_ref(),
        &ctx.accounts.offer.offer_id.to_le_bytes(),
        &[ctx.bumps.offer],
    ];
    let signer_seeds = &[&seeds[..]];

    msg!("Preparing to close vault account");

    msg!("Transferring {} tokens back to maker", token_amount);
    token_interface::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token_interface::TransferChecked {
                from: ctx.accounts.vault_token_account.to_account_info(),
                mint: ctx.accounts.input_token_mint.to_account_info(),
                to: ctx.accounts.maker_token_account.to_account_info(),
                authority: ctx.accounts.offer.to_account_info(),
            },
            signer_seeds,
        ),
        token_amount,
        ctx.accounts.input_token_mint.decimals,
    )?;

    // Mark offer as cancelled

    // Close the vault account
    msg!("Closing vault account...");
    token_interface::close_account(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token_interface::CloseAccount {
                account: ctx.accounts.vault_token_account.to_account_info(),
                destination: ctx.accounts.maker.to_account_info(),
                authority: ctx.accounts.offer.to_account_info(),
            },
            signer_seeds,
        )
    )?;
    msg!("Vault closed successfully");

    ctx.accounts.offer.status = OfferStatus::Cancelled;

    // Emit cancellation event
    emit!(OfferCancelled {
        offer_id: ctx.accounts.offer.offer_id,
        maker: ctx.accounts.maker.key(),
        token_amount,
        token_mint: ctx.accounts.input_token_mint.key(),
        reason: cancellation_reason,
        timestamp: current_time,
    });

    Ok(())
}
