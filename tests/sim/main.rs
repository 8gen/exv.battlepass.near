use core::panic;
use std::convert::TryInto;

use libsecp256k1 as secp256k1;
use near_contract_standards::non_fungible_token::Token;
use near_sdk::json_types::U128;
use near_sdk_sim::{call, deploy, init_simulator, to_yocto, view, ContractAccount, UserAccount};
use sha3::{Digest, Keccak256};

use halloffame::{
    Config, ContractContract as HallContract, GAS_FOR_NFT_MINT_CALL, GAS_FOR_RESOLVE_TRANSFER,
    GAS_FOR_SACRIFICE,
};
use nft::ContractContract as NftContract;

mod test_private;

// Load in contract bytes at runtime
near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    NFT_WASM_BYTES => "res/nft.wasm",
    HALLOFFAME_WASM_BYTES => "res/hall.wasm",
}

const NFT_ID: &str = "nft";
const HALL_ID: &str = "halloffame";

enum MomentInTime {
    BeforePrivate,
    InPrivate,
    AfterPrivate,
}

struct Runner {
    hall: ContractAccount<HallContract>,
    nft: ContractAccount<NftContract>,
    root: UserAccount,
    alice: UserAccount,
    bob: UserAccount,
    keypair: secp256k1::SecretKey,
    eva: UserAccount,
}

impl Runner {
    pub fn default() -> Runner {
        Runner::new(2000)
    }

    pub fn new(max_supply: u64) -> Runner {
        let root = init_simulator(None);
        let nft = deploy!(
            contract: NftContract,
            contract_id: NFT_ID,
            bytes: &NFT_WASM_BYTES,
            signer_account: root,
            init_method: new_default_meta(max_supply, "NAME".to_string(), "SYMBOL".to_string())
        );
        let alice = root.create_user("alice".parse().unwrap(), to_yocto("100"));
        let bob = root.create_user("bob".parse().unwrap(), to_yocto("100"));
        let eva = root.create_user("eva".parse().unwrap(), to_yocto("100"));

        let hall = deploy!(
            contract: HallContract,
            contract_id: HALL_ID,
            bytes: &HALLOFFAME_WASM_BYTES,
            signer_account: root,
            init_method: new(nft.account_id())
        );
        let sk = secp256k1::SecretKey::default();
        let pk = secp256k1::PublicKey::from_secret_key(&sk);

        call!(
            root,
            hall.sudo_config(
                None,
                None,
                None,
                None,
                None,
                Some(hex::encode(pk.serialize_compressed()))
            )
        )
        .assert_success();
        Self { root, nft, hall, keypair: sk, alice, bob, eva }
    }

    fn hash(&self, message: String) -> [u8; 32] {
        let mut hasher = Keccak256::new();
        hasher.update(message);
        let result = hasher.finalize();
        result.into()
    }

    pub fn sign(&self, account: &UserAccount, amount: u32) -> String {
        let message = format!("{}:{}", &account.account_id, amount);
        let hash = self.hash(message);
        let msg = secp256k1::Message::parse(&hash.try_into().unwrap());
        let (sig, _) = secp256k1::sign(&msg, &self.keypair);
        hex::encode(sig.serialize())
    }

    pub fn nft_total_supply(&self) -> u128 {
        let nft = &self.nft;
        let total_supply: U128 = view!(nft.nft_total_supply()).unwrap_json();
        total_supply.0
    }

    pub fn time_travel_to(&mut self, to: MomentInTime) -> &mut Runner {
        let hall = &self.hall;
        let now: u32 =
            (self.root.borrow_runtime().current_block().block_timestamp / 1_000_000_000) as u32;
        match to {
            to @ MomentInTime::BeforePrivate => {
                call!(
                    self.root,
                    hall.sudo_config(None, None, None, Some(now + 100), Some(now + 110), None)
                );
                let config: Config = view!(hall.config()).unwrap_json();
                assert_eq!(config.stage, "SOON");
            }
            to @ MomentInTime::AfterPrivate => {
                call!(
                    self.root,
                    hall.sudo_config(None, None, None, Some(now - 10), Some(now - 5), None)
                );
                let config: Config = view!(hall.config()).unwrap_json();
                assert_eq!(config.stage, "OPEN");
            }
            to @ MomentInTime::InPrivate => {
                call!(
                    self.root,
                    hall.sudo_config(None, None, None, Some(now - 10), Some(now + 100), None)
                );
                let config: Config = view!(hall.config()).unwrap_json();
                assert_eq!(config.stage, "PRIVATE");
            }
        }
        self
    }

    pub fn assert_spend_about(&self, account: &UserAccount, amount: u128) {
        let diff = to_yocto("100") - account.account().unwrap().amount;
        let possible_diff = to_yocto("0.5");
        let is_ok = match amount {
            0 => (amount + possible_diff) > diff,
            _ => (amount + possible_diff) > diff && diff > (amount - possible_diff),
        };

        assert!(is_ok, "100 NEAR - {} = {}", amount, diff);
    }

    pub fn take_out(&self, amount: u32) {
        let nft = &self.nft;
        call!(self.root, nft.nft_mints(self.root.account_id(), amount), deposit = to_yocto("1"))
            .assert_success();
    }

    pub fn change_price(&self, price: u128) {
        let hall = &self.hall;
        call!(self.root, hall.sudo_config(None, None, Some(U128::from(price)), None, None, None));
        let config: Config = view!(hall.config()).unwrap_json();
        assert_eq!(config.price_in_yocto.0, price);
    }

    pub fn personal_sacrifice(&self, price: u128, amount: u32) -> bool {
        let signature = self.sign(&self.alice, 2);
        self.internal_sacrifice(price, amount, Some(2), Some(signature))
    }

    pub fn personal_sacrifice_force(&self, price: u128, amount: u32) -> bool {
        let signature = self.sign(&self.alice, amount);
        self.internal_sacrifice(price, amount, Some(amount), Some(signature))
    }

    pub fn personal_sacrifice_signed(
        &self,
        price: u128,
        amount: u32,
        permitted_amount: u32,
        sign: String,
    ) -> bool {
        self.internal_sacrifice(price, amount, Some(permitted_amount), Some(sign))
    }

    fn internal_sacrifice(
        &self,
        deposit: u128,
        amount: u32,
        permitted_amount: Option<u32>,
        sign: Option<String>,
    ) -> bool {
        let hall = &self.hall;
        let tx = call!(
            self.alice,
            hall.sacrifice(amount, permitted_amount, sign.clone()),
            deposit,
            GAS_FOR_RESOLVE_TRANSFER.0
                + GAS_FOR_SACRIFICE.0
                + GAS_FOR_NFT_MINT_CALL.0 * amount as u64
        );
        println!("TX: {:?}", tx);
        println!("Promise: {:?}", tx.promise_results());
        println!("Sig: {:?}", sign.clone());
        match tx.is_ok() {
            true => {
                let tokens: Vec<Token> = tx.unwrap_json();
                tokens.len() as u32 == amount
            }
            false => false,
        }
    }

    pub fn sacrifice(&self, deposit: u128, amount: u32) -> bool {
        self.internal_sacrifice(deposit, amount, None, None)
    }
}
