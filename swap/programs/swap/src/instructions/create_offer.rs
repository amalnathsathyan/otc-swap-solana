use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};
use crate::state::*;
use crate::error::*;

/// Account structure for creating and managing token swap offers
/// Takes an offer_id instruction argument for unique identification
#[derive(Accounts)]
#[instruction(offer_id: u64)]
pub struct CreateOffer<'info> {
    /// Offer creator who deposits tokens and controls whitelist
    #[account(mut)]
    pub maker: Signer<'info>,

    /// PDA storing offer details
    /// Space: discriminator(8) + maker(32) + input_mint(32) + output_mint(32) + 
    /// amount(8) + expected_amount(8) + deadline(8) + fee_pct(8) + fee_wallet(32) + status(1)
    #[account(
        init,
        payer = maker,
        space = 8 + 32 + 32 + 32 + 8 + 8 + 8 + 8 + 32 + 1,
        seeds = [
            b"offer",
            maker.key().as_ref(),
            offer_id.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub offer: Account<'info, Offer>,

    /// PDA storing allowed taker addresses
    /// Space: discriminator(8) + offer(32) + maker(32) + vec_len(4) + takers(32 * 10)
    #[account(
        init,
        payer = maker,
        space = 8 + 32 + 32 + 4 + (32 * 10),
        constraint = whitelist.maker == maker.key() @ SwapError::UnauthorizedMaker,
        seeds = [
            b"whitelist",
            offer.key().as_ref()
        ],
        bump
    )]
    pub whitelist: Account<'info, Whitelist>,

    /// Input token being offered
    pub input_token_mint: Account<'info, token::Mint>,
    /// Output token requested
    pub output_token_mint: Account<'info, token::Mint>,

    /// PDA storing fee configuration
    /// Space: discriminator(8) + fee_percentage(8) + fee_address(32) + bump(1)
    #[account(
        init,
        payer = maker,
        space = 8 + 8 + 32 + 1,
        seeds = [
            b"fee_config",
            offer.key().as_ref()
        ],
        bump
    )]
    pub offer_fee_config: Account<'info, FeeConfig>,

    /// Maker's token account sending tokens
    #[account(mut)]
    pub maker_token_account: Account<'info, TokenAccount>,
    
    /// Vault token account receiving tokens
    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,

    /// PDA with authority over vault
    /// CHECK: vault authority seeds are mentioned
    #[account(
        seeds = [b"vault", input_token_mint.key().as_ref()],
        bump
    )]
    pub vault_authority: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

/// Creates a new token swap offer with specified parameters
///
/// # Arguments
/// * `ctx` - CreateOffer context containing required accounts
/// * `id` - Unique offer identifier 
/// * `input_token_mint` - Token mint being offered
/// * `output_token_mint` - Token mint requested in return
/// * `token_amount` - Amount of input tokens
/// * `expected_total_amount` - Amount of output tokens expected
/// * `deadline` - Offer expiration timestamp
/// * `fee_percentage` - Fee percentage for the swap
/// * `fee_wallet` - Wallet to receive fees
///
/// # Steps
/// 1. Initializes offer account with provided parameters
/// 2. Sets up fee configuration
/// 3. Creates empty whitelist
/// 4. Transfers tokens from maker to vault
pub fn create_offer(
    ctx: Context<CreateOffer>,
    id: u64,
    input_token_mint: Pubkey,
    output_token_mint: Pubkey,
    token_amount: u64,
    expected_total_amount: u64,
    deadline: i64,
    fee_percentage: u64,
    fee_wallet: Pubkey,
) -> Result<()> {
    let offer = &mut ctx.accounts.offer;
    offer.offer_id = id;
    offer.maker = ctx.accounts.maker.key();
    offer.input_token_mint = input_token_mint;
    offer.output_token_mint = output_token_mint;
    offer.token_amount = token_amount;
    offer.expected_total_amount = expected_total_amount;
    offer.deadline = deadline;
    offer.status = OfferStatus::Ongoing;
    offer.fee_percentage = fee_percentage;
    offer.fee_wallet = fee_wallet;

    let fee_config = &mut ctx.accounts.offer_fee_config;
    fee_config.fee_percentage = fee_percentage;
    fee_config.fee_address = fee_wallet;

    let whitelist = &mut ctx.accounts.whitelist;
    whitelist.offer = offer.key();
    whitelist.maker = ctx.accounts.maker.key();
    whitelist.takers = Vec::new();

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.maker_token_account.to_account_info(),
                to: ctx.accounts.vault_token_account.to_account_info(),
                authority: ctx.accounts.maker.to_account_info(),
            },
        ),
        token_amount,
    )?;

    Ok(())
}

/// Adds a taker address to the whitelist for offer access control
///
/// # Arguments
/// * `ctx` - CreateOffer context containing whitelist 
/// * `taker` - Address to add to whitelist
///
/// # Errors
/// * `SwapError::TakerAlreadyWhitelisted` - If taker already exists in whitelist
pub fn add_taker(ctx: Context<CreateOffer>, taker: Pubkey) -> Result<()> {
    require!(!ctx.accounts.whitelist.takers.contains(&taker), SwapError::TakerAlreadyWhitelisted);
    ctx.accounts.whitelist.takers.push(taker);
    Ok(())
}

/// Removes a taker address from the whitelist
///
/// # Arguments
/// * `ctx` - CreateOffer context containing whitelist
/// * `taker` - Address to remove from whitelist
///
/// # Errors
/// * `SwapError::TakerNotWhitelisted` - If taker not found in whitelist
pub fn remove_taker(ctx: Context<CreateOffer>, taker: Pubkey) -> Result<()> {
    let position = ctx.accounts.whitelist.takers.iter().position(|x| x == &taker)
        .ok_or(SwapError::TakerNotWhitelisted)?;
    ctx.accounts.whitelist.takers.remove(position);
    Ok(())
}