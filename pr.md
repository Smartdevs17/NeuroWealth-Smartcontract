# Pull Request: Add Mock Blend Pool for Unit Testing

## Description
This PR implements a robust on-chain mock for the Blend lending pool and integrates it into the unit testing suite. It also resolves several critical issues in the vault's Blend integration logic and standardizes error messaging across the codebase.

### Key Changes
- **MockBlendPool Implementation**: Created a comprehensive `MockBlendPool` in `utils.rs` that supports `submit_with_allowance`, `submit`, and `balance` entrypoints, simulating real Blend protocol behavior.
- **Enhanced Integration Testing**: Updated `test_blend_integration.rs` to use the new mock, validating argument encoding, cross-contract token flows, and event emissions for both supply and withdrawal paths.
- **Vault Logic Consolidation**: Fixed syntax errors in `lib.rs` and consolidated the `BlendPoolClient` helper to use modern Blend entrypoints (`submit_with_allowance` and `submit`) instead of legacy placeholders.
- **Standardized Error Messages**: Updated all `should_panic` test expectations and contract panic messages to strictly follow the project's `ERROR_STYLE_GUIDE.md` (`vault: <category> <description>`).
- **CI Workflow Validation**: Verified the entire pipeline locally:
  - ✅ `cargo fmt` (Format check)
  - ✅ `cargo clippy` (Lint check)
  - ✅ `cargo test` (All 199 tests passing)
  - ✅ `cargo build` (WASM target build successful)

## Acceptance Criteria Verified
- [x] Implement MockBlendPool contract (deposit/redeem/user data).
- [x] Wire unit tests that validate argument encoding, token flows, and returns.
- [x] Tests exercise real `env.invoke_contract` paths against the mock.
- [x] Placeholders removed from tests.
- [x] All CI workflow commands pass.

## Related Issues
- Resolves: "Add minimal mock Blend pool contract for unit tests"
