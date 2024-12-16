use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::*;

#[event]
pub struct AdminInitialized {
    pub admin: Pubkey,
    pub fee_percentage: u64,
    pub fee_wallet: Pubkey,
    pub require_whitelist: bool,
    pub initial_mints: Vec<Pubkey>,
    pub timestamp: i64,
}

#[event]
pub struct FeeUpdated {
    pub admin: Pubkey,
    pub old_fee: u64,
    pub new_fee: u64,
    pub timestamp: i64,
}

#[event]
pub struct FeeWalletUpdated {
    pub admin: Pubkey,
    pub old_wallet: Pubkey,
    pub new_wallet: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct WhitelistRequirementToggled {
    pub admin: Pubkey,
    pub new_status: bool,
    pub timestamp: i64,
}

#[event]
pub struct MintsAddedToWhitelist {
    pub admin: Pubkey,
    pub new_mints: Vec<Pubkey>,
    pub timestamp: i64,
}

#[event]
pub struct MintsRemovedFromWhitelist {
    pub admin: Pubkey,
    pub removed_mints: Vec<Pubkey>,
    pub timestamp: i64,
}


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
    #[account(
        init,
        payer = admin,
        space = 8 + 32,
        seeds = [b"admin_config"],
        bump
    )]
    pub admin_config: Account<'info, AdminConfig>,

    /// PDA storing fee configuration
    /// Space breakdown:
    /// - 8 bytes discriminator
    /// - 8 bytes fee percentage
    /// - 32 bytes fee wallet address
    #[account(
        init,
        payer = admin,
        space = 8 + 8 + 32,
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

    /// Global PDA for tracking maker sequences
    // #[account(
    //     init,
    //     payer = admin,
    //     space = 8 + 32,  // discriminator + pubkey
    //     seeds = [b"maker_sequence_pda"],
    //     bump
    // )]
    // pub maker_sequence_pda: Account<'info, MakerSequence>,

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
    require!(fee_wallet != Pubkey::default(), SwapError::InvalidAddress);
    
    // Initialize admin configuration
    let admin_config = &mut ctx.accounts.admin_config;
    admin_config.admin = ctx.accounts.admin.key();
    // admin_config.total_offers = 0;
    // admin_config.active_offers = 0;
    // admin_config.completed_offers = 0;
    // admin_config.cancelled_offers = 0;
    // admin_config.expired_offers = 0;
    // admin_config.last_expiry_check = Clock::get()?.unix_timestamp;
    // admin_config.maker_sequence_pda = ctx.accounts.maker_sequence_pda.key();
    
    // Initialize fee configuration
    let fee_config = &mut ctx.accounts.fee_config;
    fee_config.fee_percentage = fee_percentage;
    fee_config.fee_address = fee_wallet;

    // Initialize whitelist configuration
    let whitelist_config = &mut ctx.accounts.whitelist_config;
    whitelist_config.require_whitelist = require_whitelist;

    // Initialize mint whitelist
    let mint_whitelist = &mut ctx.accounts.mint_whitelist;
    mint_whitelist.mints = initial_mints.clone();

    // // Initialize maker sequence PDA
    // let maker_sequence = &mut ctx.accounts.maker_sequence_pda;
    // maker_sequence.maker = Pubkey::default();  // Will be set on first offer
    // maker_sequence.offer_count = 0;

    emit!(AdminInitialized {
        admin: ctx.accounts.admin.key(),
        fee_percentage,
        fee_wallet,
        require_whitelist,
        initial_mints: initial_mints,
        timestamp: Clock::get()?.unix_timestamp,
    });
    
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
    require!(new_address != Pubkey::default(), SwapError::InvalidAddress);
    let old_wallet = ctx.accounts.fee_config.fee_address;
    ctx.accounts.fee_config.fee_address = new_address;

    emit!(FeeWalletUpdated {
        admin: ctx.accounts.admin.key(),
        old_wallet,
        new_wallet: new_address,
        timestamp: Clock::get()?.unix_timestamp,
    });

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
    let old_fee = ctx.accounts.fee_config.fee_percentage;
    
    require!(new_fee <= 10000, SwapError::InvalidFeePercentage);
    ctx.accounts.fee_config.fee_percentage = new_fee;

    emit!(FeeUpdated {
        admin: ctx.accounts.admin.key(),
        old_fee,
        new_fee,
        timestamp: Clock::get()?.unix_timestamp,
    });
    
    Ok(())
}

/// Toggles the token mint whitelist requirement
///
/// # Arguments
/// * `ctx` - ToggleRequireWhitelist context
pub fn update_toggle_whitelist(
    ctx: Context<ToggleRequireWhitelist>
) -> Result<()> {
    let whitelist_config = &mut ctx.accounts.whitelist_config;
    whitelist_config.require_whitelist = !whitelist_config.require_whitelist;

    emit!(WhitelistRequirementToggled {
        admin: ctx.accounts.admin.key(),
        new_status: whitelist_config.require_whitelist,
        timestamp: Clock::get()?.unix_timestamp,
    });

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

    for mint in &new_mints {
        if !ctx.accounts.mint_whitelist.mints.contains(&mint) {
            ctx.accounts.mint_whitelist.mints.push(*mint);
        }
    }

    emit!(MintsAddedToWhitelist {
        admin: ctx.accounts.admin.key(),
        new_mints,
        timestamp: Clock::get()?.unix_timestamp,
    });

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
    let removed = remove_mints.clone();
    ctx.accounts.mint_whitelist.mints.retain(|mint| !remove_mints.contains(mint));

    emit!(MintsRemovedFromWhitelist {
        admin: ctx.accounts.admin.key(),
        removed_mints: removed,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}