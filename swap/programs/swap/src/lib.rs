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
        ctx.accounts.process(new_fee)
        }

    pub fn toggle_require_whitelist(ctx: Context<ToggleRequireWhitelist>) -> Result<()> {
        ctx.accounts.process()
    }

    pub fn update_fee_address(ctx: Context<UpdateFeeAddress>, new_address: Pubkey) -> Result<()> {
        ctx.accounts.process(new_address)
    }

    // Maker Functions
    pub fn create_offer_and_send_tokens_to_vault(
        ctx: Context<CreateOffer>,
        token_mint: Pubkey,
        token_amount: u64,
        expected_total_amount: u64,
        deadline: i64,
    ) -> Result<()> {
        ctx.accounts.process(token_mint, token_amount, expected_total_amount, deadline)
    }

    pub fn add_taker_whitelist(
        ctx: Context<ModifyWhitelist>,
        taker: Pubkey,
    ) -> Result<()> {
        ctx.accounts.add_taker(taker)
    }

    pub fn remove_taker_whitelist(
        ctx: Context<ModifyWhitelist>,
        taker: Pubkey,
    ) -> Result<()> {
        ctx.accounts.remove_taker(taker)
    }

    pub fn cancel_offer(ctx: Context<CancelOffer>) -> Result<()> {
        ctx.accounts.process()
    }
}
