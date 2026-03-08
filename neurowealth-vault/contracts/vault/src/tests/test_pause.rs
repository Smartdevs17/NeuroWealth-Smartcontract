//! Tests for pause functionality

use super::test_utils::*;
use soroban_sdk::{testutils::{Address as _, Events}, Address, Env, Vec};

#[test]
fn test_owner_can_pause() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    assert!(!client.is_paused(), "Vault should start unpaused");

    client.pause(&owner);

    assert!(client.is_paused(), "Vault should be paused");
}

#[test]
fn test_owner_can_unpause() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    client.pause(&owner);
    assert!(client.is_paused());

    client.unpause(&owner);
    assert!(!client.is_paused(), "Vault should be unpaused");
}

#[test]
fn test_agent_can_emergency_pause() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, agent, _owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    assert!(!client.is_paused());

    // Agent can emergency pause
    client.emergency_pause(&agent);

    assert!(client.is_paused(), "Vault should be emergency paused");
}

#[test]
#[should_panic(expected = "Not authorized: caller is not the owner")]
fn test_agent_cannot_unpause() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, agent, owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    // Owner pauses
    client.pause(&owner);
    assert!(client.is_paused());

    // Agent tries to unpause - should fail
    client.unpause(&agent);
}

#[test]
#[should_panic]
fn test_unauthorized_users_cannot_pause() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let unauthorized = Address::generate(&env);

    // This should fail due to authorization
    client.pause(&unauthorized);
}

#[test]
#[should_panic(expected = "Vault is paused")]
fn test_deposit_blocked_while_paused() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let token_client = soroban_sdk::contractclient!(TestToken).new(&env, &usdc_token);

    client.pause(&owner);
    assert!(client.is_paused());

    let user = Address::generate(&env);
    let amount = 5_000_000_i128;

    token_client.mint(&user, &amount);
    client.deposit(&user, &amount);
}

#[test]
#[should_panic(expected = "Vault is paused")]
fn test_withdraw_blocked_while_paused() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let token_client = soroban_sdk::contractclient!(TestToken).new(&env, &usdc_token);

    let user = Address::generate(&env);
    let amount = 5_000_000_i128;

    mint_and_deposit(&env, &client, &usdc_token, &user, amount);

    client.pause(&owner);
    assert!(client.is_paused());

    let balance = client.get_balance(&user);
    client.withdraw(&user, &balance);
}

#[test]
#[should_panic(expected = "Vault is paused")]
fn test_rebalance_blocked_while_paused() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    client.pause(&owner);
    assert!(client.is_paused());

    client.rebalance(&symbol_short!("blend"), &500_i128);
}

#[test]
fn test_pause_emits_event() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    client.pause(&owner);

    let events = env.events().all();
    let all: Vec<_> = events.into_iter().collect();
    let pause_events = find_events_by_topic(&all, &env, symbol_short!("paused"));
    assert!(!pause_events.is_empty(), "Pause should emit an event");
}

#[test]
fn test_emergency_pause_emits_event() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, agent, _owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    client.emergency_pause(&agent);

    let events = env.events().all();
    let all: Vec<_> = events.into_iter().collect();
    let emergency_events = find_events_by_topic(&all, &env, symbol_short!("emergency"));
    assert!(!emergency_events.is_empty(), "Emergency pause should emit an event");
}
