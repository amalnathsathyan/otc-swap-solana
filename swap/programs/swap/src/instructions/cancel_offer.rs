use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface}
};
use crate::state::*;
use crate::error::*;

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
    /// Seeds: ["offer", maker_pubkey]
    #[account(
        mut,
        constraint = offer.maker == maker.key(),
        constraint = offer.status == OfferStatus::Ongoing @ SwapError::InvalidOfferStatus,
        seeds = [b"offer", maker.key().as_ref()],
        bump,
        close = maker
    )]
    pub offer: Account<'info, Offer>,

    /// The whitelist PDA storing allowed takers
    /// Will be closed and rent returned to maker
    /// 
    /// Seeds: ["whitelist", maker_pubkey]
    #[account(
        mut,
        seeds = [b"whitelist", maker.key().as_ref()],
        bump,
        close = maker
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
        constraint = vault_token_account.mint == offer.input_token_mint
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
    // Get current time and store important values
    let current_time = Clock::get()?.unix_timestamp;
    let maker_key = ctx.accounts.maker.key();

    // Store necessary offer values before mutable borrow
    let token_amount = ctx.accounts.offer.token_amount;
    let input_token_decimals = ctx.accounts.input_token_mint.decimals;
    let offer_deadline = ctx.accounts.offer.deadline;
    let offer_maker = ctx.accounts.offer.maker;

    // Store offer's authority info for CPI calls
    let offer_auth_info = ctx.accounts.offer.to_account_info();

    // Verify cancellation conditions
    require!(
        current_time > offer_deadline || offer_maker == maker_key,
        SwapError::CannotCancelOffer
    );

    // Prepare offer PDA signer seeds
    let offer_bump = ctx.bumps.offer;
    let seeds = &[
        b"offer",
        maker_key.as_ref(),
        &[offer_bump],
    ];

    // Return tokens to maker using transfer_checked for safety
    token_interface::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token_interface::TransferChecked {
                from: ctx.accounts.vault_token_account.to_account_info(),
                mint: ctx.accounts.input_token_mint.to_account_info(),
                to: ctx.accounts.maker_token_account.to_account_info(),
                authority: offer_auth_info.clone(),
            },
            &[seeds]
        ),
        token_amount,
        input_token_decimals,
    )?;

    // Close vault token account and return rent to maker
    token_interface::close_account(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token_interface::CloseAccount {
                account: ctx.accounts.vault_token_account.to_account_info(),
                destination: ctx.accounts.maker.to_account_info(),
                authority: offer_auth_info,
            },
            &[seeds]
        )
    )?;

    // Update protocol statistics
    ctx.accounts.admin_config.update_offer_status(
        Some(OfferStatus::Ongoing),
        OfferStatus::Cancelled
    );

    // Finally update offer status - triggers PDA closure
    let offer = &mut ctx.accounts.offer;
    offer.status = OfferStatus::Cancelled;

    Ok(())
}