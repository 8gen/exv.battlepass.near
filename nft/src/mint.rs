use crate::*;
use near_sdk::{near_bindgen, serde_json::json};

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn nft_mint(&mut self, receiver_id: AccountId) -> Token {
        self.assert_owner_or_operator();
        assert!(self.next_token_id - 1 <= self.max_supply, "Player, try again next time");
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
                format!("QmcjcieB2WvqEQiviJUsfdQ8FqMJT78kobbJgnxE2iK3DG/{}", token_id).to_string(),
            ),
            reference_hash: None,
        };
        let token =
            self.tokens.internal_mint(token_id.to_string(), receiver_id, Some(token_metadata));
        self.next_token_id += 1;
        token
    }
}
