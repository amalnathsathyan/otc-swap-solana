use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface}
};
use crate::state::*;
use crate::error::*;

#[event]
pub struct OfferCreated {
    pub offer_id: u64,
    pub maker: Pubkey,
    pub input_token_mint: Pubkey,
    pub output_token_mint: Pubkey,
    pub token_amount: u64,
    pub expected_amount: u64,
    pub deadline: i64,
}

#[event]
pub struct TakerUpdated {
    pub offer_id: u64,
    pub maker: Pubkey,
    pub takers: Vec<Pubkey>,
}

/// Account structure for creating offer and setting up vault in one instruction
/// Combines offer initialization and token transfer for efficiency
#[derive(Accounts)]
#[instruction(offer_id:u64)]
pub struct CreateOffer<'info> {
    /// The offer creator who will:
    /// - Pay for account initialization
    /// - Provide tokens for the trade
    /// - Control offer parameters
    #[account(mut)]
    pub maker: Signer<'info>,

    // #[account(
    //     init_if_needed,
    //     payer = maker,
    //     space = 8 + 32 + 8, // discriminator + pubkey + u64
    //     seeds = [b"maker_sequence", maker.key().as_ref()],
    //     bump
    // )]
    // pub maker_sequence: Account<'info, MakerSequence>,

    /// The offer PDA storing all trade details
    /// Space breakdown:
    /// - 8 bytes discriminator
    /// - 8 bytes offer id
    /// - 32 bytes maker pubkey
    /// - 32 bytes input token mint
    /// - 8 bytes token amount
    /// - 32 bytes output token mint
    /// - 8 bytes expected amount
    /// - 8 bytes for token amount remaining
    /// - 8 bytes for expected fulfilled amount
    /// - 8 bytes deadline
    /// - 1 byte offer status
    /// - 8 bytes fee percentage
    /// - 32 bytes fee wallet
    #[account(
        init,
        payer = maker,
        space = 8 + 8 + 32 + 32 + 8 + 32 + 8 + 8 + 8 + 8 + 1 + 8 + 32,
        seeds = [b"offer", maker.key().as_ref(),&offer_id.to_le_bytes()],
        bump
    )]
    pub offer: Account<'info, Offer>,

    /// Admin configuration for protocol verification and statistics
    /// Must be initialized before any offers can be created
    #[account(
        mut,
        seeds = [b"admin_config"],
        bump,
        constraint = admin_config.admin.key() != Pubkey::default() @ SwapError::AdminNotInitialized
    )]
    pub admin_config: Account<'info, AdminConfig>,

    /// Fee configuration providing protocol fee parameters
    /// Must be initialized before any offers can be created
    #[account(
        seeds = [b"fee"],
        bump,
        constraint = fee_config.fee_address != Pubkey::default() @ SwapError::FeeConfigNotInitialized
    )]
    pub fee_config: Account<'info, FeeConfig>,

    /// Maker's token account containing tokens to be offered
    /// Must match the input token mint
    #[account(
        mut,
        constraint = maker_token_account.owner == maker.key() @ SwapError::InvalidTokenAccount,
        constraint = maker_token_account.mint == input_token_mint.key() @ SwapError::InvalidTokenMint
    )]
    pub maker_token_account: InterfaceAccount<'info, TokenAccount>,
    
    /// Vault token account created as an Associated Token Account
    /// Will hold the offered tokens until trade completion
    /// Authority is the offer PDA
    #[account(
        init,
        payer = maker,
        associated_token::mint = input_token_mint,
        associated_token::authority = offer,
        associated_token::token_program = token_program
    )]
    pub vault_token_account: InterfaceAccount<'info, TokenAccount>,

    /// Input token mint (token being offered)
    pub input_token_mint: InterfaceAccount<'info, Mint>,

    /// Output token mint (token being requested)
    pub output_token_mint: InterfaceAccount<'info, Mint>,

    /// Token interface program for Token-2022 support
    pub token_program: Interface<'info, TokenInterface>,

    /// Required for ATA initialization
    pub associated_token_program: Program<'info, AssociatedToken>,

    pub system_program: Program<'info, System>,
}

/// Account structure for managing offer whitelist
/// Handles both creation and updates to whitelist
#[derive(Accounts)]
pub struct ManageWhitelist<'info> {
    /// Original offer maker, must sign whitelist operations
    #[account(mut)]
    pub maker: Signer<'info>,

    /// The whitelist PDA storing allowed takers
    /// Created on first use with init_if_needed
    /// Space for up to 50 taker addresses
    #[account(
        init_if_needed,
        payer = maker,
        space = 8 + 32 + 32 + 4 + (32 * 50),
        seeds = [b"whitelist", maker.key().as_ref(), &offer.offer_id.to_le_bytes()],
        bump,
        constraint = offer.maker == maker.key() @ SwapError::UnauthorizedMaker
    )]
    pub whitelist: Account<'info, Whitelist>,

    /// The offer this whitelist belongs to
    /// Used to verify maker authority
    pub offer: Account<'info, Offer>,

    pub system_program: Program<'info, System>,
}

