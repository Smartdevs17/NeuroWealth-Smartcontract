//! Shared test utilities for NeuroWealth Vault tests

extern crate std;

use soroban_sdk::{
    contract, contractimpl, contracttype, testutils::Address as _, Address, Env, Symbol,
    TryFromVal, Val, Vec,
};

// Re-export so each submodule only needs `use super::utils::*;`
pub use crate::{NeuroWealthVault, NeuroWealthVaultClient};
pub use soroban_sdk::testutils::Events;

// ============================================================================
// SIMPLE TEST TOKEN CONTRACT
// ============================================================================

#[contracttype]
enum TokenDataKey {
    Balance(Address),
    Allowance(Address, Address),
    AllowanceExpiration(Address, Address),
}

#[derive(Clone)]
#[contracttype]
enum BlendMockDataKey {
    Supplied(Address),
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
        env: Env,
        from: Address,
        spender: Address,
        amount: i128,
        expiration_ledger: u32,
    ) {
        from.require_auth();
        assert!(amount >= 0, "amount must be non-negative");

        env.storage().persistent().set(
            &TokenDataKey::Allowance(from.clone(), spender.clone()),
            &amount,
        );
        env.storage().persistent().set(
            &TokenDataKey::AllowanceExpiration(from, spender),
            &expiration_ledger,
        );
    }

    pub fn allowance(env: Env, from: Address, spender: Address) -> i128 {
        let expiration: u32 = env
            .storage()
            .persistent()
            .get(&TokenDataKey::AllowanceExpiration(
                from.clone(),
                spender.clone(),
            ))
            .unwrap_or(0);

        if expiration > 0 && expiration < env.ledger().sequence() {
            return 0;
        }

        env.storage()
            .persistent()
            .get(&TokenDataKey::Allowance(from, spender))
            .unwrap_or(0)
    }

    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();
        assert!(amount > 0, "amount must be positive");

        let allowance = Self::allowance(env.clone(), from.clone(), spender.clone());
        assert!(allowance >= amount, "insufficient allowance");

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

        env.storage().persistent().set(
            &TokenDataKey::Balance(from.clone()),
            &(from_balance - amount),
        );
        env.storage()
            .persistent()
            .set(&TokenDataKey::Balance(to), &(to_balance + amount));
        env.storage().persistent().set(
            &TokenDataKey::Allowance(from, spender.clone()),
            &(allowance - amount),
        );
    }
}

#[contract]
pub struct MockBlendPool;

#[contractimpl]
impl MockBlendPool {
    pub fn submit_with_allowance(
        env: Env,
        from: Address,
        spender: Address,
        _to: Address,
        requests: Vec<crate::BlendRequest>,
    ) -> i128 {
        assert_eq!(requests.len(), 1, "expected one request");
        let request = requests.get(0).unwrap();
        assert_eq!(request.request_type, 0, "expected supply request");

        let token_client = TestTokenClient::new(&env, &request.address);
        let allowance = token_client.allowance(&spender, &env.current_contract_address());
        assert!(
            allowance >= request.amount,
            "expected allowance before pool pull"
        );

        token_client.transfer_from(
            &env.current_contract_address(),
            &spender,
            &env.current_contract_address(),
            &request.amount,
        );

        let total_supplied: i128 = env
            .storage()
            .persistent()
            .get(&BlendMockDataKey::Supplied(request.address.clone()))
            .unwrap_or(0);
        env.storage().persistent().set(
            &BlendMockDataKey::Supplied(request.address),
            &(total_supplied + request.amount),
        );

        from.clone().require_auth();

        request.amount
    }

    pub fn supplied(env: Env, asset: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&BlendMockDataKey::Supplied(asset))
            .unwrap_or(0)
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

pub fn setup_vault_with_token_and_blend(
    env: &Env,
) -> (Address, Address, Address, Address, Address) {
    let (contract_id, agent, owner, usdc_token) = setup_vault_with_token(env);
    let blend_pool = env.register_contract(None, MockBlendPool);

    (contract_id, agent, owner, usdc_token, blend_pool)
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
