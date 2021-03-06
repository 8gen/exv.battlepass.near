use near_contract_standards::non_fungible_token::Token;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{UnorderedMap, UnorderedSet},
    env, ext_contract,
    json_types::U128,
    near_bindgen, AccountId, Balance, BorshStorageKey, Gas, PanicOnDefault, Promise, PromiseResult,
};
use serde::{Deserialize, Serialize};

pub use crate::external::*;
pub use crate::utils::*;
mod crypto;
mod external;
mod owner;
mod utils;
mod web4;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    owner_id: AccountId,
    treasury_id: AccountId,
    operators: UnorderedSet<AccountId>,
    nft_account_id: AccountId,
    sold: UnorderedMap<AccountId, u32>,
    signer_pk: Option<String>,
    private_sale_timestamp: u64,
    open_sale_timestamp: u64,
    price_in_yocto: Balance,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Sold,
    Operator,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub nft_account_id: AccountId,
    pub signer_pk: Option<String>,
    pub owner_id: AccountId,
    pub private_sale_timestamp: u64,
    pub open_sale_timestamp: u64,
    pub curret_timestamp: u64,
    pub price_in_yocto: U128,
    pub stage: String,
    pub motivation: String,
}

#[derive(Serialize, Deserialize)]
pub struct Status {
    pub config: Config,
    pub sold: u32,
}

macro_rules! update_if_exists {
    ($self:tt, $l:tt) => {
        if let Some($l) = $l {
            $self.$l = $l;
        }
    };
    ($self:tt, $l:tt, $value: expr) => {
        if let Some($l) = $l {
            $self.$l = $value;
        }
    };
}

#[ext_contract(ext_halloffame)]
trait Contract {
    fn callback_on_nft_mints(
        &mut self,
        price: Balance,
        attached_deposit: Balance,
        desired_amount: u32,
    ) -> Vec<Token>;
}

