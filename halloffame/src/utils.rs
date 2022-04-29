use crate::*;

pub const NO_DEPOSIT: Balance = 0;
pub const ONE: Balance = 1 * 10u128.pow(18);

pub const TGAS: Gas = 10u64.pow(12);
pub const GAS_FOR_RESOLVE_TRANSFER: Gas = 5 * TGAS;
pub const GAS_FOR_FT_TRANSFER_CALL: Gas = 25 * TGAS + GAS_FOR_RESOLVE_TRANSFER;
pub const GAS_FOR_NFT_TRANSFER_CALL: Gas = 20 * TGAS + GAS_FOR_RESOLVE_TRANSFER;
pub const GAS_FOR_NFT_MINT_CALL: Gas = 50 * TGAS + GAS_FOR_RESOLVE_TRANSFER;
