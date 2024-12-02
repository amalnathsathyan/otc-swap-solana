use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::*;

/// Account structure for modifying the whitelist of allowed takers for a specific offer
/// Allows the maker to control which addresses can accept their offer
#[derive(Accounts)]
pub struct ModifyWhitelist<'info> {
    /// The maker (creator) of the offer who controls the whitelist
    /// Must be the signer of the transaction
    #[account(mut)]
    pub maker: Signer<'info>,

    /// The whitelist account storing allowed taker addresses
    /// Must be associated with the specified offer
    #[account(
        mut,
        constraint = whitelist.offer == offer.key()
    )]
    pub whitelist: Account<'info, Whitelist>,

    /// The offer account that this whitelist is associated with
    pub offer: Account<'info, Offer>,

    /// The offer account that this whitelist is associated with
    pub system_program: Program<'info, System>,
}

impl<'info> ModifyWhitelist<'info> {
    /// Adds a new taker address to the whitelist
    ///
    /// # Arguments
    /// * `taker` - The public key of the address to be added to the whitelist
    ///
    /// # Errors
    /// Returns SwapError::TakerAlreadyWhitelisted if the taker is already in the whitelist
    ///
    /// # Returns
    /// * `Result<()>` - Ok if the taker was successfully added
    pub fn add_taker(&mut self, taker: Pubkey) -> Result<()> {

        // Verify taker is not already whitelisted
        require!(!self.whitelist.takers.contains(&taker), SwapError::TakerAlreadyWhitelisted);

        // Add the new taker to the whitelist
        self.whitelist.takers.push(taker);
        Ok(())
    }

    /// Removes a taker address from the whitelist
    ///
    /// # Arguments
    /// * `taker` - The public key of the address to remove from the whitelist
    ///
    /// # Errors
    /// Returns SwapError::TakerNotWhitelisted if the taker is not found in the whitelist
    ///
    /// # Returns
    /// * `Result<()>` - Ok if the taker was successfully removed
    pub fn remove_taker(&mut self, taker: Pubkey) -> Result<()> {

        // Find the position of the taker in the whitelist
        let position = self.whitelist.takers.iter().position(|x| x == &taker)
            .ok_or(SwapError::TakerNotWhitelisted)?;

        // Remove the taker from the whitelist
        self.whitelist.takers.remove(position);
        Ok(())
    }
}