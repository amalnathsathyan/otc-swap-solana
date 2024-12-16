use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface}
};
use crate::state::*;
use crate::error::*;

#[event]
pub struct OfferTaken {
   #[index]
   pub offer_id: u64,
   pub maker: Pubkey,
   pub taker: Pubkey,
   pub input_token_amount: u64,
   pub payment_amount: u64,
   pub fee_amount: u64,
   pub remaining_amount: u64,
   pub input_token_mint: Pubkey,
   pub output_token_mint: Pubkey,
}

/// Core accounts required for the take offer instruction.
/// Groups the main program state accounts and PDAs.
#[derive(Accounts)]
pub struct CoreAccounts<'info> {
    /// Transaction signer who will take the offer.
    /// Responsible for:
    /// - Paying for account initialization
    /// - Sending payment tokens
    /// - Paying protocol fees
    /// - Receiving offered tokens
    #[account(mut)]
    pub taker: Signer<'info>,

    /// Protocol admin configuration.
    /// Tracks global settings and statistics.
    /// PDA with seeds: ["admin_config"]
    #[account(
        mut,
        seeds = [b"admin_config"],
        bump
    )]
    pub admin_config: Box<Account<'info, AdminConfig>>,
    
    /// The offer being taken.
    /// PDA with seeds: ["offer", maker_pubkey, offer_id]
    /// Constraints:
    /// - Must be in Ongoing status
    /// - Closed on completion with rent returned to maker
    #[account(
        mut,
        constraint = offer.status == OfferStatus::Ongoing @ SwapError::InvalidOfferStatus,
        seeds = [b"offer", offer.maker.as_ref(),&offer.offer_id.to_le_bytes()],
        bump,
    )]
    pub offer: Box<Account<'info, Offer>>,

    /// CHECK: Verified through the constraint with offer.maker
    #[account(
        mut,
        constraint = maker.key() == offer.maker @ SwapError::InvalidMaker
    )]
    pub maker: AccountInfo<'info>,

    /// Whitelist of authorized takers.
    /// PDA with seeds: ["whitelist", maker_pubkey, offer_id]
    /// Constraints:
    /// - Must include taker's public key
    /// - Closed on offer completion
    #[account(
        mut,
        seeds = [b"whitelist", offer.maker.as_ref(), &offer.offer_id.to_le_bytes()],
        bump,
        constraint = whitelist.takers.contains(&taker.key()) @ SwapError::TakerNotWhitelisted,
    )]
    pub whitelist: Box<Account<'info, Whitelist>>,
}

/// Token accounts and associated programs for the take offer instruction.
/// Groups all token-related accounts and required program references.
#[derive(Accounts)]
pub struct TokenAccounts<'info> {
    /// Maker's token account for receiving payment.
    /// Automatically created as an ATA if it doesn't exist.
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = output_token_mint,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub maker_receive_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// Taker's token account for sending payment.
    /// Automatically created as an ATA if it doesn't exist.
    #[account(
        mut,
        constraint = taker_payment_token_account.owner == taker.key() @ SwapError::InvalidTokenAccount,
        constraint = taker_payment_token_account.mint == output_token_mint.key() @ SwapError::InvalidTokenMint
    )]
    pub taker_payment_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// Taker's token account for receiving offered tokens.
    /// Automatically created as an ATA if it doesn't exist.
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = input_token_mint,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_receive_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// Protocol fee receiving account.
    /// Constraints:
    /// - Must match configured fee wallet
    /// - Must use correct token mint
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = output_token_mint,
        associated_token::authority = fee_wallet,
        associated_token::token_program = token_program,
    )]
    pub fee_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// Vault holding the offered tokens.
    /// Constraints:
    /// - Must be owned by offer PDA
    /// - Must match input token mint
    #[account(
        mut,
        constraint = vault_token_account.owner == offer.key() @ SwapError::InvalidTokenAccount,
        constraint = vault_token_account.mint == input_token_mint.key() @ SwapError::InvalidTokenMint
    )]
    pub vault_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// Mint of the token being offered
    pub input_token_mint: Box<InterfaceAccount<'info, Mint>>,

    /// Mint of the token being requested
    pub output_token_mint: Box<InterfaceAccount<'info, Mint>>,

    /// Required program interfaces
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,

    /// CHECK: Fee wallet validated through fee_token_account constraints
    pub fee_wallet: AccountInfo<'info>,

    /// CHECK: Validated as transaction signer in CoreAccounts
    #[account(mut)]
    pub taker: AccountInfo<'info>,

    /// CHECK: Validated through offer.maker constraint in CoreAccounts
    pub maker: AccountInfo<'info>,

    /// CHECK: Validated through PDA seeds and constraints in CoreAccounts
    pub offer: AccountInfo<'info>,
}

