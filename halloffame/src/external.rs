use crate::*;

/// external contract calls
#[ext_contract(ext_nft)]
trait NonFungibleToken {
    // change methods
    fn nft_transfer(
        &mut self,
        receiver_id: String,
        token_id: String,
        approval_id: Option<u64>,
        memo: Option<String>,
    );
    fn nft_transfer_call(
        &mut self,
        receiver_id: String,
        token_id: String,
        approval_id: Option<u64>,
        memo: Option<String>,
        msg: String,
    ) -> bool;

    // view method
    fn nft_token(&self, token_id: String) -> Option<Token>;
    fn nft_mints(&mut self, receiver_id: String, amount: u32) -> Vec<Token>;
}
