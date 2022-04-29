use std::convert::TryInto;

use near_contract_standards::non_fungible_token::{
    metadata::{
        NFTContractMetadata,
        NonFungibleTokenMetadataProvider,
        TokenMetadata,
        NFT_METADATA_SPEC,
    },
    Token,
    TokenId,
    NonFungibleToken,
};
use near_sdk::{borsh::{self, BorshDeserialize, BorshSerialize}, collections::UnorderedSet};
use near_sdk::collections::LazyOption;
use near_sdk::{
    ext_contract,
    env,
    near_bindgen,
    AccountId,
    BorshStorageKey,
    PanicOnDefault,
    Promise,
    PromiseOrValue,
    Gas,
    Balance,
};

pub use crate::external::*;
pub use crate::utils::*;
mod external;
mod mint;
mod owner;
mod utils;


const DATA_IMAGE_SVG_ICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 512 512' xml:space='preserve'%3E%3Cdefs/%3E%3CclipPath id='ArtboardFrame'%3E%3Crect height='512' width='512' x='0' y='0'/%3E%3C/clipPath%3E%3Cg clip-path='url(%23ArtboardFrame)'%3E%3Cpath d='M13.3866 157.467L160.015 75.543L200.383 215.639L292.401 36.3624L379.217 21.8194L385.004 101.962L412.167 15.5799L511.452 0L269.243 316.555L340.479 480.403L206.019 512.458L167.734 413.616L102.728 512.458L3 480.403L100.061 331.701L13.3866 157.467Z' fill='%23d512f6' fill-rule='evenodd' opacity='1' stroke='none'/%3E%3C/g%3E%3C/svg%3E";

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    next_token_id: u64,
    max_supply: u64,
    tokens: NonFungibleToken,
    operators: UnorderedSet<AccountId>,
    metadata: LazyOption<NFTContractMetadata>,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
    Operator,
}

#[near_bindgen]
impl Contract {
    /// Initializes the contract owned by `owner_id` with
    /// default metadata (for example purposes only).
    #[init]
    pub fn new_default_meta(max_supply: u64, name: String, symbol: String) -> Self {
        Self::new(
            max_supply,
            NFTContractMetadata {
                spec: NFT_METADATA_SPEC.to_string(),
                name,
                symbol,
                icon: Some(DATA_IMAGE_SVG_ICON.to_string()),
                base_uri: None,
                reference: None,
                reference_hash: None,
            },
        )
    }

    #[init]
    pub fn new(max_supply: u64, metadata: NFTContractMetadata) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        Self {
            next_token_id: 1,
            max_supply,
            tokens: NonFungibleToken::new(
                StorageKey::NonFungibleToken,
                env::current_account_id().to_string().try_into().unwrap(),
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval),
            ),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
            operators: UnorderedSet::new(StorageKey::Operator),
        }
    }
}

near_contract_standards::impl_non_fungible_token_core!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_approval!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_enumeration!(Contract, tokens);

#[near_bindgen]
impl NonFungibleTokenMetadataProvider for Contract {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }
}
