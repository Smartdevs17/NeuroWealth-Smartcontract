//! Tests for share-based accounting (ERC-4626-inspired model)

use super::utils::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_first_deposit_mints_shares_one_to_one() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let amount = 5_000_000_i128;

    mint_and_deposit(&env, &client, &usdc_token, &user, amount);

    // First deposit: shares minted == assets deposited (1:1)
    assert_eq!(client.get_shares(&user), amount);
    assert_eq!(client.get_total_assets(), amount);
}

#[test]
fn test_total_shares_consistency_after_multiple_operations() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let token_client = TestTokenClient::new(&env, &usdc_token);

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    mint_and_deposit(&env, &client, &usdc_token, &user1, 10_000_000_i128);
    mint_and_deposit(&env, &client, &usdc_token, &user2, 5_000_000_i128);

    let total_after_deposits = client.get_total_shares();

    client.withdraw(&user1, &3_000_000_i128);

    let total_after_withdraw = client.get_total_shares();
    assert_eq!(total_after_withdraw, total_after_deposits - 3_000_000_i128);

    token_client.mint(&contract_id, &7_500_000_i128);
    client.update_total_assets(&agent, &(19_500_000_i128));

    assert_eq!(
        client.get_total_shares(),
        total_after_withdraw,
        "Total shares unchanged by yield"
    );

    mint_and_deposit(&env, &client, &usdc_token, &user1, 2_000_000_i128);

    assert_eq!(
        client.get_total_shares(),
        total_after_withdraw + 2_000_000_i128
    );
}

#[test]
fn test_new_user_has_zero_balance() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, _usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    assert_eq!(client.get_balance(&user), 0);
}

#[test]
fn test_two_deposits_share_proportional_ownership() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let amount1 = 10_000_000_i128;
    let amount2 = 5_000_000_i128;

    mint_and_deposit(&env, &client, &usdc_token, &user1, amount1);
    mint_and_deposit(&env, &client, &usdc_token, &user2, amount2);

    // With no yield, price stays 1:1
    assert_eq!(client.get_shares(&user1), amount1);
    assert_eq!(client.get_shares(&user2), amount2);
    assert_eq!(client.get_total_assets(), amount1 + amount2);
}

#[test]
fn test_convert_to_shares_returns_correct_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let deposit_amount = 10_000_000_i128;

    mint_and_deposit(&env, &client, &usdc_token, &user, deposit_amount);

    // With 1:1 price, convert_to_shares(assets) == assets
    let shares = client.convert_to_shares(&5_000_000_i128);
    assert_eq!(shares, 5_000_000_i128);
}

#[test]
fn test_convert_to_assets_returns_correct_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let deposit_amount = 10_000_000_i128;

    mint_and_deposit(&env, &client, &usdc_token, &user, deposit_amount);

    // With 1:1 price, convert_to_assets(shares) == shares
    let assets = client.convert_to_assets(&5_000_000_i128);
    assert_eq!(assets, 5_000_000_i128);
}

#[test]
fn test_share_price_increases_after_yield() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let token_client = TestTokenClient::new(&env, &usdc_token);

    let user = Address::generate(&env);
    let deposit_amount = 10_000_000_i128;
    mint_and_deposit(&env, &client, &usdc_token, &user, deposit_amount);

    let shares_before = client.get_shares(&user);

    // Simulate 50% yield: mint tokens to vault, then update total_assets
    let yield_amount = deposit_amount / 2;
    let new_total = deposit_amount + yield_amount;
    token_client.mint(&contract_id, &yield_amount);
    client.update_total_assets(&agent, &new_total);

    // After yield: same shares but each share is worth more assets
    assert_eq!(
        client.get_shares(&user),
        shares_before,
        "Share count unchanged by yield"
    );
    let balance_after = client.get_balance(&user);
    assert!(
        balance_after > deposit_amount,
        "User balance should grow with yield"
    );
    assert_eq!(balance_after, new_total);
}

#[test]
fn test_withdraw_burns_correct_shares() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let deposit_amount = 10_000_000_i128;
    mint_and_deposit(&env, &client, &usdc_token, &user, deposit_amount);

    let shares_before = client.get_shares(&user);
    let withdraw_amount = 4_000_000_i128;
    client.withdraw(&user, &withdraw_amount);

    let shares_after = client.get_shares(&user);
    assert!(shares_after < shares_before, "Withdrawal burns shares");
    assert_eq!(
        shares_after,
        shares_before - withdraw_amount,
        "Exactly withdraw_amount shares burned at 1:1"
    );
}

#[test]
fn test_total_shares_equals_sum_of_user_shares() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);
    let amounts = [3_000_000_i128, 5_000_000_i128, 2_000_000_i128];

    mint_and_deposit(&env, &client, &usdc_token, &user1, amounts[0]);
    mint_and_deposit(&env, &client, &usdc_token, &user2, amounts[1]);
    mint_and_deposit(&env, &client, &usdc_token, &user3, amounts[2]);

    let expected_total = amounts[0] + amounts[1] + amounts[2];

    let shares1 = client.get_shares(&user1);
    let shares2 = client.get_shares(&user2);
    let shares3 = client.get_shares(&user3);

    assert_eq!(shares1 + shares2 + shares3, expected_total);
    assert_eq!(client.get_total_assets(), expected_total);
}

