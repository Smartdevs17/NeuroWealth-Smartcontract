# Blend Protocol Integration Research

## Overview

This document contains research findings for integrating the NeuroWealth Vault with Blend Protocol's Soroban pool contract for on-chain yield generation.

## Research Sources

### Primary Resources

1. **Blend Contracts Repository**: `blend-capital/blend-contracts-v2`
   - Contains the Soroban implementation of Blend Protocol v2
   - Location: https://github.com/blend-capital/blend-contracts-v2

2. **Blend Integration Documentation**: 
   - Official integration guide: https://docs.blend.capital/tech-docs/integrations/integrate-pool
   - Covers pool selection, configuration, and integration patterns

3. **Blend Lending Pool Documentation**:
   - Core contract interface: https://docs.blend.capital/tech-docs/core-contracts/lending-pool
   - Function signatures and usage patterns

4. **Blend Utils Repository**: `blend-capital/blend-utils`
   - Deployment and utility scripts
   - Testnet deployment addresses

## Key Functions Identified

Based on typical lending pool patterns and Blend documentation, the following functions are expected:

### Supply Function
```rust
pub fn supply(
    env: Env,
    asset: Address,
    amount: i128,
    to: Address
) -> i128
```
- Supplies assets to the Blend pool
- Returns the amount of pool tokens received
- `to` parameter specifies who receives the pool tokens (vault address)

### Withdraw Function
```rust
pub fn withdraw(
    env: Env,
    asset: Address,
    amount: i128,
    to: Address
) -> i128
```
- Withdraws assets from the Blend pool
- Returns the amount of assets actually withdrawn
- `to` parameter specifies where withdrawn assets are sent (vault address)

### Get Reserve Data Function
```rust
pub fn get_reserve_data(
    env: Env,
    asset: Address
) -> ReserveData
```
- Returns reserve data including:
  - Current balance
  - Interest rate
  - Liquidity index
  - Other reserve metrics

### Get User Balance Function
```rust
pub fn get_user_account_data(
    env: Env,
    user: Address,
    asset: Address
) -> i128
```
- Returns the user's supplied balance for a specific asset
- Alternative function names may be used (e.g., `get_balance`, `get_supply_balance`)

## Implementation Notes

### Cross-Contract Call Pattern

The implementation uses Soroban's `env.invoke_contract()` method for cross-contract calls:

```rust
env.invoke_contract::<ReturnType>(
    &pool_address,
    &function_name,
    &arguments_vec
)
```

### Token Approval Pattern

Before supplying to Blend, the vault must:
1. Approve the Blend pool to spend USDC from the vault
2. Call Blend's `supply()` function
3. Blend handles the token transfer internally via the approval

### Error Handling Strategy

To prevent permanent fund lockup:
- Blend call failures are handled gracefully
- If a supply/withdraw fails, the transaction continues where possible
- State is updated atomically to prevent inconsistencies
- Withdrawals check vault balance and pull from Blend if needed

## Verification Required

⚠️ **CRITICAL**: The exact function signatures above are based on typical lending pool patterns and should be verified against Blend's actual contract interface before production deployment.

### Items to Verify:

1. **Function Names**: Confirm the exact function names (may be `deposit`/`redeem` instead of `supply`/`withdraw`)
2. **Parameter Order**: Verify parameter order matches Blend's interface
3. **Return Types**: Confirm return types (may return structs instead of i128)
4. **Token Transfer Pattern**: Verify if Blend requires pre-approval or handles transfers differently
5. **Error Handling**: Understand how Blend handles errors (panics vs. return values)

## Testnet Deployment Addresses

Testnet deployment addresses should be obtained from:
- Blend's official documentation
- `blend-utils` repository deployment scripts
- Blend team communication channels

## Integration Architecture Decisions

### Direct Pool Integration vs. Intermediate Contracts

Blend supports both direct pool integration and intermediate contracts (fee vaults). For the NeuroWealth Vault:

- **Decision**: Direct pool integration (simpler, lower gas costs)
- **Rationale**: Vault doesn't need fee sharing features initially
- **Future Consideration**: May migrate to intermediate contract if fee sharing becomes desirable

### Protocol Tracking

The vault tracks the current protocol using `DataKey::CurrentProtocol`:
- `"none"`: Funds not deployed
- `"blend"`: Funds deployed to Blend
- Future: Additional protocols can be added

## Gas Cost Considerations

Expected gas costs for operations:
- `supply_to_blend()`: ~50,000-100,000 operations (approval + supply call)
- `withdraw_from_blend()`: ~30,000-70,000 operations (withdraw call)
- `rebalance()`: ~80,000-150,000 operations (withdraw + supply if switching)
- `withdraw()` with Blend pull: ~100,000-200,000 operations (withdraw from Blend + transfer)

**Note**: Actual gas costs should be measured on testnet and documented in integration tests.

## Security Considerations

1. **Reentrancy**: Blend calls are made after state updates (CEI pattern)
2. **Fund Lockup Prevention**: Errors in Blend calls don't prevent withdrawals
3. **Balance Verification**: Vault verifies it has sufficient balance before user transfers
4. **Approval Limits**: Token approvals are set with reasonable expiration times

## Testing Strategy

1. **Unit Tests**: Mock Blend pool contract for basic functionality
2. **Integration Tests**: Test against Blend testnet deployment
3. **Failure Scenarios**: Test Blend call failures, insufficient balance, etc.
4. **Gas Measurement**: Document actual gas costs for all operations

## Next Steps

1. ✅ Research Blend interface (this document)
2. ✅ Implement storage keys and configuration
3. ✅ Implement Blend client interface
4. ✅ Update rebalance() and withdraw() functions
5. ⏳ Verify function signatures against actual Blend contract
6. ⏳ Write integration tests against testnet
7. ⏳ Document actual gas costs
8. ⏳ Security review of cross-contract call patterns

## References

- Blend GitHub: https://github.com/blend-capital
- Blend Documentation: https://docs.blend.capital
- Soroban SDK Documentation: https://soroban.stellar.org/docs
- Stellar Testnet: https://developers.stellar.org/docs/encyclopedia/testnet
