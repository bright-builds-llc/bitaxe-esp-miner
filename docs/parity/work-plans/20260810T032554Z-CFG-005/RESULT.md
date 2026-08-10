# Parity work result

- Parity row: `CFG-005`
- Final status: `verified`
- Implementation commit: `5faf33c119653b58abe857425e5a46fad06a0a08`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The reference-derived fixture at
`crates/bitaxe-api/fixtures/api/settings-patch-cases.json` covers every one of
the 42 upstream REST settings fields and proves the exact 44 typed NVS writes,
including the frequency and manual-fan legacy mirrors. Pure API transaction
tests prove whole-request validation before adapter access, unknown-only no-op
behavior, ordered typed writes, one commit, independent exact reconciliation,
post-commit uncertainty, serialized ownership through publication, and
hostname effects only after confirmed success. Firmware source-ownership tests
bind the production PATCH route to that contract, all string, `u16`, `i32`, and
`u64` ESP-IDF NVS operations, private reconciliation, and the non-secret public
snapshot filter.

The following commands passed against implementation commit
`5faf33c119653b58abe857425e5a46fad06a0a08`:

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bun scripts/bright-builds-check.ts all`
- `cargo test -p bitaxe-api --all-features`
- `bazel test //crates/bitaxe-api:tests //firmware/bitaxe:settings_source_ownership_tests --test_output=errors`
- `bazel build //firmware/bitaxe:firmware`
- `just verify-reference`
- `just verify-redaction`
- `git diff --check`

The canonical firmware build produced
`bazel-bin/firmware/bitaxe/bitaxe-firmware.elf`; reference verification bound
the unchanged pinned reference; redaction verification passed all committed
evidence roots; and the immutable plan SHA-256 remained
`edb4a077309e55e1257b04d300ab776ad8ace87007b5926f56c9b52b79265626`.

## Conclusion

The production settings PATCH path now persists every validated upstream
setting through the same closed, serialized write-commit-reread-reconcile-
publish contract. Invalid known fields fail before storage, unknown fields are
ignored, exact typed values and compatibility mirrors match the pinned
reference-derived fixture, and public diagnostics and snapshots do not expose
credential values. This satisfies `CFG-005` with `unit,golden,workflow`
software evidence.

## Non-claims and residual risks

This result does not claim physical NVS-media durability, live hostname
application, Wi-Fi or pool reconnection, credential consumption, mining or
ASIC effects, display effects, voltage, frequency, fan, thermal, or power
actuation, OTA, recovery, non-Ultra-205 behavior, or any `API-003` status.
Those live and hardware effects retain their separate evidence and safety
gates. A direct host-target Cargo build of the firmware remains unsupported;
the repo-owned Bazel ESP32-S3 build is the canonical firmware build path.
