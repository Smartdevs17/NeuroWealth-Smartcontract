//! Tests verifying that each contract operation emits the expected event

use super::test_utils::*;
use soroban_sdk::{testutils::Address as _, symbol_short, Address, Env};

#[test]
fn test_initialize_emits_init_event() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, NeuroWealthVault);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let agent = Address::generate(&env);
    let usdc_token = Address::generate(&env);
    client.initialize(&agent, &usdc_token);

    let init_events = find_events_by_topic(env.events().all(), &env, symbol_short!("init"));
    assert_eq!(init_events.len(), 1, "Exactly one init event should be emitted");
}

#[test]
fn test_deposit_emits_deposit_event() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    mint_and_deposit(&env, &client, &usdc_token, &user, 5_000_000_i128);

    let deposit_events = find_events_by_topic(env.events().all(), &env, symbol_short!("deposit"));
    assert!(!deposit_events.is_empty(), "Deposit should emit an event");
}

#[test]
fn test_withdraw_emits_withdraw_event() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let deposit_amount = 10_000_000_i128;
    mint_and_deposit(&env, &client, &usdc_token, &user, deposit_amount);

    client.withdraw(&user, &3_000_000_i128);

    let withdraw_events = find_events_by_topic(env.events().all(), &env, symbol_short!("withdraw"));
    assert!(!withdraw_events.is_empty(), "Withdraw should emit an event");
}

#[test]
fn test_pause_emits_paused_event() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    client.pause(&owner);

    let pause_events = find_events_by_topic(env.events().all(), &env, symbol_short!("paused"));
    assert!(!pause_events.is_empty(), "Pause should emit an event");
}

#[test]
fn test_unpause_emits_unpaused_event() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    client.pause(&owner);
    client.unpause(&owner);

    let unpause_events = find_events_by_topic(env.events().all(), &env, symbol_short!("unpaused"));
    assert!(!unpause_events.is_empty(), "Unpause should emit an event");
}

#[test]
fn test_emergency_pause_emits_emergency_event() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, agent, _owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    client.emergency_pause(&agent);

    let emergency_events =
        find_events_by_topic(env.events().all(), &env, symbol_short!("emerg"));
    assert!(!emergency_events.is_empty(), "Emergency pause should emit an event");
}

#[test]
fn test_set_deposit_limits_emits_limits_event() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    client.set_deposit_limits(&2_000_000_i128, &20_000_000_000_i128);

    let limits_events = find_events_by_topic(env.events().all(), &env, symbol_short!("l_upd"));
    assert!(!limits_events.is_empty(), "set_deposit_limits should emit a limits event");
}

#[test]
fn test_update_agent_emits_agent_event() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _old_agent, _owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    client.update_agent(&Address::generate(&env));

    let agent_events = find_events_by_topic(env.events().all(), &env, symbol_short!("agent"));
    assert!(!agent_events.is_empty(), "update_agent should emit an event");
}

#[test]
fn test_update_total_assets_emits_assets_event() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let token_client = TestTokenClient::new(&env, &usdc_token);

    let user = Address::generate(&env);
    let deposit_amount = 10_000_000_i128;
    mint_and_deposit(&env, &client, &usdc_token, &user, deposit_amount);

    let yield_amount = 5_000_000_i128;
    token_client.mint(&contract_id, &yield_amount);
    client.update_total_assets(&agent, &(deposit_amount + yield_amount));

    let assets_events = find_events_by_topic(env.events().all(), &env, symbol_short!("assets"));
    assert!(!assets_events.is_empty(), "update_total_assets should emit an event");
}

#[test]
fn test_rebalance_emits_rebalance_event() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    client.rebalance(&symbol_short!("none"), &500_i128);

    let rebalance_events =
        find_events_by_topic(env.events().all(), &env, symbol_short!("rebalance"));
    assert!(!rebalance_events.is_empty(), "rebalance should emit an event");
}