const MINT_COST: u128 = 10_u128.pow(20) * 80;

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(nft_account_id: AccountId) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self {
            owner_id: env::predecessor_account_id(),
            treasury_id: env::predecessor_account_id(),
            operators: UnorderedSet::new(StorageKey::Operator),
            nft_account_id,
            price_in_yocto: 175 * 10_u128.pow(23),
            signer_pk: None,
            sold: UnorderedMap::new(StorageKey::Sold),
            private_sale_timestamp: 0,
            open_sale_timestamp: 0,
        }
    }

    pub fn status(self, account_id: AccountId) -> Status {
        assert!(env::state_exists(), "State is not initialized");
        let sold = match self.sold.get(&account_id) {
            None => 0,
            Some(value) => value,
        };
        Status { config: self.config(), sold }
    }

    pub fn config(self) -> Config {
        assert!(env::state_exists(), "State is not initialized");
        Config {
            signer_pk: self.signer_pk,
            owner_id: self.owner_id,
            nft_account_id: self.nft_account_id,
            price_in_yocto: self.price_in_yocto.into(),
            private_sale_timestamp: self.private_sale_timestamp / 1_000_000_000_u64,
            open_sale_timestamp: self.open_sale_timestamp / 1_000_000_000_u64,
            curret_timestamp: env::block_timestamp() / 1_000_000_000_u64,
            stage: match env::block_timestamp() {
                ts if self.private_sale_timestamp == 0 || ts < self.private_sale_timestamp => {
                    "SOON".to_string()
                }
                ts if ts < self.open_sale_timestamp => "PRIVATE".to_string(),
                _ => "OPEN".to_string(),
            },
            motivation: "The zero city is coming. <3 Human Guild!".to_string(),
        }
    }

    pub fn sudo_config(
        &mut self,
        nft_account_id: Option<AccountId>,
        treasury_id: Option<AccountId>,
        price_in_yocto: Option<U128>,
        private_sale_timestamp: Option<u32>,
        open_sale_timestamp: Option<u32>,
        signer_pk: Option<String>,
    ) {
        self.assert_owner_or_operator();
        assert!(env::state_exists(), "State is not initialized");
        update_if_exists!(self, nft_account_id);
        update_if_exists!(self, treasury_id);
        update_if_exists!(self, price_in_yocto, price_in_yocto.0);
        update_if_exists!(
            self,
            private_sale_timestamp,
            private_sale_timestamp as u64 * 1_000_000_000_u64
        );
        update_if_exists!(
            self,
            open_sale_timestamp,
            open_sale_timestamp as u64 * 1_000_000_000_u64
        );
        update_if_exists!(self, signer_pk, Some(signer_pk));
    }

    #[private]
    pub fn callback_on_nft_mints(
        &mut self,
        price: Balance,
        attached_deposit: Balance,
        desired_amount: u32,
    ) -> Vec<Token> {
        assert_eq!(env::predecessor_account_id(), env::current_account_id(), "ERR_WRONG_CALLBACK");
        assert_eq!(env::promise_results_count(), 1);
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => {
                Promise::new(env::signer_account_id())
                    .transfer(attached_deposit + MINT_COST * desired_amount as u128);
                vec![]
            }
            PromiseResult::Successful(val) => {
                let tokens: Vec<Token> =
                    near_sdk::serde_json::from_slice(&val).expect("ERR_WRONG_VAL_RECEIVED");
                let actual_amount = tokens.len() as u32;
                let refund: Balance = attached_deposit
                    + (desired_amount - actual_amount) as u128 * (MINT_COST + price)
                    - (actual_amount) as u128 * price;
                let already_sold = self.sold.get(&env::signer_account_id()).unwrap();
                self.sold.insert(&env::signer_account_id(), &(already_sold + actual_amount));

                Promise::new(self.owner_id.clone())
                    .transfer(self.price_in_yocto * actual_amount as u128);

                if refund > 0 {
                    Promise::new(env::signer_account_id()).transfer(refund);
                }
                tokens
            }
        }
    }

    #[payable]
    pub fn sacrifice(
        &mut self,
        amount: u32,
        permitted_amount: Option<u32>,
        signature: Option<String>,
    ) -> Promise {
        assert_ne!(self.open_sale_timestamp, 0, "ERR_NOT_STARTED");
        assert_ne!(self.private_sale_timestamp, 0, "ERR_NOT_STARTED");
        assert!(
            env::prepaid_gas()
                >= GAS_FOR_NFT_MINT_CALL * amount.into()
                    + GAS_FOR_RESOLVE_TRANSFER
                    + GAS_FOR_SACRIFICE,
            "ERR_NOT_ENOUGH_GAS"
        );

        let receiver_id = env::predecessor_account_id();
        let mut attached_deposit = env::attached_deposit() - MINT_COST * amount as u128;
        let already_sold = match self.sold.get(&receiver_id) {
            None => {
                let storage_usage = env::storage_usage();
                self.sold.insert(&receiver_id, &0);
                attached_deposit = attached_deposit
                    - (env::storage_usage() - storage_usage) as u128 * env::storage_byte_cost();
                0
            }
            Some(value) => value,
        };

        if self.open_sale_timestamp < env::block_timestamp() {
            // Skip check, already public
            assert!(already_sold + amount <= 2, "ERR_TOO_MUCH");
        } else if self.private_sale_timestamp < env::block_timestamp()
            && permitted_amount.is_some()
            && signature.is_some()
        {
            // Private, check signature and permitted_amount
            let permitted_amount = permitted_amount.unwrap();
            assert!(self.signer_pk.is_some(), "ERR_NOT_VALID_SIGNER");
            assert!(
                self.verify_signature(
                    self.signer_pk.clone().unwrap(),
                    signature.expect("ERR_WRONG_SIG:MISS"),
                    format!("{}:{}", env::predecessor_account_id(), permitted_amount)
                ),
                "ERR_WRONG_SIG:PAYLOAD"
            );
            assert!(already_sold + amount <= permitted_amount, "ERR_TOO_MUCH");
        } else {
            env::panic_str("ERR_NOT_STARTED");
        }

        assert!(attached_deposit >= amount as u128 * self.price_in_yocto, "ERR_NOT_ENOUGH");

        ext_nft::nft_mints(
            env::predecessor_account_id().to_string(),
            amount,
            self.nft_account_id.clone(),
            MINT_COST * amount as u128,
            env::prepaid_gas() - GAS_FOR_SACRIFICE - GAS_FOR_RESOLVE_TRANSFER,
        )
        .then(ext_halloffame::callback_on_nft_mints(
            self.price_in_yocto,
            attached_deposit,
            amount,
            env::current_account_id(),
            0,
            GAS_FOR_RESOLVE_TRANSFER,
        ))
    }
}
