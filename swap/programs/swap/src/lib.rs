use anchor_lang::prelude::*;
pub mod instructions;
pub mod error;
pub mod state;

pub use instructions::*;
pub use error::*;
pub use state::*;

declare_id!("5JAQD9WVWqjKeeeybZ3cBfP59RrfBCrNeWGfeiLnrUxe");

#[program]
pub mod swap {

    use super::*;

    // Admin functions
    pub fn add_mint_to_whitelist(ctx: Context<ModifyMintWhitelist>, mint: Pubkey) -> Result<()> {
        ctx.accounts.add_mint(mint)
    }

    pub fn remove_mint_from_whitelist(ctx: Context<ModifyMintWhitelist>, mint: Pubkey) -> Result<()> {
        ctx.accounts.remove_mint(mint)
    }

    pub fn update_fee(ctx: Context<UpdateFee>, new_fee: u64) -> Result<()> {
        instructions::update_fee::fee_update(ctx, new_fee)
    }

    pub fn toggle_require_whitelist(ctx: Context<ToggleRequireWhitelist>) -> Result<()> {
        instructions::toggle_whitelist::update_toggle_whitelist(ctx)
    }

    pub fn update_fee_address(ctx: Context<UpdateFeeAddress>, new_address: Pubkey) -> Result<()> {
        instructions::update_fee_address::fee_address_update(ctx, new_address)
    }

    pub fn expire_offer(ctx: Context<CheckExpiredOffer>) -> Result<()> {
        instructions::expire_offer::update_expire_offer(ctx)
    }

    // Maker Functions
    pub fn create_offer_and_send_tokens_to_vault(
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
        instructions::create_offer::create_offer(ctx, id, input_token_mint, output_token_mint, token_amount, expected_total_amount, deadline, fee_percentage, fee_wallet)
    }

    pub fn add_taker_whitelist(
        ctx: Context<CreateOffer>,
        taker: Pubkey,
    ) -> Result<()> {
        instructions::create_offer::add_taker(ctx, taker)
    }

    pub fn remove_taker_whitelist(
        ctx: Context<CreateOffer>,
        taker: Pubkey,
    ) -> Result<()> {
        instructions::create_offer::remove_taker(ctx, taker)
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
