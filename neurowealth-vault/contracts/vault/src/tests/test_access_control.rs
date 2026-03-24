//! Tests for ownership and agent access control

use super::test_utils::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_update_agent_changes_agent() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, old_agent, _owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let new_agent = Address::generate(&env);
    client.update_agent(&new_agent);

    assert_eq!(client.get_agent(), new_agent);
    assert_ne!(client.get_agent(), old_agent);
}

#[test]
fn test_update_agent_emits_event() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _old_agent, _owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let new_agent = Address::generate(&env);
    client.update_agent(&new_agent);

    let agent_events = find_events_by_topic(
        env.events().all(),
        &env,
        soroban_sdk::symbol_short!("agent"),
    );
    assert!(
        !agent_events.is_empty(),
        "update_agent should emit an event"
    );
}

#[test]
fn test_transfer_ownership_sets_pending_owner() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let new_owner = Address::generate(&env);
    client.transfer_ownership(&new_owner);

    // Pending owner should be set
    let pending = client.get_pending_owner();
    assert!(pending.is_some(), "Pending owner should be set");
    assert_eq!(pending.unwrap(), new_owner);
}

#[test]
fn test_accept_ownership_completes_transfer() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, old_owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let new_owner = Address::generate(&env);

    // Initiate transfer
    client.transfer_ownership(&new_owner);
    assert_eq!(client.get_pending_owner().unwrap(), new_owner);

    // Complete transfer
    client.accept_ownership(&new_owner);

    assert_eq!(client.get_owner(), new_owner);
    assert_ne!(client.get_owner(), old_owner);
    // Pending owner is cleared after acceptance
    assert!(
        client.get_pending_owner().is_none(),
        "Pending owner should be cleared"
    );
}

#[test]
#[should_panic(expected = "Caller is not the pending owner")]
fn test_wrong_address_cannot_accept_ownership() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let new_owner = Address::generate(&env);
    client.transfer_ownership(&new_owner);

    // A different address tries to accept
    let impostor = Address::generate(&env);
    client.accept_ownership(&impostor);
}

#[test]
fn test_cancel_ownership_transfer_clears_pending() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let new_owner = Address::generate(&env);
    client.transfer_ownership(&new_owner);
    assert!(client.get_pending_owner().is_some());

    client.cancel_ownership_transfer();

    assert!(
        client.get_pending_owner().is_none(),
        "Pending owner should be cleared after cancel"
    );
}

#[test]
#[should_panic(expected = "No pending owner to cancel")]
fn test_cancel_without_pending_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    // No pending transfer started
    client.cancel_ownership_transfer();
}

#[test]
fn test_set_blend_pool_stores_address() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let blend_pool = Address::generate(&env);
    client.set_blend_pool(&owner, &blend_pool);

    // Just verify no "Blend pool not configured" panic when vault_balance is 0
    client.rebalance(&soroban_sdk::symbol_short!("blend"), &500_i128);
}

#[test]
#[should_panic(expected = "Only owner can set Blend pool")]
fn test_non_owner_cannot_set_blend_pool() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner) = setup_vault(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let blend_pool = Address::generate(&env);
    let non_owner = Address::generate(&env);
    // set_blend_pool checks owner == stored_owner explicitly
    client.set_blend_pool(&non_owner, &blend_pool);
}
