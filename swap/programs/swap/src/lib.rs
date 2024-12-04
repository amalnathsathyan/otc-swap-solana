use anchor_lang::prelude::*;
pub mod instructions;
pub mod error;
pub mod state;

pub use instructions::*;
pub use error::*;
pub use state::*;

declare_id!("24Su4NEPKHey1pnr7LS35h4hTzCUANfsysMT9oy5G8iU");

#[program]
pub mod swap {

    use super::*;

    // Admin functions
    pub fn initialize_admin(
        ctx: Context<Initialize>,
        fee_percentage: u64,
        fee_wallet: Pubkey,
        require_whitelist: bool,
        initial_mints: Vec<Pubkey>
    ) -> Result<()> {
        instructions::admin::initialize(ctx, fee_percentage, fee_wallet, require_whitelist, initial_mints)
    }

    pub fn add_mints_to_whitelist(
        ctx: Context<ModifyMintWhitelist>,
        new_mints: Vec<Pubkey>,
    ) -> Result<()> {
        instructions::admin::add_mints(ctx, new_mints)
    }

    pub fn remove_mints_from_whitelist(
        ctx: Context<ModifyMintWhitelist>,
        remove_mints: Vec<Pubkey>,
    ) -> Result<()> {
        instructions::admin::remove_mints(ctx, remove_mints)
    }

    pub fn update_fee_percentage(
        ctx: Context<UpdateFee>,
        new_fee: u64
    ) -> Result<()> {
        instructions::admin::fee_update(ctx, new_fee)
    }

    pub fn update_fee_address(
        ctx: Context<UpdateFeeAddress>,
        new_address: Pubkey
    ) -> Result<()> {
        instructions::admin::fee_address_update(ctx, new_address)
    }

    pub fn toggle_require_whitelist(
        ctx: Context<ToggleRequireWhitelist>
    ) -> Result<()> {
        instructions::admin::update_toggle_whitelist(ctx)
    }

    pub fn expire_offer(
        ctx: Context<CheckExpiredOffer>
    ) -> Result<()> {
        instructions::admin::update_expire_offer(ctx)
    }

    // Maker Functions
    pub fn create_offer_and_send_tokens_to_vault(
        ctx: Context<CreateOffer>,
        token_amount: u64,
        expected_total_amount: u64,
        deadline: i64,
        initial_takers: Vec<Pubkey>
    ) -> Result<()> {
        instructions::create_offer::create_offer(ctx, token_amount, expected_total_amount, deadline,initial_takers)
    }

    pub fn add_taker_whitelist(
        ctx: Context<UpdateWhitelist>,
        taker: Vec<Pubkey>,
    ) -> Result<()> {
        instructions::update_taker_whitelist::add_takers(ctx, taker)
    }

    pub fn remove_taker_whitelist(
        ctx: Context<UpdateWhitelist>,
        taker: Vec<Pubkey>,
    ) -> Result<()> {
        instructions::update_taker_whitelist::remove_takers(ctx, taker)
    }

    pub fn cancel_offer(ctx: Context<CancelOffer>) -> Result<()> {
        instructions::cancel_offer::update_cancel_offer(ctx)
    }

    // Taker Function
    pub fn take_offer(
        ctx: Context<TakeOffer>, 
        token_amount: u64
    ) -> Result<()> {
        instructions::taker_offer::process(ctx, token_amount)
    }
}
