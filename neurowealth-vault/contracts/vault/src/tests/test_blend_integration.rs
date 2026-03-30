#![cfg(test)]

use crate::{NeuroWealthVault, NeuroWealthVaultClient};
use soroban_sdk::{
    contract, contractimpl, testutils::Address as _, token, Address, Env, IntoVal, Symbol,
};

#[contract]
pub struct MockBlendPool;

#[contractimpl]
impl MockBlendPool {
    pub fn supply(env: Env, asset: Address, amount: i128, to: Address) -> i128 {
        // Mock supply logic: return amount
        amount
    }

    pub fn withdraw(env: Env, asset: Address, amount: i128, to: Address) -> i128 {
        // Mock withdraw logic: return amount
        amount
    }

    pub fn get_user_account_data(env: Env, user: Address, asset: Address) -> i128 {
        // Mock balance: return a static balance
        1000
    }
}

pub struct MockBlendTestSetup {
    pub env: Env,
    pub vault: NeuroWealthVaultClient<'static>,
    pub agent: Address,
    pub owner: Address,
    pub usdc: Address,
    pub blend_pool: Address,
}

fn setup_vault_with_blend(env: &Env) -> MockBlendTestSetup {
    env.mock_all_auths();

    // Register Vault
    let vault_id = env.register_contract(None, NeuroWealthVault);
    let vault_client = NeuroWealthVaultClient::new(env, &vault_id);

    // Register Mock Blend
    let blend_pool_id = env.register_contract(None, MockBlendPool);

    let agent = Address::generate(env);
    let owner = agent.clone();
    let usdc = Address::generate(env);

    vault_client.initialize(&agent, &usdc);
    vault_client.set_blend_pool(&owner, &blend_pool_id);

    MockBlendTestSetup {
        env: env.clone(),
        vault: vault_client,
        agent,
        owner,
        usdc,
        blend_pool: blend_pool_id,
    }
}

#[test]
fn test_blend_integration_supply() {
    let env = Env::default();
    let setup = setup_vault_with_blend(&env);
    let client = &setup.vault;

    // We emulate a rebalance flow to trigger `supply` to Blend pool
    // Need to first give the vault some USDC balance.
    // However, since we mock_all_auths and we don't have a real USDC token in this basic test,
    // the usdc transfer might fail if it's not registered.
    // Let's register a mock token or let it panic if we don't handle it.
}