#[test]
fn test_multi_user_principal_tracking_isolation() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    let deposit1 = 5_000_000_i128;
    let deposit2 = 3_000_000_i128;

    mint_and_deposit(&env, &client, &usdc_token, &user1, deposit1);
    mint_and_deposit(&env, &client, &usdc_token, &user2, deposit2);

    assert_eq!(client.get_balance(&user1), deposit1);
    assert_eq!(client.get_balance(&user2), deposit2);
    assert_eq!(client.get_total_assets(), deposit1 + deposit2);

    client.withdraw(&user1, &2_000_000_i128);

    assert_eq!(client.get_balance(&user1), deposit1 - 2_000_000_i128);
    assert_eq!(client.get_balance(&user2), deposit2);

    let remaining_shares1 = client.get_shares(&user1);
    let remaining_shares2 = client.get_shares(&user2);
    assert_eq!(
        remaining_shares1 + remaining_shares2,
        deposit1 + deposit2 - 2_000_000_i128
    );
}

#[test]
fn test_withdraw_never_over_withdraws() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let token_client = TestTokenClient::new(&env, &usdc_token);

    let user = Address::generate(&env);
    let deposit_amount = 10_000_000_i128;
    mint_and_deposit(&env, &client, &usdc_token, &user, deposit_amount);

    token_client.mint(&contract_id, &5_000_000_i128);
    client.update_total_assets(&agent, &(15_000_000_i128));

    let shares_before = client.get_shares(&user);
    let balance_before = client.get_balance(&user);

    client.withdraw(&user, &10_000_000_i128);

    let shares_after = client.get_shares(&user);
    assert_eq!(shares_after, shares_before - 10_000_000_i128);
    assert!(
        client.get_balance(&user) <= balance_before,
        "Balance should not increase from withdrawal"
    );
}

#[test]
fn test_proportional_shares_after_yield() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let token_client = TestTokenClient::new(&env, &usdc_token);

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let deposit1 = 10_000_000_i128;
    let deposit2 = 10_000_000_i128;

    mint_and_deposit(&env, &client, &usdc_token, &user1, deposit1);
    mint_and_deposit(&env, &client, &usdc_token, &user2, deposit2);

    let shares1_before = client.get_shares(&user1);
    let shares2_before = client.get_shares(&user2);
    assert_eq!(shares1_before, deposit1);
    assert_eq!(shares2_before, deposit2);

    token_client.mint(&contract_id, &10_000_000_i128);
    client.update_total_assets(&agent, &(30_000_000_i128));

    assert_eq!(
        client.get_shares(&user1),
        shares1_before,
        "Shares unchanged by yield"
    );
    assert_eq!(
        client.get_shares(&user2),
        shares2_before,
        "Shares unchanged by yield"
    );

    let balance1 = client.get_balance(&user1);
    let balance2 = client.get_balance(&user2);
    assert_eq!(balance1, 15_000_000_i128);
    assert_eq!(balance2, 15_000_000_i128);
}

#[test]
fn test_rounding_down_on_share_mint() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let token_client = TestTokenClient::new(&env, &usdc_token);

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    mint_and_deposit(&env, &client, &usdc_token, &user1, 3_i128);

    token_client.mint(&contract_id, &1_i128);
    client.update_total_assets(&agent, &(4_i128));

    mint_and_deposit(&env, &client, &usdc_token, &user2, 1_i128);

    let user2_shares = client.get_shares(&user2);
    assert!(
        user2_shares <= 1_i128,
        "Due to rounding, shares should be <= deposited amount"
    );
}

#[test]
fn test_withdraw_burns_proportional_shares_after_yield() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);
    let token_client = TestTokenClient::new(&env, &usdc_token);

    let user = Address::generate(&env);
    let deposit = 10_000_000_i128;
    mint_and_deposit(&env, &client, &usdc_token, &user, deposit);

    token_client.mint(&contract_id, &10_000_000_i128);
    client.update_total_assets(&agent, &(20_000_000_i128));

    let shares_before = client.get_shares(&user);
    let withdraw_amount = 5_000_000_i128;

    client.withdraw(&user, &withdraw_amount);

    let shares_after = client.get_shares(&user);
    assert_eq!(
        shares_after,
        shares_before - withdraw_amount,
        "At 2x price, should burn exactly withdraw_amount shares"
    );
}

#[test]
fn test_full_withdrawal_burns_all_shares() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, _agent, _owner, usdc_token) = setup_vault_with_token(&env);
    let client = NeuroWealthVaultClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let deposit = 7_500_000_i128;
    mint_and_deposit(&env, &client, &usdc_token, &user, deposit);

    assert_eq!(client.get_shares(&user), deposit);

    client.withdraw_all(&user);

    assert_eq!(client.get_shares(&user), 0);
    assert_eq!(client.get_balance(&user), 0);
}
