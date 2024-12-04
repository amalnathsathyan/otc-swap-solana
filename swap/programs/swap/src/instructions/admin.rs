use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::*;

/// Account validation struct for protocol initialization
/// Creates and initializes all configuration PDAs
#[derive(Accounts)]
pub struct Initialize<'info> {
    /// The admin account that will control protocol settings
    #[account(mut)]
    pub admin: Signer<'info>,
    
    /// PDA storing admin details and protocol statistics
    /// Space breakdown:
    /// - 8 bytes discriminator
    /// - 32 bytes admin pubkey
    /// - 8 bytes total offers
    /// - 8 bytes active offers
    /// - 8 bytes completed offers
    /// - 8 bytes cancelled offers
    /// - 8 bytes expired offers
    /// - 8 bytes last expiry check timestamp
    #[account(
        init,
        payer = admin,
        space = 8 + 32 + 8 + 8 + 8 + 8 + 8 + 8,
        seeds = [b"admin_config"],
        bump
    )]
    pub admin_config: Account<'info, AdminConfig>,

    /// PDA storing fee configuration
    /// Space breakdown:
    /// - 8 bytes discriminator
    /// - 8 bytes fee percentage
    /// - 32 bytes fee wallet address
    /// - 1 byte bump
    #[account(
        init,
        payer = admin,
        space = 8 + 8 + 32 + 1,
        seeds = [b"fee"],
        bump
    )]
    pub fee_config: Account<'info, FeeConfig>,

    /// PDA storing whitelist enforcement configuration
    /// Space breakdown:
    /// - 8 bytes discriminator
    /// - 1 byte boolean flag
    #[account(
        init,
        payer = admin,
        space = 8 + 1,
        seeds = [b"whitelist_config"],
        bump
    )]
    pub whitelist_config: Account<'info, WhitelistConfig>,

    /// PDA storing whitelisted token mints
    /// Space breakdown:
    /// - 8 bytes discriminator
    /// - 4 bytes vector length
    /// - 32 * 50 bytes for pubkeys (max 50 mints)
    #[account(
        init,
        payer = admin,
        space = 8 + 4 + (32 * 50),
        seeds = [b"mint_whitelist"],
        bump
    )]
    pub mint_whitelist: Account<'info, MintWhitelist>,

    pub system_program: Program<'info, System>,
}

/// Account validation struct for updating fee recipient address
#[derive(Accounts)]
pub struct UpdateFeeAddress<'info> {
    /// Admin signer
    #[account(mut)]
    pub admin: Signer<'info>,

    /// PDA verifying admin authority
    #[account(
        seeds = [b"admin_config"],
        bump,
        constraint = admin_config.admin == admin.key() @ SwapError::UnauthorizedAdmin
    )]
    pub admin_config: Account<'info, AdminConfig>,

    /// Fee configuration to update
    #[account(
        mut,
        seeds = [b"fee"],
        bump,
    )]
    pub fee_config: Account<'info, FeeConfig>,

    pub system_program: Program<'info, System>,
}

/// Account validation struct for updating fee percentage
#[derive(Accounts)]
pub struct UpdateFee<'info> {
    /// Admin signer
    #[account(mut)]
    pub admin: Signer<'info>,

    /// PDA verifying admin authority
    #[account(
        seeds = [b"admin_config"],
        bump,
        constraint = admin_config.admin == admin.key() @ SwapError::UnauthorizedAdmin
    )]
    pub admin_config: Account<'info, AdminConfig>,

    /// Fee configuration to update
    #[account(
        mut,
        seeds = [b"fee"],
        bump,
    )]
    pub fee_config: Account<'info, FeeConfig>,

    pub system_program: Program<'info, System>,
}

/// Account validation struct for toggling whitelist requirement
#[derive(Accounts)]
pub struct ToggleRequireWhitelist<'info> {
    /// Admin signer
    #[account(mut)]
    pub admin: Signer<'info>,

    /// PDA verifying admin authority
    #[account(
        seeds = [b"admin_config"],
        bump,
        constraint = admin_config.admin == admin.key() @ SwapError::UnauthorizedAdmin
    )]
    pub admin_config: Account<'info, AdminConfig>,

    /// Whitelist configuration to toggle
    #[account(
        mut,
        seeds = [b"whitelist_config"],
        bump,
    )]
    pub whitelist_config: Account<'info, WhitelistConfig>,

    pub system_program: Program<'info, System>,
}

/// Account validation struct for modifying mint whitelist
#[derive(Accounts)]
pub struct ModifyMintWhitelist<'info> {
    /// Admin signer
    #[account(mut)]
    pub admin: Signer<'info>,

    /// PDA verifying admin authority
    #[account(
        seeds = [b"admin_config"],
        bump,
        constraint = admin_config.admin == admin.key() @ SwapError::UnauthorizedAdmin
    )]
    pub admin_config: Account<'info, AdminConfig>,

    /// Mint whitelist to modify
    #[account(
        mut,
        seeds = [b"mint_whitelist"],
        bump,
    )]
    pub mint_whitelist: Account<'info, MintWhitelist>,

    pub system_program: Program<'info, System>,
}

/// Account validation struct for expiring an offer
#[derive(Accounts)]
pub struct CheckExpiredOffer<'info> {
    /// Admin signer for authorization
    #[account(mut)]
    pub admin: Signer<'info>,

    /// Admin config account for tracking offer states
    #[account(
        mut,
        seeds = [b"admin_config"],
        bump,
        constraint = admin_config.admin == admin.key() @ SwapError::UnauthorizedAdmin
    )]
    pub admin_config: Account<'info, AdminConfig>,

    /// Offer account to be marked as expired
    #[account(
        mut,
        constraint = Clock::get()?.unix_timestamp > offer.deadline @ SwapError::OfferNotExpired,
        constraint = offer.status == OfferStatus::Ongoing @ SwapError::InvalidOfferStatus
    )]
    pub offer: Account<'info, Offer>,

    pub system_program: Program<'info, System>,
}

