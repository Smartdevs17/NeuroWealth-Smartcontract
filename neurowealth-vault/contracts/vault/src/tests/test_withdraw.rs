//! Tests for withdrawal functionality

use super::test_utils::*;
use soroban_sdk::{testutils::{Address as _, Events}, Address, Env, Vec};

#[test]
fn test_full_withdrawal_burns_all_shares() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let token_client = soroban_sdk::contractclient!(TestToken).new(&env, &usdc_token);

    let user = Address::generate(&env);
    let deposit_amount = 10_000_000_i128;

    mint_and_deposit(&env, &client, &usdc_token, &user, amount);

    let shares_before = client.get_shares(&user);
    assert!(shares_before > 0);

    let balance = client.get_balance(&user);
    client.withdraw(&user, &balance);

    assert_eq!(client.get_shares(&user), 0, "All shares should be burned");
    assert_eq!(client.get_balance(&user), 0, "Balance should be zero");
}

#[test]
fn test_partial_withdrawal_returns_correct_proportional_assets() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let token_client = soroban_sdk::contractclient!(TestToken).new(&env, &usdc_token);

    let user = Address::generate(&env);
    let deposit_amount = 10_000_000_i128;

    mint_and_deposit(&env, &client, &usdc_token, &user, amount);

    let initial_shares = client.get_shares(&user);
    let withdraw_amount = 3_000_000_i128;

    client.withdraw(&user, &withdraw_amount);

    let remaining_shares = client.get_shares(&user);
    let remaining_balance = client.get_balance(&user);

    assert!(remaining_shares < initial_shares, "Shares should decrease");
    assert_eq!(remaining_balance, deposit_amount - withdraw_amount);
}

#[test]
#[should_panic]
fn test_withdraw_more_than_balance_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let token_client = soroban_sdk::contractclient!(TestToken).new(&env, &usdc_token);

    let user = Address::generate(&env);
    let deposit_amount = 5_000_000_i128;

    mint_and_deposit(&env, &client, &usdc_token, &user, amount);

    let excessive_amount = deposit_amount + 1_000_000_i128;
    client.withdraw(&user, &excessive_amount);
}

#[test]
#[should_panic(expected = "Amount must be positive")]
fn test_withdraw_zero_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let token_client = soroban_sdk::contractclient!(TestToken).new(&env, &usdc_token);

    let user = Address::generate(&env);
    let deposit_amount = 5_000_000_i128;

    mint_and_deposit(&env, &client, &usdc_token, &user, amount);

    client.withdraw(&user, &0);
}

#[test]
#[should_panic(expected = "Vault is paused")]
fn test_withdraw_while_paused_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let token_client = soroban_sdk::contractclient!(TestToken).new(&env, &usdc_token);

    let user = Address::generate(&env);
    let deposit_amount = 5_000_000_i128;

    mint_and_deposit(&env, &client, &usdc_token, &user, amount);

    // Pause the vault
    client.pause(&owner);
    assert!(client.is_paused());

    let balance = client.get_balance(&user);
    client.withdraw(&user, &balance);
}

#[test]
#[should_panic]
fn test_withdraw_with_no_balance_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    assert_eq!(client.get_balance(&user), 0);

    client.withdraw(&user, &1_000_000_i128);
}

#[test]
fn test_withdraw_all_returns_correct_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let token_client = soroban_sdk::contractclient!(TestToken).new(&env, &usdc_token);

    let user = Address::generate(&env);
    let deposit_amount = 10_000_000_i128;

    mint_and_deposit(&env, &client, &usdc_token, &user, amount);

    let expected_balance = client.get_balance(&user);
    let withdrawn = client.withdraw_all(&user);

    assert_eq!(withdrawn, expected_balance);
    assert_eq!(client.get_shares(&user), 0);
    assert_eq!(client.get_balance(&user), 0);
}

#[test]
fn test_withdraw_emits_event() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let token_client = soroban_sdk::contractclient!(TestToken).new(&env, &usdc_token);

    let user = Address::generate(&env);
    let deposit_amount = 10_000_000_i128;

    mint_and_deposit(&env, &client, &usdc_token, &user, amount);

    let withdraw_amount = 3_000_000_i128;
    client.withdraw(&user, &withdraw_amount);

    let events = env.events().all();
    let all: Vec<_> = events.into_iter().collect();
    let withdraw_events = find_events_by_topic(&all, &env, symbol_short!("withdraw"));
    assert!(!withdraw_events.is_empty(), "Withdraw should emit an event");
}