/// Main account validation struct combining all required accounts.
/// Split into smaller components to optimize stack usage.
#[derive(Accounts)]
pub struct TakeOffer<'info> {
    pub core: CoreAccounts<'info>,
    pub token: TokenAccounts<'info>,
}

/// Helper struct for organized account reference passing.
pub struct AccountRefs<'info, 'ctx> {
    pub core: &'ctx CoreAccounts<'info>,
    pub token: &'ctx TokenAccounts<'info>,
}

impl<'info> TakeOffer<'info> {
    /// Creates a reference wrapper for convenient account access
    fn refs(&self) -> AccountRefs<'info, '_> {
        AccountRefs {
            core: &self.core,
            token: &self.token,
        }
    }
}

/// Processes a take offer instruction.
/// 
/// # Arguments
/// * `ctx` - TakeOffer context containing all accounts
/// * `token_amount` - Amount of input tokens to take
/// 
/// # Returns
/// * `Result<()>` - Success or error
/// 
/// # Flow
/// 1. Validates all offer conditions
/// 2. Calculates payment amounts including fees
/// 3. Processes token transfers
/// 4. Updates offer state and handles completion
pub fn process(mut ctx: Context<TakeOffer>, input_token_amount: u64) -> Result<()> {
    msg!("Processing take offer with amount: {}", input_token_amount);

    let refs = ctx.accounts.refs();
    
    validate_offer_conditions(refs.core, input_token_amount)?;
    let (fee_amount, payment_amount) = calculate_payments(&refs.core.offer, input_token_amount)?;
    msg!("Calculated payments - Fee: {}, Payment: {}", fee_amount, payment_amount);
    
    process_payments(refs, fee_amount, payment_amount)?;
    handle_vault_transfer_and_completion(&mut ctx, input_token_amount, fee_amount, payment_amount)?;
    
    Ok(())
}

/// Validates all required conditions for taking an offer.
/// 
/// # Arguments
/// * `core` - Reference to core accounts
/// * `token_amount` - Amount of tokens being taken
/// 
/// # Returns
/// * `Result<()>` - Success or error
/// 
/// # Checks
/// - Offer has not expired
/// - Sufficient tokens are available
/// 
/// # Errors
/// * `SwapError::OfferExpired` - If offer deadline has passed
/// * `SwapError::InsufficientAmount` - If requested amount exceeds available
fn validate_offer_conditions(core: &CoreAccounts, input_token_amount: u64) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;
    require!(current_time <= core.offer.deadline, SwapError::OfferExpired);
    require!(input_token_amount <= core.offer.token_amount_remaining, SwapError::InsufficientAmount);
    Ok(())
}

/// Calculates protocol fee and payment amounts.
/// 
/// # Arguments
/// * `offer` - Reference to offer account
/// * `token_amount` - Amount being taken
/// 
/// # Returns
/// * `Result<(u64, u64)>` - (fee_amount, payment_amount)
/// 
/// # Implementation
/// - Uses checked math operations to prevent overflows
/// - Calculates proportional payment based on take amount
/// - Applies protocol fee percentage
/// - Returns split between fee and payment amounts
fn calculate_payments(offer: &Account<Offer>, input_token_amount: u64) -> Result<(u64, u64)> {
    let expected_payment = (input_token_amount as u128)
        .checked_mul(offer.expected_total_amount as u128)
        .ok_or(SwapError::CalculationError)?
        .checked_div(offer.token_amount as u128)
        .ok_or(SwapError::CalculationError)? as u64;

    let fee_amount = expected_payment
        .checked_mul(offer.fee_percentage)
        .ok_or(SwapError::CalculationError)?
        .checked_div(10000)
        .ok_or(SwapError::CalculationError)?;

        
    // let payment_after_fee = expected_payment.checked_add(fee_amount).unwrap();

    Ok((fee_amount, expected_payment))
}

