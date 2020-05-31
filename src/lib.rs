use borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::collections::Map;
use near_sdk::{env, near_bindgen, AccountId, Promise};
// use std::process::{abort};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Prepaid gas for making a single simple call.
// const SINGLE_CALL_GAS: u64 = 1_000_000_000_000_000_000;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Account {
  username: String,
  account_id: AccountId,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct ContractWallet {
  owner_id: AccountId,
  owner_pk: Vec<u8>,
  manager_id: AccountId,
  managet_pk: Vec<u8>,
  gateway_contract: AccountId,
  accounts: Map<Vec<u8>, Account>
}

impl Default for ContractWallet {
  fn default() -> Self {
    panic!("Contract wallet must be initialized before use!")
    // abort()
  }
}


#[near_bindgen]
impl ContractWallet {
  #[init]
  pub fn new(manager_id: AccountId, manager_pk: Vec<u8>, gateway_contract: AccountId) -> Self {
    // assert!(env::is_valid_account_id(manager_id.as_bytes()), "Manager's account ID is invalid!");
    ContractWallet {
      owner_id: env::signer_account_id(),
      owner_pk: env::signer_account_pk(),
      manager_id: manager_id,
      managet_pk: manager_pk,
      gateway_contract: gateway_contract.into(),
      accounts: Map::new("a-".as_bytes().to_vec()),
    }
  }

  fn only_owner(&self) {
    if env::signer_account_pk() != self.owner_pk {
      panic!("Only contract owner can perform this action!")
    }
  }

  fn only_manager(&self) {
    if env::signer_account_pk() != self.managet_pk && env::signer_account_pk() != self.owner_pk {
      panic!("Only contract manager or owner can perform this action!")
    }
  }

  pub fn transfer_ownership(&mut self, new_owner_id: AccountId, new_owner_pk: Vec<u8>) {
    self.only_owner();
    assert!(env::is_valid_account_id(new_owner_id.as_bytes()), "New owner's account ID is invalid!");
    self.owner_id = new_owner_id;
    self.owner_pk = new_owner_pk;
  }

  pub fn update_manager(&mut self, new_manager_id: AccountId, new_manager_pk: Vec<u8>) {
    self.only_owner();
    assert!(env::is_valid_account_id(new_manager_id.as_bytes()), "New manager's account ID is invalid!");
    self.manager_id = new_manager_id;
    self.managet_pk = new_manager_pk;
  }

  pub fn get_account_hash(username: &String, account_id: &String) -> Vec<u8> {
    let account_bytes = [username.as_bytes(), account_id.as_bytes()].concat();
    env::keccak256(&account_bytes)
  }

  fn get_account(&self, account_hash: Vec<u8>) -> Option<Account> {
    self.accounts.get(&account_hash)
  }

  fn set_account(&mut self, account_hash: &Vec<u8>, account: &Account) -> Option<Account> {
    self.accounts.insert(account_hash, account)
  }

  pub fn create_account(&mut self, username: String) -> AccountId {
    self.only_manager();
    let mut account_id = username.clone();
    account_id.push_str(&env::current_account_id());
    let account_hash = Self::get_account_hash(&username, &account_id);
    // panic if account id already taken
    if self.get_account(account_hash).is_some() {
      panic!("account with given username already exists!")
    } else {
      // create the account
      Promise::new(account_id.clone())
        .create_account();
      let hash = Self::get_account_hash(&username, &account_id);
      let new_account = Account {
        username: username,
        account_id: account_id.to_owned()
      };
      self.set_account(&hash, &new_account);
      account_id
    }
  }
}