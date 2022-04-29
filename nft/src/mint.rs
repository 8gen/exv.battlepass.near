use crate::*;
use near_sdk::{serde_json::{self, json}, near_bindgen, json_types::Base64VecU8};


#[derive(BorshStorageKey, BorshSerialize)]
enum TokensStorageKey {
    TokensPerOwner { account_hash: Vec<u8> },
}


impl Contract {
    pub fn internal_mint(
        &mut self,
        token_id: TokenId,
        token_owner_id: ValidAccountId,
        token_metadata: Option<TokenMetadata>,
    ) -> Token {
        let initial_storage_usage = env::storage_usage();
        if self.tokens.token_metadata_by_id.is_some() && token_metadata.is_none() {
            env::panic(b"Must provide metadata");
        }
        if self.tokens.owner_by_id.get(&token_id).is_some() {
            env::panic(b"token_id must be unique");
        }

        let owner_id: AccountId = token_owner_id.into();

        // Core behavior: every token must have an owner
        self.tokens.owner_by_id.insert(&token_id, &owner_id);

        // Metadata extension: Save metadata, keep variable around to return later.
        // Note that check above already panicked if metadata extension in use but no metadata
        // provided to call.
        self.tokens.token_metadata_by_id
            .as_mut()
            .and_then(|by_id| by_id.insert(&token_id, &token_metadata.as_ref().unwrap()));

        // Enumeration extension: Record tokens_per_owner for use with enumeration view methods.
        if let Some(tokens_per_owner) = &mut self.tokens.tokens_per_owner {
            let mut token_ids = tokens_per_owner.get(&owner_id).unwrap_or_else(|| {
                UnorderedSet::new(TokensStorageKey::TokensPerOwner {
                    account_hash: env::sha256(owner_id.as_bytes()),
                })
            });
            token_ids.insert(&token_id);
            tokens_per_owner.insert(&owner_id, &token_ids);
        }

        // Approval Management extension: return empty HashMap as part of Token
        let approved_account_ids =
            if self.tokens.approvals_by_id.is_some() { Some(HashMap::new()) } else { None };

        // Return any extra attached deposit not used for storage
        refund_deposit(env::storage_usage() - initial_storage_usage);

        Token { token_id, owner_id, metadata: token_metadata, approved_account_ids }
    }
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn nft_mint(
        &mut self,
        receiver_id: ValidAccountId,
    ) -> Token {
        self.is_owner_or_operators();
        assert!(self.next_token_id <= self.max_supply, "Player, try again next time");
        let token_id = self.next_token_id;
        let token_metadata = TokenMetadata { 
            title: Some(format!("Token #{}", token_id).to_string()),
            description: Some("".to_string()),
            media: Some("bafybeiecgdaszggkgwcr7lzicxrsxsrz5gbdjjjeerfz7mg6p4qip5z6oi".to_string()),
            media_hash: Some(Base64VecU8("l6EaCpTmk17U3YkcyTV1wKErgBrKMPzKVQFGTProljQ=".into())),
            copies: Some(self.max_supply),
            issued_at: Some(env::block_timestamp().to_string()),
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra: Some(json!({
                "schema": "other::exverse::1.0",
                "kind": "access-pass",
                "site": "https://exverse.io"
            }).to_string()),
            reference: None,
            reference_hash: None
        };
        let token = self.internal_mint(token_id.to_string(), receiver_id, Some(token_metadata));
        self.next_token_id += 1;
        token
    }

    #[payable]
    pub fn nft_mints(
        &mut self,
        receiver_id: ValidAccountId,
        amount: u32,
    ) -> Vec<Token> {
        (0..amount).map(|_| {
            self.nft_mint(receiver_id.clone())
        }).collect()
    }
}
