use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface}
};
use crate::state::*;
use crate::error::*;

/// Account structure for taking an existing offer
/// Handles token transfers and account initialization
#[derive(Accounts)]
pub struct TakeOffer<'info> {
    /// The taker (transaction signer) who will:
    /// - Pay for any account initializations
    /// - Send output tokens as payment
    /// - Pay protocol fees
    /// - Receive input tokens from vault
    #[account(mut)]
    pub taker: Signer<'info>,
    
    /// The offer PDA that also serves as vault authority
    /// Stores all trade details and validates the transaction
    /// Will be closed when offer is fully taken
    #[account(
        mut,
        constraint = offer.status == OfferStatus::Ongoing @ SwapError::InvalidOfferStatus,
        seeds = [b"offer", offer.maker.as_ref()],
        bump,
        close = maker
    )]
    pub offer: Account<'info, Offer>,

    /// Original offer maker who will:
    /// - Receive payment in output tokens
    /// - Get rent refunds from closed accounts
    /// CHECK: Read-only maker account validated by offer PDA
    #[account(
        mut,
        constraint = maker.key() == offer.maker
    )]
    pub maker: AccountInfo<'info>,

    /// Whitelist PDA controlling who can take this offer
    /// Validates taker is authorized and closes with offer
    #[account(
        mut,
        seeds = [b"whitelist", offer.maker.as_ref()],
        bump,
        constraint = whitelist.takers.contains(&taker.key()) @ SwapError::TakerNotWhitelisted,
        close = maker
    )]
    pub whitelist: Account<'info, Whitelist>,

    /// Maker's token account to receive payment
    /// Initialized if needed as an ATA
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = output_token_mint,
        associated_token::authority = maker,
    )]
    pub maker_receive_token_account: InterfaceAccount<'info, TokenAccount>,

    /// Taker's token account to pay from
    /// Initialized if needed as an ATA
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = output_token_mint,
        associated_token::authority = taker,
    )]
    pub taker_payment_token_account: InterfaceAccount<'info, TokenAccount>,

    /// Taker's token account to receive offered tokens
    /// Initialized if needed as an ATA
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = input_token_mint,
        associated_token::authority = taker,
    )]
    pub taker_receive_token_account: InterfaceAccount<'info, TokenAccount>,

    /// Protocol fee receiving account
    /// Must be owned by fee_wallet from offer
    #[account(
        mut,
        constraint = fee_token_account.owner == offer.fee_wallet,
        constraint = fee_token_account.mint == offer.output_token_mint
    )]
    pub fee_token_account: InterfaceAccount<'info, TokenAccount>,

    /// Vault token account holding the offered tokens
    /// Owned by the offer PDA
    #[account(
        mut,
        associated_token::mint = input_token_mint,
        associated_token::authority = offer,
        constraint = vault_token_account.mint == offer.input_token_mint
    )]
    pub vault_token_account: InterfaceAccount<'info, TokenAccount>,

    /// Mint of the token being traded in
    /// Used for transfers and account validation 
    pub input_token_mint: InterfaceAccount<'info, Mint>,

    /// Mint of the token being traded out
    /// Used for transfers and account validation
    pub output_token_mint: InterfaceAccount<'info, Mint>,

    /// Token interface program for Token-2022 compatibility
    pub token_program: Interface<'info, TokenInterface>,

    /// Required for ATA initialization
    pub associated_token_program: Program<'info, AssociatedToken>,

    pub system_program: Program<'info, System>,
}

