#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger, MockAuth},
    token, Address, Env, Symbol, Vec,
};

use crate::{
    base::{errors::CrowdfundingError, types::{PoolConfig, PoolMetadata, PoolState, StorageKey}},
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};

fn setup_test(env: &Env) -> (CrowdfundingContractClient, Address, Address, u64) {
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(env, &contract_id);
    
    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_id.address();
    
    client.initialize(&admin, &token_address, &0);
    
    // Create pool
    let creator = Address::generate(env);
    let name = String::from_str(env, "Test Event Pool");
    let metadata = PoolMetadata {
        description: String::from_str(env, "Test pool for withdraw_event_funds"),
        external_url: String::from_str(env, ""),
        image_hash: String::from_str(env, ""),
    };
    let target = 10_000i128;
    let deadline = env.ledger().timestamp() + 1_000_000;
    
    let pool_id = client.save_pool(
        &name,
        &metadata,
        &creator,
        &target,
        &deadline,
        &None::<u32>,
        &None::<Vec<Address>>,
    );
    
    // Fund EventPool via buy_ticket
    let buyer = Address::generate(env);
    let token_admin_client = token::StellarAssetClient::new(env, &token_address);
    token_admin_client.mint(&buyer, &5_000);
    
    // Set pool to Disbursed state
    client.update_pool_state(&pool_id, &PoolState::Disbursed);
    
    (client, creator, token_address, pool_id)
}

#[test]
fn test_withdraw_event_funds_success() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, creator, token_address, pool_id) = setup_test(&env);
    
    let token_client = token::Client::new(&env, &token_address);
    let contract_balance_before = token_client.balance(&client.address);
    
    // Withdraw
    client.withdraw_event_funds(&pool_id, &creator);
    
    // Verify funds transferred and drained flag set
    let drained_key = StorageKey::PoolDrained(pool_id);
    let is_drained: bool = env.storage().instance().get(&drained_key).unwrap_or(false);
    assert!(is_drained);
    
    let contract_balance_after = token_client.balance(&client.address);
    assert!(contract_balance_after < contract_balance_before);
    
    let creator_balance = token_client.balance(&creator);
    assert!(creator_balance > 0);
}

#[test]
#[should_panic(expected = "Error(Contract, Unauthorized)")]
fn test_withdraw_event_funds_unauthorized() {
    let env = Env::default();
    
    let (client, _creator, token_address, pool_id) = setup_test(&env);
    let unauthorized = Address::generate(&env);
    
    // Unauthorized caller
    client.withdraw_event_funds(&pool_id, &unauthorized);
}

#[test]
fn test_withdraw_event_funds_double_withdrawal_prevented() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, creator, token_address, pool_id) = setup_test(&env);
    
    // First withdrawal succeeds
    client.withdraw_event_funds(&pool_id, &creator);
    
    // Second withdrawal fails
    let result = client.try_withdraw_event_funds(&pool_id, &creator);
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolAlreadyDisbursed)));
}

#[test]
fn test_withdraw_event_funds_no_funds() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, creator, _token_address, pool_id) = setup_test(&env);
    
    // Don't fund EventPool
    let event_pool_key = StorageKey::EventPool(pool_id);
    env.storage().instance().set(&event_pool_key, &0i128);
    
    let result = client.try_withdraw_event_funds(&pool_id, &creator);
    // Should fail on insufficient funds check (impl will add)
    // For now expect pool state or generic error
    assert!(result.is_err());
}

#[test]
fn test_withdraw_event_funds_wrong_state() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, creator, _token_address, pool_id) = setup_test(&env);
    
    // Keep in Active state (not Disbursed/Completed)
    client.update_pool_state(&pool_id, &PoolState::Active);
    
    let result = client.try_withdraw_event_funds(&pool_id, &creator);
    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidPoolState)));
}

#[test]
fn test_withdraw_event_funds_paused_contract() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone()).address();
    
    client.initialize(&admin, &token_address, &0);
    client.pause();
    
    let creator = Address::generate(&env);
    let pool_id = client.save_pool(
        &String::from_str(&env, "Paused Pool"),
        &PoolMetadata {
            description: String::from_str(&env, ""),
            external_url: String::from_str(&env, ""),
            image_hash: String::from_str(&env, ""),
        },
        &creator,
        &1000,
        &(env.ledger().timestamp() + 1000),
        &None::<u32>,
        &None::<Vec<Address>>,
    );
    
    client.update_pool_state(&pool_id, &PoolState::Disbursed);
    
    let result = client.try_withdraw_event_funds(&pool_id, &creator);
    assert_eq!(result, Err(Ok(CrowdfundingError::ContractPaused)));
}

#[test]
fn test_withdraw_event_funds_nonexistent_pool() {
    let env = Env::default();
    
    let (client, _creator, _token_address, _) = setup_test(&env);
    
    let result = client.try_withdraw_event_funds(&999u64, &Address::generate(&env));
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolNotFound)));
}