/// Initializes the protocol with administrative settings
///
/// # Arguments
/// * `ctx` - Initialize context containing required accounts
/// * `fee_percentage` - Initial fee in basis points (1/100th of a percent)
/// * `fee_wallet` - Address to receive protocol fees
/// * `require_whitelist` - Whether to enforce token mint whitelist
/// * `initial_mints` - Initial set of whitelisted token mints
///
/// # Errors
/// * `SwapError::InvalidFeePercentage` - If fee exceeds 100%
/// * `SwapError::TooManyMints` - If initial mints exceed capacity
pub fn initialize(
    ctx: Context<Initialize>,
    fee_percentage: u64,
    fee_wallet: Pubkey,
    require_whitelist: bool,
    initial_mints: Vec<Pubkey>,
) -> Result<()> {
    require!(fee_percentage <= 10000, SwapError::InvalidFeePercentage);
    require!(initial_mints.len() <= 50, SwapError::TooManyMints);
    
    // Initialize admin configuration
    let admin_config = &mut ctx.accounts.admin_config;
    admin_config.admin = ctx.accounts.admin.key();
    admin_config.total_offers = 0;
    admin_config.active_offers = 0;
    admin_config.completed_offers = 0;
    admin_config.cancelled_offers = 0;
    admin_config.expired_offers = 0;
    admin_config.last_expiry_check = Clock::get()?.unix_timestamp;
    
    // Initialize fee configuration
    let fee_config = &mut ctx.accounts.fee_config;
    fee_config.fee_percentage = fee_percentage;
    fee_config.fee_address = fee_wallet;

    // Initialize whitelist configuration
    let whitelist_config = &mut ctx.accounts.whitelist_config;
    whitelist_config.require_whitelist = require_whitelist;

    // Initialize mint whitelist
    let mint_whitelist = &mut ctx.accounts.mint_whitelist;
    mint_whitelist.mints = initial_mints;
    
    Ok(())
}

/// Updates the protocol fee recipient address
///
/// # Arguments
/// * `ctx` - UpdateFeeAddress context
/// * `new_address` - New fee recipient address
pub fn fee_address_update(
    ctx: Context<UpdateFeeAddress>, 
    new_address: Pubkey
) -> Result<()> {
    ctx.accounts.fee_config.fee_address = new_address;
    Ok(())
}

/// Updates the protocol fee percentage
///
/// # Arguments
/// * `ctx` - UpdateFee context
/// * `new_fee` - New fee in basis points
///
/// # Errors
/// * `SwapError::InvalidFeePercentage` - If fee exceeds 100%
pub fn fee_update(
    ctx: Context<UpdateFee>, 
    new_fee: u64
) -> Result<()> {
    require!(new_fee <= 10000, SwapError::InvalidFeePercentage);
    ctx.accounts.fee_config.fee_percentage = new_fee;
    Ok(())
}

/// Toggles the token mint whitelist requirement
///
/// # Arguments
/// * `ctx` - ToggleRequireWhitelist context
pub fn update_toggle_whitelist(
    ctx: Context<ToggleRequireWhitelist>
) -> Result<()> {
    ctx.accounts.whitelist_config.require_whitelist = !ctx.accounts.whitelist_config.require_whitelist;
    Ok(())
}

/// Adds multiple token mints to the whitelist
///
/// # Arguments
/// * `ctx` - ModifyMintWhitelist context
/// * `new_mints` - Vector of mint addresses to add
///
/// # Errors
/// * `SwapError::TooManyMints` - If adding mints would exceed capacity
pub fn add_mints(
    ctx: Context<ModifyMintWhitelist>,
    new_mints: Vec<Pubkey>,
) -> Result<()> {
    require!(
        ctx.accounts.mint_whitelist.mints.len() + new_mints.len() <= 50,
        SwapError::TooManyMints
    );

    for mint in new_mints {
        if !ctx.accounts.mint_whitelist.mints.contains(&mint) {
            ctx.accounts.mint_whitelist.mints.push(mint);
        }
    }
    Ok(())
}

/// Removes multiple token mints from the whitelist
///
/// # Arguments
/// * `ctx` - ModifyMintWhitelist context
/// * `remove_mints` - Vector of mint addresses to remove
pub fn remove_mints(
    ctx: Context<ModifyMintWhitelist>,
    remove_mints: Vec<Pubkey>,
) -> Result<()> {
    ctx.accounts.mint_whitelist.mints.retain(|mint| !remove_mints.contains(mint));
    Ok(())
}

/// Marks an expired offer as Expired status
/// Only callable by admin when offer deadline has passed
///
/// # Arguments
/// * `ctx` - CheckExpiredOffer context containing admin and offer accounts
///
/// # Errors
/// * `SwapError::UnauthorizedAdmin` - If caller is not the admin
/// * `SwapError::OfferNotExpired` - If offer deadline hasn't passed
/// * `SwapError::InvalidOfferStatus` - If offer is not in Ongoing status
pub fn update_expire_offer(ctx: Context<CheckExpiredOffer>) -> Result<()> {
    let admin_config = &mut ctx.accounts.admin_config;
    let offer = &mut ctx.accounts.offer;

    // Update admin tracking stats
    admin_config.update_offer_status(Some(offer.status), OfferStatus::Expired);
    
    // Mark offer as expired
    offer.status = OfferStatus::Expired;
    admin_config.last_expiry_check = Clock::get()?.unix_timestamp;

    Ok(())
}
