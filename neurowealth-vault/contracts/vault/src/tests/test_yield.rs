//! Tests for yield functionality

use super::test_utils::*;
use soroban_sdk::{testutils::{Address as _, Events}, Address, Env, Vec};

#[test]
fn test_agent_can_update_total_assets() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let token_client = soroban_sdk::contractclient!(TestToken).new(&env, &usdc_token);

    let user = Address::generate(&env);
    let deposit_amount = 10_000_000_i128;

    mint_and_deposit(&env, &client, &usdc_token, &user, amount);

    // Agent should be able to update total assets
    let new_total = 15_000_000_i128; // Simulated yield
    client.update_total_assets(&agent, &new_total);

    assert_eq!(client.get_total_assets(), new_total);
}

#[test]
#[should_panic(expected = "Not authorized: caller is not the agent")]
fn test_non_agent_cannot_update_total_assets() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let token_client = soroban_sdk::contractclient!(TestToken).new(&env, &usdc_token);

    let user = Address::generate(&env);
    let deposit_amount = 10_000_000_i128;

    mint_and_deposit(&env, &client, &usdc_token, &user, amount);

    let non_agent = Address::generate(&env);
    let new_total = 15_000_000_i128;

    client.update_total_assets(&non_agent, &new_total);
}

#[test]
fn test_yield_increases_user_asset_balance() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let token_client = soroban_sdk::contractclient!(TestToken).new(&env, &usdc_token);

    let user = Address::generate(&env);
    let deposit_amount = 10_000_000_i128;

    mint_and_deposit(&env, &client, &usdc_token, &user, amount);

    let balance_before_yield = client.get_balance(&user);
    assert_eq!(balance_before_yield, deposit_amount);

    // Simulate yield: increase total assets by 50%
    let yield_amount = deposit_amount / 2;
    let new_total_assets = deposit_amount + yield_amount;
    client.update_total_assets(&agent, &new_total_assets);

    // Mint yield tokens to vault so withdrawal can succeed
    token_client.mint(&contract_id, &yield_amount);

    let balance_after_yield = client.get_balance(&user);
    assert!(balance_after_yield > balance_before_yield, "Balance should increase with yield");
    assert_eq!(balance_after_yield, new_total_assets, "User should get proportional share");
}

#[test]
fn test_yield_distributed_proportionally_between_users() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let token_client = soroban_sdk::contractclient!(TestToken).new(&env, &usdc_token);

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let deposit1 = 10_000_000_i128;
    let deposit2 = 5_000_000_i128;

    mint_and_deposit(&env, &client, &token_client, &user1, deposit1);
    mint_and_deposit(&env, &client, &token_client, &user2, deposit2);

    let total_deposits = deposit1 + deposit2;
    let balance1_before = client.get_balance(&user1);
    let balance2_before = client.get_balance(&user2);

    // Simulate 50% yield
    let yield_amount = total_deposits / 2;
    let new_total_assets = total_deposits + yield_amount;
    client.update_total_assets(&agent, &new_total_assets);

    // Mint yield tokens to vault
    token_client.mint(&contract_id, &yield_amount);

    let balance1_after = client.get_balance(&user1);
    let balance2_after = client.get_balance(&user2);

    // User1 should get 2/3 of the yield (deposited 2x more)
    // User2 should get 1/3 of the yield
    let expected_balance1 = deposit1 + (yield_amount * 2) / 3;
    let expected_balance2 = deposit2 + yield_amount / 3;

    // Allow small rounding differences
    assert!(
        (balance1_after - expected_balance1).abs() <= 1,
        "User1 should get proportional yield"
    );
    assert!(
        (balance2_after - expected_balance2).abs() <= 1,
        "User2 should get proportional yield"
    );

    // Verify total is correct
    assert_eq!(
        balance1_after + balance2_after,
        new_total_assets,
        "Total balances should equal total assets"
    );
}

#[test]
fn test_yield_emits_event() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let token_client = soroban_sdk::contractclient!(TestToken).new(&env, &usdc_token);

    let user = Address::generate(&env);
    let deposit_amount = 10_000_000_i128;

    mint_and_deposit(&env, &client, &usdc_token, &user, amount);

    let new_total = 15_000_000_i128;
    client.update_total_assets(&agent, &new_total);

    let events = env.events().all();
    let all: Vec<_> = events.into_iter().collect();
    let assets_events = find_events_by_topic(&all, &env, symbol_short!("assets"));
    assert!(!assets_events.is_empty(), "Assets update should emit an event");
}
