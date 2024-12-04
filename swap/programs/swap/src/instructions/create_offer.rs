use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface}
};
use crate::state::*;
use crate::error::*;

/// Account structure for creating and managing token swap offers
/// Creates a unique vault for each offer using ATA
#[derive(Accounts)]
pub struct CreateOffer<'info> {
    /// The offer creator (maker) who will deposit tokens
    /// Pays for all account initializations
    #[account(mut)]
    pub maker: Signer<'info>,

    /// PDA storing offer details
    /// Space breakdown:
    /// - 8 bytes discriminator
    /// - 32 bytes offer_id
    /// - 32 bytes maker pubkey
    /// - 32 bytes input token mint
    /// - 8 bytes token amount
    /// - 32 bytes output token mint
    /// - 8 bytes expected amount
    /// - 8 bytes deadline
    /// - 32 bytes fee wallet
    /// - 1 byte status enum
    #[account(
        init,
        payer = maker,
        space = 8 + 32 + 32 + 32 + 8 + 32 + 8 + 8 + 32 + 1,
        seeds = [b"offer", maker.key().as_ref()],
        bump
    )]
    pub offer: Account<'info, Offer>,

    /// PDA storing allowed taker addresses
    /// Space breakdown:
    /// - 8 bytes discriminator
    /// - 32 bytes offer pubkey
    /// - 32 bytes maker pubkey
    /// - 4 bytes vec length
    /// - 32 * 50 bytes for taker pubkeys (max 50 takers)
    #[account(
        init,
        payer = maker,
        space = 8 + 32 + 32 + 4 + (32 * 50),
        seeds = [b"whitelist", maker.key().as_ref()],
        bump
    )]
    pub whitelist: Account<'info, Whitelist>,

    /// Admin configuration PDA for protocol statistics and verification
    /// Must be initialized before any offers can be created
    #[account(
        mut,
        seeds = [b"admin_config"],
        bump,
        constraint = admin_config.admin.key() != Pubkey::default() @ SwapError::AdminNotInitialized
    )]
    pub admin_config: Account<'info, AdminConfig>,

    /// Global fee configuration PDA
    /// Provides fee parameters that will be copied to the offer
    #[account(
        seeds = [b"fee_config"],
        bump,
        constraint = fee_config.fee_address != Pubkey::default() @ SwapError::FeeConfigNotInitialized
    )]
    pub fee_config: Account<'info, FeeConfig>,

    /// Token account owned by maker containing input tokens
    /// Must match the input token mint and be owned by maker
    #[account(
        mut,
        constraint = maker_token_account.owner == maker.key() @ SwapError::InvalidTokenAccount,
        constraint = maker_token_account.mint == input_token_mint.key() @ SwapError::InvalidTokenMint
    )]
    pub maker_token_account: InterfaceAccount<'info, TokenAccount>,
    
    /// Protocol vault specific to this offer
    /// Initialized as an Associated Token Account with offer PDA as authority
    /// Will hold the tokens until the offer is taken or cancelled
    #[account(
        init,
        payer = maker,
        associated_token::mint = input_token_mint,
        associated_token::authority = offer,
        associated_token::token_program = token_program
    )]
    pub vault_token_account: InterfaceAccount<'info, TokenAccount>,

    /// The mint of the token being offered
    /// Used to validate token accounts and create vault
    pub input_token_mint: InterfaceAccount<'info, Mint>,

    /// The mint of the token being requested
    /// Will be used by takers to fulfill the offer
    pub output_token_mint: InterfaceAccount<'info, Mint>,

    /// Token program interface supporting both SPL Token and Token-2022
    pub token_program: Interface<'info, TokenInterface>,
    
    /// Required for initializing the vault ATA
    pub associated_token_program: Program<'info, AssociatedToken>,
    
    pub system_program: Program<'info, System>,
}

/// Creates a new token swap offer with specified parameters
/// Initializes a unique vault for the offered tokens
///
/// # Arguments
/// * `ctx` - CreateOffer context containing required accounts
/// * `offer_id` - Unique identifier for this offer
/// * `token_amount` - Amount of input tokens being offered
/// * `expected_total_amount` - Amount of output tokens expected in return
/// * `deadline` - Unix timestamp when offer expires
/// * `initial_takers` - Vector of addresses initially allowed to take offer
///
/// # Flow
/// 1. Validates offer parameters and accounts
/// 2. Initializes offer PDA with input parameters
/// 3. Sets up taker whitelist
/// 4. Updates protocol statistics 
/// 5. Creates offer-specific vault as ATA
/// 6. Transfers tokens from maker to vault
///
/// # Security
/// - Verifies admin initialization
/// - Validates token account ownership
/// - Ensures matching token mints
/// - Uses PDA derivation for security
/// - Creates unique vault per offer
/// - Uses Associated Token Program for deterministic addresses
///
/// # Errors
/// * `SwapError::InvalidDeadline` - If deadline is in the past
/// * `SwapError::AdminNotInitialized` - If admin config not set up
/// * `SwapError::FeeConfigNotInitialized` - If fee config not set up
/// * `SwapError::InvalidTokenAccount` - If token accounts don't match
/// * `SwapError::InvalidTokenMint` - If token mints don't match
pub fn create_offer(
    ctx: Context<CreateOffer>,
    token_amount: u64,
    expected_total_amount: u64,
    deadline: i64,
    initial_takers: Vec<Pubkey>,
) -> Result<()> {

    let offer_key = ctx.accounts.offer.key();
    // Verify deadline is in the future
    let current_time = Clock::get()?.unix_timestamp;
    require!(deadline > current_time, SwapError::InvalidDeadline);

    // Initialize offer account
    let offer = &mut ctx.accounts.offer;
    offer.offer_id = offer_key;
    offer.maker = ctx.accounts.maker.key();
    offer.input_token_mint = ctx.accounts.input_token_mint.key();
    offer.output_token_mint = ctx.accounts.output_token_mint.key();
    offer.token_amount = token_amount;
    offer.expected_total_amount = expected_total_amount;
    offer.deadline = deadline;
    offer.status = OfferStatus::Ongoing;
    
    // Copy global fee configuration to offer
    offer.fee_percentage = ctx.accounts.fee_config.fee_percentage;
    offer.fee_wallet = ctx.accounts.fee_config.fee_address;

    // Set up taker whitelist
    let whitelist = &mut ctx.accounts.whitelist;
    whitelist.offer = offer_key;
    whitelist.maker = ctx.accounts.maker.key();
    whitelist.takers = initial_takers;

    // Update protocol statistics
    let admin_config = &mut ctx.accounts.admin_config;
    admin_config.total_offers += 1;
    admin_config.active_offers += 1;

    // Transfer tokens to offer's vault
    token_interface::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token_interface::TransferChecked {
                from: ctx.accounts.maker_token_account.to_account_info(),
                mint: ctx.accounts.input_token_mint.to_account_info(),
                to: ctx.accounts.vault_token_account.to_account_info(),
                authority: ctx.accounts.maker.to_account_info(),
            },
        ),
        token_amount,
        ctx.accounts.input_token_mint.decimals,
    )?;

    Ok(())
}

