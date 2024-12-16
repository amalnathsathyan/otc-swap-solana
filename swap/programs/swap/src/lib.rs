use anchor_lang::prelude::*;
pub mod instructions;
pub mod error;
pub mod state;

pub use instructions::*;
pub use error::*;
pub use state::*;

declare_id!("BDurA1PZPYYD3SnhRDxnd592fCUDYWFHGGakLVbixp5S");

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

    // Maker Functions
    pub fn create_offer_and_send_tokens_to_vault(
        ctx: Context<CreateOffer>,
        offer_id:u64,
        token_amount: u64,
        expected_total_amount: u64,
        deadline: i64,
    ) -> Result<()> {
        instructions::create_offer::initialize_offer(ctx, offer_id, token_amount, expected_total_amount, deadline)
    }

    pub fn manage_whitelist(
        ctx: Context<ManageWhitelist>,
        takers: Vec<Pubkey>,
    ) -> Result<()> {
        instructions::create_offer::manage_takers(ctx, takers)
    }

    ///if incase, it's completed shouldn't be able to call this
    pub fn cancel_offer(ctx: Context<CancelOffer>) -> Result<()> {
        instructions::cancel_offer::update_cancel_offer(ctx)
    }

    // Taker Function
    pub fn take_offer(
        ctx: Context<TakeOffer>, 
        input_token_amount: u64 //token_a
    ) -> Result<()> {
        instructions::taker_offer::process(ctx, input_token_amount)
    }
}
