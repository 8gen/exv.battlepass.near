use crate::*;
use std::convert::TryInto;

use near_sdk::env::keccak256;
use secp256k1;

impl Contract {
    pub fn verify_signature(&self, pk: String, signature: String, payload: String) -> bool {
        let mut pk_slice: [u8; 33] = [0; 33];
        hex::decode_to_slice(pk, &mut pk_slice).expect("ERR_WRONG_PKEY:HEX");
        let public_key =
            secp256k1::PublicKey::parse_compressed(&pk_slice).expect("ERR_WRONG_PKEY:PARSE");

        let hash = keccak256(payload.as_bytes());
        let msg = secp256k1::Message::parse(&hash.try_into().unwrap());

        let sign_vec = hex::decode(signature).expect("ERR_WRONG_SIG:HEX");
        assert_eq!(sign_vec.len(), 64, "ERR_WRONG_SIG:SIZE");
        let sign = secp256k1::Signature::parse(&sign_vec.try_into().unwrap());

        secp256k1::verify(&msg, &sign, &public_key)
    }
}