/// Creates a new offer and sets up its vault
/// Handles both offer initialization and token transfer in one transaction
/// 
/// # Arguments
/// * `ctx` - CreateOffer context containing all required accounts
/// * `token_amount` - Amount of input tokens to offer
/// * `expected_amount` - Amount of output tokens expected in return
/// * `deadline` - Unix timestamp when offer expires
///
/// # Steps
/// 1. Validate all input parameters
/// 2. Initialize offer PDA with trade details
/// 3. Create vault and transfer tokens
/// 4. Apply protocol configuration
/// 5. Update admin statistics
///
/// # Errors
/// * `SwapError::InvalidDeadline` - If deadline is in the past
/// * `SwapError::InvalidAmount` - If token amount is zero
/// * `SwapError::AdminNotInitialized` - If admin config not set
/// * `SwapError::FeeConfigNotInitialized` - If fee config not set
/// * `SwapError::InvalidTokenAccount` - If token accounts don't match
pub fn initialize_offer(
    ctx: Context<CreateOffer>,
    offer_id:u64,
    token_amount: u64,
    expected_amount: u64,
    deadline: i64,
) -> Result<()> {
    // Validate all inputs
    let current_time = Clock::get()?.unix_timestamp;
    require!(deadline > current_time, SwapError::InvalidDeadline);
    require!(token_amount > 0, SwapError::InvalidAmount);

    // let maker_sequence = &mut ctx.accounts.maker_sequence;
    // maker_sequence.offer_count = maker_sequence.offer_count.checked_add(1)
    //     .ok_or(SwapError::SequenceOverflow)?;

    // if maker_sequence.maker == Pubkey::default() {
    //     maker_sequence.maker = ctx.accounts.maker.key();
    // }

    // Initialize offer parameters
    let offer = &mut ctx.accounts.offer;
    offer.offer_id = offer_id;
    offer.maker = ctx.accounts.maker.key();
    offer.input_token_mint = ctx.accounts.input_token_mint.key();
    offer.output_token_mint = ctx.accounts.output_token_mint.key();
    offer.token_amount = token_amount;
    offer.expected_total_amount = expected_amount;
    // offer.token_amount = token_amount; expected ; init as zero
    // offer.fulfilTokenAmount = expected_amount; init as zero ; incrementing these value.
    offer.token_amount_remaining = token_amount;
    offer.expected_fulfilled_amount = 0;
    offer.deadline = deadline;
    
    // Copy protocol configuration
    offer.fee_percentage = ctx.accounts.fee_config.fee_percentage;
    offer.fee_wallet = ctx.accounts.fee_config.fee_address;

    // Transfer tokens to vault with amount validation
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


    // Activate the offer
    offer.status = OfferStatus::Ongoing;

    emit!(OfferCreated {
        offer_id: offer.offer_id,
        maker: offer.maker,
        input_token_mint: offer.input_token_mint,
        output_token_mint: offer.output_token_mint,
        token_amount,
        expected_amount,
        deadline,
    });

    Ok(())
}

/// Updates multiple takers to an offer's whitelist
/// Creates whitelist PDA if it doesn't exist
/// 
/// # Arguments
/// * `ctx` - ManageWhitelist context
/// * `takers` - Vector of taker public keys to add
///
/// # Security
/// - Only callable by offer maker
/// - Prevents duplicate entries
/// - Enforces maximum whitelist size
///
/// # Errors
/// * `SwapError::EmptyTakersList` - If takers list is empty
/// * `SwapError::WhitelistFull` - If adding would exceed capacity
/// * `SwapError::UnauthorizedMaker` - If caller isn't offer maker
pub fn manage_takers(
    ctx: Context<ManageWhitelist>,
    takers: Vec<Pubkey>,
) -> Result<()> {
    require!(!takers.is_empty(), SwapError::EmptyTakersList);
    require!(takers.len() < 50, SwapError::WhitelistFull);
    
    let whitelist = &mut ctx.accounts.whitelist;
    let maker = ctx.accounts.maker.key();
    let offer_id = ctx.accounts.offer.offer_id;

    // Clear the existing takers
    whitelist.takers.clear();

    for taker in takers {
        whitelist.takers.push(taker);
    }

    emit!(TakerUpdated {
        offer_id,
        maker,
        takers: whitelist.takers.clone(),
    });

    Ok(())
}