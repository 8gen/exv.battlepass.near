use crate::*;
use near_sdk::{assert_one_yocto, near_bindgen};

impl Contract {
    pub fn assert_owner(&self) {
        assert!(env::predecessor_account_id() == self.tokens.owner_id, "ERR_NOT_OWNER");
    }

    pub fn is_owner_or_operators(&self) -> bool {
        env::predecessor_account_id() == self.tokens.owner_id
            || self.operators.contains(&env::predecessor_account_id())
    }

    pub fn assert_owner_or_operator(&self) {
        assert!(self.is_owner_or_operators(), "ERR_NOT_OWNER_OR_OPERATOR");
    }
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn set_owner(&mut self, new_owner_id: AccountId) {
        assert_one_yocto();
        self.assert_owner();
        self.tokens.owner_id = new_owner_id;
    }

    /// Extend operators. Only can be called by owner.
    #[payable]
    pub fn extend_operators(&mut self, operators: Vec<AccountId>) {
        assert_one_yocto();
        self.assert_owner();
        for operator in operators {
            self.operators.insert(&operator);
        }
    }

    /// Remove operators. Only can be called by owner.
    #[payable]
    pub fn remove_operators(&mut self, operators: Vec<AccountId>) {
        assert_one_yocto();
        self.assert_owner();
        for operator in operators {
            self.operators.remove(&operator);
        }
    }
}
