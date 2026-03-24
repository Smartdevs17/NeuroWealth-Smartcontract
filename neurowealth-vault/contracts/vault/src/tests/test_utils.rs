//! Shared test utilities for NeuroWealth Vault tests

extern crate std;

use soroban_sdk::{
    contract, contractimpl, contracttype, testutils::Address as _, Address, Env, Symbol,
    TryFromVal, Val, Vec,
};

// Re-export so each submodule only needs `use super::test_utils::*;`
pub use crate::{NeuroWealthVault, NeuroWealthVaultClient};
pub use soroban_sdk::testutils::Events;

// ============================================================================
// SIMPLE TEST TOKEN CONTRACT
// ============================================================================

#[contracttype]
enum TokenDataKey {
    Balance(Address),
}

#[contract]
pub struct TestToken;

#[contractimpl]
impl TestToken {
    pub fn mint(env: Env, to: Address, amount: i128) {
        let balance: i128 = env
            .storage()
            .persistent()
            .get(&TokenDataKey::Balance(to.clone()))
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&TokenDataKey::Balance(to), &(balance + amount));
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        assert!(amount > 0, "amount must be positive");

        let from_balance: i128 = env
            .storage()
            .persistent()
            .get(&TokenDataKey::Balance(from.clone()))
            .unwrap_or(0);
        assert!(from_balance >= amount, "insufficient balance");

        let to_balance: i128 = env
            .storage()
            .persistent()
            .get(&TokenDataKey::Balance(to.clone()))
            .unwrap_or(0);

        env.storage()
            .persistent()
            .set(&TokenDataKey::Balance(from), &(from_balance - amount));
        env.storage()
            .persistent()
            .set(&TokenDataKey::Balance(to), &(to_balance + amount));
    }

    pub fn balance(env: Env, owner: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&TokenDataKey::Balance(owner))
            .unwrap_or(0)
    }

    pub fn approve(
        _env: Env,
        _from: Address,
        _spender: Address,
        _amount: i128,
        _expiration_ledger: u32,
    ) {
        // Stub – no-op for testing
    }
}

// ============================================================================
// TEST SETUP FUNCTIONS
// ============================================================================

/// Sets up a vault with a mock (non-functional) USDC token address.
pub fn setup_vault(env: &Env) -> (Address, Address, Address) {
    let contract_id = env.register_contract(None, NeuroWealthVault);
    let client = NeuroWealthVaultClient::new(env, &contract_id);

    let agent = Address::generate(env);
    let usdc_token = Address::generate(env);
    let owner = agent.clone();

    client.initialize(&agent, &usdc_token);

    (contract_id, agent, owner)
}

/// Sets up a vault with a real deployed TestToken contract.
pub fn setup_vault_with_token(env: &Env) -> (Address, Address, Address, Address) {
    let contract_id = env.register_contract(None, NeuroWealthVault);
    let client = NeuroWealthVaultClient::new(env, &contract_id);
    let agent = Address::generate(env);
    let usdc_token = env.register_contract(None, TestToken);
    let owner = agent.clone();

    client.initialize(&agent, &usdc_token);

    (contract_id, agent, owner, usdc_token)
}

// ============================================================================
// EVENT HELPERS
// ============================================================================

/// Returns all events whose topics contain `topic`.
///
/// `env.events().all()` (requires `Events` trait in scope) yields
/// `(contract_address, topics, data)` tuples. The first element is the
/// emitting contract's address; the second is a `soroban_sdk::Vec<Val>` of
/// topic values; the third is the event data `Val`.
pub fn find_events_by_topic(
    events: Vec<(Address, Vec<Val>, Val)>,
    env: &Env,
    topic: Symbol,
) -> std::vec::Vec<(Address, Vec<Val>, Val)> {
    let mut result = std::vec::Vec::new();
    for i in 0..events.len() {
        if let Some((contract_addr, topics, data)) = events.get(i) {
            for j in 0..topics.len() {
                if let Some(t) = topics.get(j) {
                    if let Ok(s) = Symbol::try_from_val(env, &t) {
                        if s == topic {
                            result.push((contract_addr.clone(), topics.clone(), data));
                            break;
                        }
                    }
                }
            }
        }
    }
    result
}

// ============================================================================
// DEPOSIT HELPER
// ============================================================================

/// Mints `amount` test tokens for `user` and deposits them into the vault.
pub fn mint_and_deposit(
    env: &Env,
    vault_client: &NeuroWealthVaultClient,
    token_address: &Address,
    user: &Address,
    amount: i128,
) {
    let token_client = TestTokenClient::new(env, token_address);
    token_client.mint(user, &amount);
    vault_client.deposit(user, &amount);
}
