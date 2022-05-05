use std::collections::HashMap;

use crate::*;
use near_contract_standards::non_fungible_token::{
    core::StorageKey, core::StorageKey as NftStorageKey, events::NftMint, refund_deposit_to_account,
};
use near_sdk::{near_bindgen, serde_json::json};

impl Contract {
    pub fn internal_mint_without_refund(
        &mut self,
        token_id: TokenId,
        token_owner_id: AccountId,
        token_metadata: Option<TokenMetadata>,
        refund_id: Option<AccountId>,
    ) -> Token {
        if self.tokens.token_metadata_by_id.is_some() && token_metadata.is_none() {
            env::panic_str("Must provide metadata");
        }
        if self.tokens.owner_by_id.get(&token_id).is_some() {
            env::panic_str("token_id must be unique");
        }

        let owner_id: AccountId = token_owner_id;

        // Core behavior: every token must have an owner
        self.tokens.owner_by_id.insert(&token_id, &owner_id);

        // Metadata extension: Save metadata, keep variable around to return later.
        // Note that check above already panicked if metadata extension in use but no metadata
        // provided to call.
        self.tokens
            .token_metadata_by_id
            .as_mut()
            .and_then(|by_id| by_id.insert(&token_id, token_metadata.as_ref().unwrap()));

        // Enumeration extension: Record tokens_per_owner for use with enumeration view methods.
        if let Some(tokens_per_owner) = &mut self.tokens.tokens_per_owner {
            let mut token_ids = tokens_per_owner.get(&owner_id).unwrap_or_else(|| {
                UnorderedSet::new(NftStorageKey::TokensPerOwner {
                    account_hash: env::sha256(owner_id.as_bytes()),
                })
            });
            token_ids.insert(&token_id);
            tokens_per_owner.insert(&owner_id, &token_ids);
        }

        // Approval Management extension: return empty HashMap as part of Token
        let approved_account_ids =
            if self.tokens.approvals_by_id.is_some() { Some(HashMap::new()) } else { None };

        Token { token_id, owner_id, metadata: token_metadata, approved_account_ids }
    }
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn nft_mints(&mut self, receiver_id: AccountId, amount: u32) -> Vec<Token> {
        self.assert_owner_or_operator();
        assert!(
            self.next_token_id + amount as u64 - 1 as u64 <= self.max_supply,
            "Player, try again next time"
        );
        let initial_storage_usage = env::storage_usage();
        let tokens = (0..amount)
            .map(|_| {
                let token_id = self.next_token_id;
                let token_metadata = TokenMetadata {
                    title: Some("Exverse Pass".to_string()),
                    description: None,
                    media: Some("QmTWewETfuHsP3EXJ6zYh1Us6uFs75rXnvyk2ktbidhZmu".to_string()),
                    media_hash: None,
                    copies: Some(1),
                    issued_at: Some(env::block_timestamp().to_string()),
                    expires_at: None,
                    starts_at: None,
                    updated_at: None,
                    extra: None,
                    reference: Some(
                        format!("QmcjcieB2WvqEQiviJUsfdQ8FqMJT78kobbJgnxE2iK3DG/{}", token_id)
                            .to_string(),
                    ),
                    reference_hash: None,
                };
                let token = self.internal_mint_without_refund(
                    token_id.to_string(),
                    receiver_id.clone(),
                    Some(token_metadata),
                    Some(env::predecessor_account_id()),
                );
                self.next_token_id += 1;
                NftMint { owner_id: &token.owner_id, token_ids: &[&token.token_id], memo: None }
                    .emit();
                token
            })
            .collect();
        refund_deposit_to_account(
            env::storage_usage() - initial_storage_usage,
            env::predecessor_account_id(),
        );
        tokens
    }
}
