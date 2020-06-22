#![deny(warnings)]

use borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::collections::UnorderedSet;
use serde::*;

use near_sdk::{env, near_bindgen, AccountId};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// This trait provides the baseline of functions as described at:
/// https://github.com/nearprotocol/NEPs/blob/nep-4/specs/Standards/Tokens/NonFungibleToken.md
pub trait NFT {
    // Grant the access to the given `accountId` for the given `tokenId`.
    // Requirements:
    // * The caller of the function (`predecessor_id`) should have access to the token.
    fn grant_access(&mut self, escrow_account_id: AccountId);

    // Revoke the access to the given `accountId` for the given `tokenId`.
    // Requirements:
    // * The caller of the function (`predecessor_id`) should have access to the token.
    fn revoke_access(&mut self, escrow_account_id: AccountId);

    // Transfer the given `tokenId` to the given `accountId`. Account `accountId` becomes the new owner.
    // Requirements:
    // * The caller of the function (`predecessor_id`) should have access to the token.
    fn transfer_from(&mut self, owner_id: AccountId, new_owner_id: AccountId, token_id: TokenId);

    // Transfer the given `tokenId` to the given `accountId`. Account `accountId` becomes the new owner.
    // Requirements:
    // * The caller of the function (`predecessor_id`) should be the owner of the token. Callers who have
    // escrow access should use transfer_from.
    fn transfer(&mut self, new_owner_id: AccountId, token_id: TokenId);

    // Returns `true` or `false` based on caller of the function (`predecessor_id) having access to the token
    fn check_access(&self, account_id: AccountId) -> bool;

    // Get an individual owner by given `tokenId`.
    fn get_token_owner(&self, token_id: TokenId) -> String;
}

pub type StampId = u64;

/// The token ID type is also defined in the NEP
pub type TokenId = String;
pub type AccountIdHash = Vec<u8>;

#[derive(BorshDeserialize, BorshSerialize,Serialize)]
pub struct StampEntry {
    pub image_src: String,
    pub stamp_desc: String,
    pub token_id: TokenId,
    pub price: u64,
}


#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct StampToken {
    pub account_gives_access: UnorderedMap<AccountIdHash, UnorderedSet<AccountIdHash>>,

    pub token_to_account: UnorderedMap<TokenId, AccountId>,
    /// TokenId -> Stamp details.
    pub stamps: UnorderedMap<TokenId, StampEntry>,
    // Vec<u8> is sha256 of account, makes it safer and is how fungible token also works
    pub owner_id: AccountId,
    pub base_uri: String,
    pub count_id: u32,//stamp id total
    pub count_id_token: UnorderedMap<i32, TokenId>,//stamp id to token_id
}

impl Default for StampToken {
    fn default() -> Self { panic!("StampToken should be initialized before usage") }
}

#[near_bindgen]
impl StampToken  {
    #[init]
    pub fn new(owner_id: AccountId, base_uri: String) -> Self {
        assert!(env::is_valid_account_id(owner_id.as_bytes()), "Owner's account ID is invalid.");
        assert!(!env::state_exists(), "Already initialized");
        Self {
            token_to_account:  UnorderedMap::new(b"token-belongs-to".to_vec()),
            account_gives_access: UnorderedMap::new(b"gives-access".to_vec()),
            owner_id,
            stamps: UnorderedMap::new(b"stamps".to_vec()),
            base_uri: base_uri,
            count_id: 0,
            count_id_token: UnorderedMap::new(b"".to_vec()),
        }
    }
}

#[near_bindgen]
impl NFT for StampToken {
    fn grant_access(&mut self, escrow_account_id: AccountId) {
        let escrow_hash = env::sha256(escrow_account_id.as_bytes());
        let predecessor = env::predecessor_account_id();
        let predecessor_hash = env::sha256(predecessor.as_bytes());

        let mut access_set = match self.account_gives_access.get(&predecessor_hash) {
            Some(existing_set) => {
                existing_set
            },
            None => {
                UnorderedSet::new(b"new-access-set".to_vec())
            }
        };
        access_set.insert(&escrow_hash);
        self.account_gives_access.insert(&predecessor_hash, &access_set);
    }

