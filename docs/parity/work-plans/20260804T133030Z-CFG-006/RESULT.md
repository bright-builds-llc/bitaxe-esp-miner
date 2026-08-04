# CFG-006 work result

- Parity row: `CFG-006`
- Final status: `implemented`
- Implementation commit: `1583feb3966f32be73782ed22489b7a79a0dc248`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware attempts: none

## Evidence and verification

The Rust-owned defaults model now contains an exact, ordered 21-entry matrix:
all 20 numbered upstream board seed files plus the explicit custom override.
Each typed entry exposes the public seed identifier and reference path, board
version, device and ASIC models, frequency, voltage, display rotation, automatic
and manual fan defaults, self-test and overheat defaults, and primary pool port.

The pinned-reference golden fixture requires exact equality for every field and
records the reference commit and source pattern. Focused tests additionally
prove that:

- all 20 numbered seeds are selectable and have matching catalog entries;
- catalog device family, ASIC model, default frequency, and default voltage
  equal each seed;
- Ultra 205 is the sole active hardware-evidence scope;
- all other numbered seeds remain explicitly not hardware verified; and
- the custom seed is non-selectable, remains distinct from numbered seed 207,
  and preserves its one board-profile discriminator exception, pool port 21496.

The following gates passed on the implementation commit:

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bun scripts/bright-builds-check.ts all`
- `just test` (all 28 Bazel test targets passed)
- `just parity`
- `just parity-progress`
- `just verify-redaction`
- `just verify-reference`
- `git diff --check`

The focused `bitaxe-config` suite passed all 51 tests through both Cargo and
Bazel. The pinned reference remained clean.

## Conclusion

The complete upstream board-profile defaults matrix is implemented with typed
unit and golden evidence. The implementation is purely declarative and changes
neither runtime board selection nor firmware hardware behavior.

## Non-claims and residual evidence gap

`CFG-006` remains `implemented`, not `verified`. No non-205 profile has live
device evidence proving seeded defaults or runtime behavior, and only Ultra 205
may retain the active verification scope. No hardware, credentials, network,
mining, voltage, fan, thermal, power, OTA, recovery, direct UART, or pin action
was used or claimed.
