use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::*;

/// Account constraints for expiring an offer
#[derive(Accounts)]
pub struct CheckExpiredOffer<'info> {
    /// Admin config account for tracking offer states
    #[account(
        mut,
        constraint = admin_config.admin == admin.key() @ SwapError::UnauthorizedAdmin
    )]
    pub admin_config: Account<'info, AdminConfig>,

    /// Admin signer for authorization
    #[account(mut)]
    pub admin: Signer<'info>,
    
    /// Offer account to be marked as expired
    #[account(
        mut,
        constraint = Clock::get()?.unix_timestamp > offer.deadline @ SwapError::OfferNotExpired,
        constraint = offer.status == OfferStatus::Ongoing @ SwapError::InvalidOfferStatus
    )]
    pub offer: Account<'info, Offer>,
}

/// Marks an expired offer as Expired status
/// Only callable by admin when offer deadline has passed
pub fn update_expire_offer(ctx: Context<CheckExpiredOffer>) -> Result<()> {
    let admin_config = &mut ctx.accounts.admin_config;
    let offer = &mut ctx.accounts.offer;
    
    admin_config.update_offer_status(Some(offer.status), OfferStatus::Expired);
    offer.status = OfferStatus::Expired;
    
    Ok(())
}