    fn revoke_access(&mut self, escrow_account_id: AccountId) {
        let predecessor = env::predecessor_account_id();
        let predecessor_hash = env::sha256(predecessor.as_bytes());
        let mut existing_set = match self.account_gives_access.get(&predecessor_hash) {
            Some(existing_set) => existing_set,
            None => env::panic(b"Access does not exist.")
        };
        let escrow_hash = env::sha256(escrow_account_id.as_bytes());
        if existing_set.contains(&escrow_hash) {
            existing_set.remove(&escrow_hash);
            self.account_gives_access.insert(&predecessor_hash, &existing_set);
            env::log(b"Successfully removed access.")
        } else {
            env::panic(b"Did not find access for escrow ID.")
        }
    }

    fn transfer_from(&mut self, owner_id: AccountId, new_owner_id: AccountId, token_id: TokenId) {
        let token_owner_account_id = self.get_token_owner(token_id.clone());
        if owner_id != token_owner_account_id {
            env::panic(b"Attempt to transfer a token from a different owner.")
        }

        if !self.check_access(token_owner_account_id) {
            env::panic(b"Attempt to transfer a token with no access.")
        }
        self.token_to_account.insert(&token_id, &new_owner_id);
    }

    fn transfer(&mut self, new_owner_id: AccountId, token_id: TokenId) {
        let token_owner_account_id = self.get_token_owner(token_id.clone());
        let predecessor = env::predecessor_account_id();
        if predecessor != token_owner_account_id {
            env::panic(b"Attempt to call transfer on tokens belonging to another account.")
        }
        self.token_to_account.insert(&token_id, &new_owner_id);
    }

    fn check_access(&self, account_id: AccountId) -> bool {
        let account_hash = env::sha256(account_id.as_bytes());
        let predecessor = env::predecessor_account_id();
        if predecessor == account_id {
            return true;
        }
        match self.account_gives_access.get(&account_hash) {
            Some(access) => {
                let predecessor = env::predecessor_account_id();
                let predecessor_hash = env::sha256(predecessor.as_bytes());
                access.contains(&predecessor_hash)
            }
            None => false
        }
    }

    fn get_token_owner(&self, token_id: TokenId) -> AccountId {
        match self.token_to_account.get(&token_id) {
            Some(owner_id) => owner_id,
            None => env::panic(b"No owner of the token ID specified")
        }
    }
}

#[near_bindgen]
impl StampToken {
    /// Creates a token for owner_id, doesn't use autoincrement, fails if id is taken
    pub fn mint_token(&mut self, owner_id: String, token_id: TokenId, price: u64, stamp_desc: String, image_src: String) {
        // make sure that only the owner can call this funtion
        self.only_owner();
        // Since Map doesn't have `contains` we use match
        let token_check = self.token_to_account.get(&token_id);
        if token_check.is_some() {
            env::panic(b"Token ID already exists.")
        }
        let stamp_entry = StampEntry {
            price: price,
            stamp_desc: stamp_desc,
            image_src: image_src,
            token_id: token_id.clone(),
        };
        self.token_to_account.insert(&token_id, &owner_id);
        // No token with that ID exists, mint and add token to data structures
        self.stamps.insert(&token_id, &stamp_entry);
        self.count_id += 1;
        self.count_id_token.insert(&(self.count_id as i32), &token_id);
       
        
    }
    pub fn stamp_info(&self, token_id: TokenId) -> Option<StampEntry> {
        // Since Map doesn't have `contains` we use match
        let token_info_check = self.stamps.get(&token_id);
        if !token_info_check.is_some() {
            env::panic(b"Token ID not exists.")
        }
        token_info_check
    }

    pub fn total(&self) -> Option<i32> {
        Some(self.count_id as i32)
    }

    pub fn count_id_token(&self, id: i32) -> Option<TokenId> {
        let token_id = self.count_id_token.get(&id);
        if !token_id.is_some() {
            env::panic(b"Token ID not exists.")
        }
        token_id
    }

    /// helper function determining contract ownership
    fn only_owner(&mut self) {
        assert_eq!(env::predecessor_account_id(), self.owner_id, "Only contract owner can call this method.");
    }

}

