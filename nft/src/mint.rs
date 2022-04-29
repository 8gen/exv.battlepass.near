use crate::*;
use near_sdk::{serde_json::json, near_bindgen, json_types::Base64VecU8};



#[near_bindgen]
impl Contract {
    #[payable]
    pub fn nft_mint(
        &mut self,
        receiver_id: AccountId,
    ) -> Token {
        self.is_owner_or_operators();
        assert!(self.next_token_id <= self.max_supply, "Player, try again next time");
        let token_id = self.next_token_id;
        let token_metadata = TokenMetadata { 
            title: Some(format!("Token #{}", token_id).to_string()),
            description: Some("".to_string()),
            media: Some("bafybeiecgdaszggkgwcr7lzicxrsxsrz5gbdjjjeerfz7mg6p4qip5z6oi".to_string()),
            media_hash: Some(Base64VecU8("l6EaCpTmk17U3YkcyTV1wKErgBrKMPzKVQFGTProljQ=".into())),
            copies: Some(1),
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
        let token = self.tokens.internal_mint(token_id.to_string(), receiver_id, Some(token_metadata));
        self.next_token_id += 1;
        token
    }

    #[payable]
    pub fn nft_mints(
        &mut self,
        receiver_id: AccountId,
        amount: u32,
    ) -> Vec<Token> {
        (0..amount).map(|_| {
            self.nft_mint(receiver_id.clone())
        }).collect()
    }
}
