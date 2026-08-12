# Parity work plan

- Run ID: `20260812T004157Z-CFG-006`
- Parity row: `CFG-006`
- Initial status: `implemented`
- Source commit: `8203a96fe59baafe5f86ab8c11a61074c45fd19b`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-cfg006-defaults-matrix`

## Selection

The deterministic selector reported no open plan and listed `CFG-006` first.
No candidate was skipped. The row is actionable because its surface is the
declarative upstream board-profile/defaults matrix, not live non-205 runtime
behavior, and the checklist reserves mandatory hardware evidence for
safety-critical and hardware-control surfaces. The checked-out pinned
reference contains a bounded inventory of 20 numbered CSV seeds plus one
custom seed that can be compared directly to the existing public Rust matrix.

The earlier CFG-006 result proved equality between the Rust matrix and a
provenance-bearing golden fixture, but stopped at `implemented` because it
treated non-205 live behavior as part of this row. The remaining row-local
trust gap is narrower: no independent workflow currently compares the Rust
matrix to the actual pinned reference CSV files, so coordinated drift in the
fixture and implementation would remain undetected.

## Scope and non-scope

Add a pure, source-backed validator to the parity report. It must inventory
every checked-out `reference/esp-miner/config-*.cvs` file, parse the exact
board-profile fields represented by `BoardProfileDefaults`, and compare them
to `board_profile_defaults()` with closed cardinality, source-path, seed-kind,
type, encoding, and value checks. Missing, extra, duplicate, malformed, or
mismatched seeds must invalidate `just parity`. Keep the existing golden and
catalog cross-checks as an independent second evidence layer.

This plan does not change runtime board selection, firmware images, NVS data,
credentials, network behavior, mining, ASIC behavior, voltage, fan, thermal,
power, display, self-test, OTA, USB, direct UART, or pins. It does not claim
that any non-205 board boots or behaves correctly on hardware. The connected
Ultra 205 is not accessed because no hardware effect is relevant to this row.

## Implementation

- [ ] Add a typed reference-seed parser and direct matrix comparator with
      closed inventory and field validation.
- [ ] Integrate the comparator into parity-report validation and Bazel/Cargo
      ownership so `just parity` fails closed on reference or matrix drift.
- [ ] Add behavior-focused tests for the accepted inventory and representative
      missing, extra, malformed, duplicate, and mismatched inputs.
- [ ] Produce a verified `RESULT.md` only when direct reference comparison,
      existing golden/catalog tests, and all mandatory gates pass.

## Verification and promotion

Run focused `bitaxe-parity` and `bitaxe-config` Cargo/Bazel tests, then:

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bun scripts/bright-builds-check.ts all`
- `just test`
- `just parity`
- `just parity-progress`
- `just verify-redaction`
- `just verify-reference`
- `git diff --check`

Promote only `CFG-006` to `verified` with `unit,golden,workflow` evidence when
the direct pinned-reference comparison covers exactly all 21 seeds, the
existing golden and catalog checks pass, all verification gates pass, and the
result explicitly withholds non-205 runtime and hardware claims. Add the new
validator to the Rust-owned target cell. Any unresolved source mismatch,
inventory ambiguity, reference dirtiness, or gate failure keeps the row at
`implemented` and closes this plan truthfully without hardware fallback.