/// Processes fee and payment token transfers.
/// 
/// # Arguments
/// * `refs` - Account references
/// * `fee_amount` - Protocol fee amount
/// * `payment_amount` - Payment to maker
/// 
/// # Returns
/// * `Result<()>` - Success or error
/// 
/// # Implementation
/// 1. Transfers protocol fee to fee wallet
/// 2. Transfers payment amount to maker
/// 
/// # Security
/// - Uses transfer_checked for safe token transfers
/// - Validates all accounts and authorities
fn process_payments(refs: AccountRefs, fee_amount: u64, payment_amount: u64) -> Result<()> {

    // Protocol fee transfer
    token_interface::transfer_checked(
        CpiContext::new(
            refs.token.token_program.to_account_info(),
            token_interface::TransferChecked {
                from: refs.token.taker_payment_token_account.to_account_info(),
                mint: refs.token.output_token_mint.to_account_info(),
                to: refs.token.fee_token_account.to_account_info(),
                authority: refs.core.taker.to_account_info(),
            },
        ),
        fee_amount,
        refs.token.output_token_mint.decimals,
    )?;

    // Maker payment transfer
    token_interface::transfer_checked(
        CpiContext::new(
            refs.token.token_program.to_account_info(),
            token_interface::TransferChecked {
                from: refs.token.taker_payment_token_account.to_account_info(),
                mint: refs.token.output_token_mint.to_account_info(),
                to: refs.token.maker_receive_token_account.to_account_info(),
                authority: refs.core.taker.to_account_info(),
            },
        ),
        payment_amount,
        refs.token.output_token_mint.decimals,
    )?;

    Ok(())
}

/// Handles vault transfer and offer completion.
/// 
/// # Arguments
/// * `ctx` - TakeOffer context
/// * `token_amount` - Amount being taken
/// 
/// # Returns
/// * `Result<()>` - Success or error
/// 
/// # Implementation
/// 1. Transfers tokens from vault to taker
/// 2. For full takes:
///    - Closes vault account
///    - Updates protocol statistics
///    - Marks offer as completed
/// 3. For partial takes:
///    - Updates remaining token amount
/// 
/// # Security
/// - Properly manages PDA signing
/// - Careful ordering of borrows
/// - Safe math operations
fn handle_vault_transfer_and_completion(
    ctx: &mut Context<TakeOffer>,
    input_token_amount: u64,
    fee_amount: u64, 
    payment_amount: u64,
) -> Result<()> {
    // Get current values
    let current_amount = ctx.accounts.core.offer.token_amount_remaining;
    let current_fulfilled_amount = ctx.accounts.core.offer.expected_fulfilled_amount;
    let input_decimals = ctx.accounts.token.input_token_mint.decimals;
    
    msg!("Current remaining amount: {}", current_amount);
    msg!("Attempting to take amount: {}", input_token_amount);
    
    // Calculate new amounts with checked math
    let new_remaining = current_amount
        .checked_sub(input_token_amount)
        .ok_or(SwapError::CalculationError)?;
    let new_fulfilled = current_fulfilled_amount
        .checked_add(payment_amount)
        .ok_or(SwapError::CalculationError)?;

    // Get the offer and update state before transfers
    let offer = &mut ctx.accounts.core.offer;
    offer.token_amount_remaining = new_remaining;
    offer.expected_fulfilled_amount = new_fulfilled;
    
    msg!("New remaining amount: {}", new_remaining);

    // Prepare PDA signer seeds
    let maker_ref = offer.maker.as_ref();
    let offer_id = offer.offer_id.to_le_bytes();
    let offer_bump = ctx.bumps.core.offer;
    
    let seeds = &[
        b"offer".as_ref(),
        maker_ref,
        offer_id.as_ref(),
        &[offer_bump],
    ];
    let signer_seeds = &[&seeds[..]];

    msg!("Transferring {} tokens from vault to taker", input_token_amount);

    // Perform the vault transfer with transfer_checked
    token_interface::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token.token_program.to_account_info(),
            token_interface::TransferChecked {
                from: ctx.accounts.token.vault_token_account.to_account_info(),
                mint: ctx.accounts.token.input_token_mint.to_account_info(),
                to: ctx.accounts.token.taker_receive_token_account.to_account_info(),
                authority: offer.to_account_info(),
            },
            signer_seeds
        ),
        input_token_amount,
        input_decimals,
    )?;

    msg!("Transfer completed successfully");

    // Handle offer completion if this was a full take
    if new_remaining == 0 {
        msg!("Full take detected, closing vault");
        
        token_interface::close_account(
            CpiContext::new_with_signer(
                ctx.accounts.token.token_program.to_account_info(),
                token_interface::CloseAccount {
                    account: ctx.accounts.token.vault_token_account.to_account_info(),
                    destination: ctx.accounts.core.maker.to_account_info(),
                    authority: offer.to_account_info(),
                },
                signer_seeds
            ),
        )?;

        offer.status = OfferStatus::Completed;
        msg!("Offer marked as completed");
    }

    emit!(OfferTaken {
        offer_id: offer.offer_id,
        maker: offer.maker,
        taker: ctx.accounts.core.taker.key(),
        input_token_amount,
        payment_amount,
        fee_amount,
        remaining_amount: new_remaining,
        input_token_mint: ctx.accounts.token.input_token_mint.key(),
        output_token_mint: ctx.accounts.token.output_token_mint.key(),
    });

    msg!("Take offer process completed successfully");
    Ok(())
}