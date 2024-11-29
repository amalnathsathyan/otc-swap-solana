use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Trade {
    pub offer_id: Pubkey,
    pub taker: Pubkey,
    pub in_amount: u64,
    pub out_amount: u64,
    pub timestamp: u64,
    pub fee_amount: u64,
}