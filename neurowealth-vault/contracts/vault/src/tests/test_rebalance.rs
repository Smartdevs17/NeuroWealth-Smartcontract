//! Tests for rebalance functionality

use super::utils::*;
use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env};

#[test]
fn test_agent_can_rebalance_with_custom_protocol() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, agent, _owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    // Verify agent is set correctly
    assert_eq!(client.get_agent(), agent);

    // Use a non-blend protocol so no Blend pool config is required
    let protocol = symbol_short!("balanced");
    let expected_apy = 500_i128; // 5% APY in basis points

    // Should succeed with mock_all_auths (require_is_agent passes)
    client.rebalance(&protocol, &expected_apy);
}

#[test]
fn test_rebalance_emits_event() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    // Set up Blend pool and deposit so there are assets
    let blend_pool = Address::generate(&env);
    client.set_blend_pool(&owner, &blend_pool);

    let user = Address::generate(&env);
    let deposit_amount = 10_000_000_i128;
    mint_and_deposit(&env, &client, &usdc_token, &user, deposit_amount);

    client.rebalance(&symbol_short!("blend"), &500_i128);

    let rebalance_events =
        find_events_by_topic(env.events().all(), &env, symbol_short!("rebalance"));
    assert!(
        !rebalance_events.is_empty(),
        "Rebalance should emit an event"
    );
}

#[test]
fn test_rebalance_with_none_protocol_succeeds() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    // "none" protocol just sets current protocol to "none" — always safe to call
    client.rebalance(&symbol_short!("none"), &0_i128);
}

#[test]
fn test_rebalance_with_blend_after_deposit() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    // Configure Blend pool
    let blend_pool = Address::generate(&env);
    client.set_blend_pool(&owner, &blend_pool);

    // Deposit so vault has a token balance to supply
    let user = Address::generate(&env);
    let deposit_amount = 10_000_000_i128;
    mint_and_deposit(&env, &client, &usdc_token, &user, deposit_amount);

    // Rebalance should succeed (BlendPoolClient methods are stubs)
    client.rebalance(&symbol_short!("blend"), &500_i128);
}

#[test]
fn test_rebalance_apy_parameter_accepted() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    // Various APY values should be accepted without panicking
    client.rebalance(&symbol_short!("none"), &0_i128);
    client.rebalance(&symbol_short!("none"), &850_i128);
    client.rebalance(&symbol_short!("none"), &2000_i128);
}

#[test]
#[should_panic(expected = "Vault is paused")]
fn test_rebalance_while_paused_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    // Pause the vault
    client.pause(&owner);
    assert!(client.is_paused());

    client.rebalance(&symbol_short!("none"), &500_i128);
}

#[test]
#[should_panic(expected = "Blend pool not configured")]
fn test_blend_rebalance_without_pool_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    // Deposit so vault_balance > 0, triggering the blend pool check
    let user = Address::generate(&env);
    let deposit_amount = 5_000_000_i128;
    mint_and_deposit(&env, &client, &usdc_token, &user, deposit_amount);

    // blend pool not set → should panic
    client.rebalance(&symbol_short!("blend"), &500_i128);
}