// use the attribute below for unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    fn joe() -> AccountId {
        "joe.testnet".to_string()
    }

    fn robert() -> AccountId {
        "robert.testnet".to_string()
    }

    fn mike() -> AccountId {
        "mike.testnet".to_string()
    }

    // part of writing unit tests is setting up a mock context
    // this is a useful list to peek at when wondering what's available in env::*
    fn get_context(predecessor_account_id: String, storage_usage: u64) -> VMContext {
        VMContext {
            current_account_id: "alice.testnet".to_string(),
            signer_account_id: "jane.testnet".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id,
            input: vec![],
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view: false,
            output_data_receivers: vec![],
            epoch_height: 19,
        }
    }

    #[test]
    fn grant_access() {
        let context = get_context(robert(), 0);
        testing_env!(context);
        let mut contract = StampToken::new(robert(), String::from("http://localhost:8080"));
        let length_before = contract.account_gives_access.len();
        assert_eq!(0, length_before, "Expected empty account access Map.");
        contract.grant_access(mike());
        contract.grant_access(joe());
        let length_after = contract.account_gives_access.len();
        assert_eq!(1, length_after, "Expected an entry in the account's access Map.");
        let predecessor_hash = env::sha256(robert().as_bytes());
        let num_grantees = contract.account_gives_access.get(&predecessor_hash).unwrap();
        assert_eq!(2, num_grantees.len(), "Expected two accounts to have access to predecessor.");
    }

    #[test]
    #[should_panic(
    expected = r#"Access does not exist."#
    )]
    fn revoke_access_and_panic() {
        let context = get_context(robert(), 0);
        testing_env!(context);
        let mut contract = StampToken::new(robert(), String::from("http://localhost:8080"));
        contract.revoke_access(joe());
    }

    #[test]
    fn mint_token() {
        let context = get_context(robert(), 0);
        testing_env!(context);
        let mut contract = StampToken::new(robert(), String::from("http://localhost:8080"));
        let length_before = contract.token_to_account.len();
        assert_eq!(0, length_before, "Expected empty account access Map.");
        contract.mint_token(robert(), "token001".to_string(), 1000, "stamp001".to_string(), "image001".to_string());
        let length_before = contract.token_to_account.len();
        assert_eq!(1, length_before, "Expected empty account access Map.");
    }

    #[test]
    fn get_stamp_info() {
        let context = get_context(robert(), 0);
        testing_env!(context);
        let mut contract = StampToken::new(robert(), String::from("http://localhost:8080"));
        let length_before = contract.token_to_account.len();
        assert_eq!(0, length_before, "Expected empty account access Map.");
        contract.mint_token(robert(), "token001".to_string(), 1000, "stamp001".to_string(), "image001".to_string());

        let token_info = contract.stamp_info("token001".to_string()).unwrap();
        assert_eq!("token001", token_info.token_id, "Expected empty account access Map.");
    }
    #[test]
    fn get_total() {
        let context = get_context(robert(), 0);
        testing_env!(context);
        let mut contract = StampToken::new(robert(), String::from("http://localhost:8080"));
        let length_before = contract.token_to_account.len();
        assert_eq!(0, length_before, "Expected empty account access Map.");
        contract.mint_token(robert(), "token001".to_string(), 1000, "stamp001".to_string(), "image001".to_string());
        contract.mint_token(robert(), "token002".to_string(), 1000, "stamp002".to_string(), "image002".to_string());

        let total = contract.total().unwrap();
        println!("total-{:?}",total);
        assert_eq!(2, total, "Expected empty account access Map.");
    }
    #[test]
    fn get_token_id_by_id() {
        let context = get_context(robert(), 0);
        testing_env!(context);
        let mut contract = StampToken::new(robert(), String::from("http://localhost:8080"));
        let length_before = contract.token_to_account.len();
        assert_eq!(0, length_before, "Expected empty account access Map.");
        contract.mint_token(robert(), "token001".to_string(), 1000, "stamp001".to_string(), "image001".to_string());
        contract.mint_token(robert(), "token002".to_string(), 1000, "stamp002".to_string(), "image002".to_string());

        let token_id = contract.count_id_token(1).unwrap();
        println!("total-{:?}",token_id);
        let token_id2 = contract.count_id_token(2).unwrap();
        println!("total2-{:?}",token_id2);
    }
}
