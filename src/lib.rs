use borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::collections::Map;
use near_sdk::json_types::{U128};
use near_sdk::{env, near_bindgen, ext_contract, AccountId, Balance};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Prepaid gas for making a single simple call.
const SINGLE_CALL_GAS: u64 = 1_000_000_000_000_000_000;


#[ext_contract(chiron)]
pub trait ChironCrossContract {
  fn create_account(&mut self);
  fn transfer(&mut self, recipient: AccountId, amount: U128);
  fn get_total_supply(&self) -> U128;
  fn get_balance(&self, owner_id: AccountId) -> U128;
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Account {
  username: String,
  account_id: AccountId,
  pk: Vec<u8> // public key used to access this account
}

impl Account {
  fn new(username: String) -> Self {
    Account {
      username: username,
      account_id: env::signer_account_id(),
      pk: env::signer_account_pk()
    }
  }

  fn authorize(&self) -> bool {
    return env::signer_account_pk() == self.pk
  }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct PeleonContractWallet {
  mentorship_contract: AccountId,
  chiron_token_contract: AccountId,
  accounts: Map<Vec<u8>, Account>
}

#[near_bindgen]
impl PeleonContractWallet {
  pub fn create_account(&mut self, username: String) {
    let new_account = Account::new(username);
    let account_hash = Self::get_account_hash(new_account.account_id.to_owned(), &mut new_account.pk.to_owned());
    assert!(self.accounts.get(&account_hash).is_none(), "account already exists");
    self.set_account(account_hash, new_account);
    chiron::create_account(&self.chiron_token_contract, 0, SINGLE_CALL_GAS);
  }
}

impl PeleonContractWallet {
  fn get_account_hash(account_id: AccountId, pk: &mut Vec<u8>) -> Vec<u8> {
    let mut account_bytes = account_id.as_bytes().to_vec();
    account_bytes.append(pk);
    env::sha256(account_bytes.as_slice())
  }

  fn set_account(&mut self, account_hash: Vec<u8>, account: Account) {
    self.accounts.insert(&account_hash, &account);
  }

  // panics if account DNE
  fn get_account(&self, account_hash: Vec<u8>) -> Account {
    match self.accounts.get(&account_hash) {
      Some(account) => account,
      None => panic!("account does not exist")
    }
  }

  fn check_account_authorized(&self) -> bool {
    let account_hash = Self::get_account_hash(env::signer_account_id(), &mut env::signer_account_pk());
    let account = self.get_account(account_hash);
    account.authorize()
  }

  // ----- CHIRON CALLS ----- //
  
  fn transfer_chiron(&self, recipient: AccountId, amount: U128) {
    assert!(self.check_account_authorized(), "incorrect wallet");
    chiron::transfer(recipient, amount, &self.chiron_token_contract, 0, SINGLE_CALL_GAS);
  }

  fn get_total_chiron_supply(&self, owner_id: AccountId) -> U128 {
    chiron::get_total_supply()
  }
  fn get_chiron_balance(&self, owner_id: AccountId) -> U128 {}
}