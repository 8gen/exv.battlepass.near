use crate::*;

pub const NO_DEPOSIT: Balance = 0;
pub const ONE: Balance = 1 * 10u128.pow(18);

pub const TGAS: Gas = Gas(10u64.pow(12));
pub const GAS_FOR_RESOLVE_TRANSFER: Gas = Gas(5_000_000_000_000);
pub const GAS_FOR_FT_TRANSFER_CALL: Gas = Gas(30_000_000_000_000);
pub const GAS_FOR_NFT_TRANSFER_CALL: Gas = Gas(25_000_000_000_000);
pub const GAS_FOR_NFT_MINT_CALL: Gas = Gas(55_000_000_000_000);