/// Takes an existing offer fully or partially
/// 
/// # Arguments
/// * `ctx` - TakeOffer context containing all accounts
/// * `token_amount` - Amount of input tokens to take from the offer
///
/// # Flow
/// 1. Validates offer status and amount
/// 2. Initializes any missing token accounts
/// 3. Calculates proportional payment and fees
/// 4. Transfers fees to protocol
/// 5. Transfers payment to maker
/// 6. Transfers input tokens from vault to taker
/// 7. Handles account closing for full takes
///
/// # Security Checks
/// - Verifies offer hasn't expired
/// - Validates taker is whitelisted
/// - Ensures sufficient amounts
/// - Validates all token accounts
/// - Uses checked math operations
/// - Proper PDA validation
///
/// # Errors
/// * `SwapError::InvalidOfferStatus` - If offer not ongoing
/// * `SwapError::OfferExpired` - If deadline passed
/// * `SwapError::TakerNotWhitelisted` - If taker not authorized
/// * `SwapError::InsufficientAmount` - If amount exceeds available
pub fn process(
    ctx: Context<TakeOffer>,
    token_amount: u64,
) -> Result<()> {
    // Get current timestamp once
    let current_time = Clock::get()?.unix_timestamp;
    
    // Store offer values we need before mutation
    let offer_maker = ctx.accounts.offer.maker;
    let offer_amount = ctx.accounts.offer.token_amount;
    let offer_deadline = ctx.accounts.offer.deadline;
    let offer_expected_amount = ctx.accounts.offer.expected_total_amount;
    let offer_fee_percentage = ctx.accounts.offer.fee_percentage;

    // Verify offer hasn't expired
    require!(
        current_time <= offer_deadline,
        SwapError::OfferExpired
    );

    // Verify requested amount is available
    require!(
        token_amount <= offer_amount,
        SwapError::InsufficientAmount
    );

    // Calculate proportional payment based on taken amount
    let expected_payment = (token_amount as u128)
        .checked_mul(offer_expected_amount as u128)
        .unwrap()
        .checked_div(offer_amount as u128)
        .unwrap() as u64;

    // Calculate protocol fee and final payment
    let fee_amount = expected_payment
        .checked_mul(offer_fee_percentage)
        .unwrap()
        .checked_div(10000)
        .unwrap();
    let payment_after_fee = expected_payment.checked_sub(fee_amount).unwrap();

    // Transfer protocol fee
    token_interface::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token_interface::TransferChecked {
                from: ctx.accounts.taker_payment_token_account.to_account_info(),
                mint: ctx.accounts.output_token_mint.to_account_info(),
                to: ctx.accounts.fee_token_account.to_account_info(),
                authority: ctx.accounts.taker.to_account_info(),
            },
        ),
        fee_amount,
        ctx.accounts.output_token_mint.decimals,
    )?;

    // Transfer payment to maker
    token_interface::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token_interface::TransferChecked {
                from: ctx.accounts.taker_payment_token_account.to_account_info(),
                mint: ctx.accounts.output_token_mint.to_account_info(),
                to: ctx.accounts.maker_receive_token_account.to_account_info(),
                authority: ctx.accounts.taker.to_account_info(),
            },
        ),
        payment_after_fee,
        ctx.accounts.output_token_mint.decimals,
    )?;

    // Get signer seeds for vault operations
    let offer_bump = ctx.bumps.offer;
    let seeds = &[
        b"offer",
        offer_maker.as_ref(),
        &[offer_bump],
    ];

    // Transfer tokens from vault
    token_interface::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token_interface::TransferChecked {
                from: ctx.accounts.vault_token_account.to_account_info(),
                mint: ctx.accounts.input_token_mint.to_account_info(),
                to: ctx.accounts.taker_receive_token_account.to_account_info(),
                authority: ctx.accounts.offer.to_account_info(),
            },
            &[seeds]
        ),
        token_amount,
        ctx.accounts.input_token_mint.decimals,
    )?;

    // Handle offer completion
    if token_amount == offer_amount {
        // Close vault if fully taken
        token_interface::close_account(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token_interface::CloseAccount {
                    account: ctx.accounts.vault_token_account.to_account_info(),
                    destination: ctx.accounts.maker.to_account_info(),
                    authority: ctx.accounts.offer.to_account_info(),
                },
                &[seeds]
            )
        )?;

        // Mark as completed
        ctx.accounts.offer.status = OfferStatus::Completed;
    } else {
        // Update remaining amount for partial takes
        ctx.accounts.offer.token_amount = offer_amount.checked_sub(token_amount).unwrap();
    }

    Ok(())
}