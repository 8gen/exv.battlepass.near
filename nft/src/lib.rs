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
    NonFungibleToken, refund_deposit,
};
use near_sdk::{borsh::{self, BorshDeserialize, BorshSerialize}, collections::UnorderedSet};
use near_sdk::collections::LazyOption;
use near_sdk::json_types::ValidAccountId;
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

near_sdk::setup_alloc!();


const DATA_IMAGE_SVG_ICON: &str = "data:image/svg+xml,%3Csvg%20viewBox%3D%220%200%20512%20512%22%20xml%3Aspace%3D%22preserve%22%3E%3Cdefs/%3E%3CclipPath%20id%3D%22ArtboardFrame%22%3E%3Crect%20height%3D%22512%22%20width%3D%22512%22%20x%3D%220%22%20y%3D%220%22/%3E%3C/clipPath%3E%3Cg%20clip-path%3D%22url%28%23ArtboardFrame%29%22%3E%3Cpath%20d%3D%22M13.3866%20157.467L160.015%2075.543L200.383%20215.639L292.401%2036.3624L379.217%2021.8194L385.004%20101.962L412.167%2015.5799L511.452%200L269.243%20316.555L340.479%20480.403L206.019%20512.458L167.734%20413.616L102.728%20512.458L3%20480.403L100.061%20331.701L13.3866%20157.467Z%22%20fill%3D%22%23d512f6%22%20fill-rule%3D%22evenodd%22%20opacity%3D%221%22%20stroke%3D%22none%22/%3E%3C/g%3E%3C/svg%3E";


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
