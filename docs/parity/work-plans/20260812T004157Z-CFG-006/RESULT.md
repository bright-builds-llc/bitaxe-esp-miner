# Parity work result

- Parity row: `CFG-006`
- Final status: `verified`
- Implementation commit: `428041800ae232955a7468c384527cde83263503`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware attempts: none

## Evidence and verification

The parity report now inventories every checked-out
`reference/esp-miner/config-*.cvs` file and compares the parsed projection
directly to `bitaxe_config::board_profile_defaults()`. The comparison closes
source-path and seed-ID uniqueness, numbered versus custom seed kind, exact
inventory membership, required field presence, CSV type and encoding,
canonical integer and boolean forms, and every modeled value. Any missing,
extra, duplicate, malformed, or mismatched seed invalidates `just parity` with
row ID `CFG-006`.

The real workflow found exactly the existing 20 numbered sources and one custom
source at pinned reference commit
`c1915b0a63bfabebdb95a515cedfee05146c1d50`, compared them to the Rust matrix,
and reported `validation_errors: none`. Existing independent tests continue to
compare the same Rust matrix to the provenance-bearing golden fixture and
cross-check every numbered profile against the typed board catalog.

Nine focused regressions prove accepted production-shaped input and rejection
of missing or extra sources, missing or duplicate fields, wrong encodings,
noncanonical integers, value drift, and duplicate matrix identities. The
following gates passed on implementation commit
`428041800ae232955a7468c384527cde83263503`:

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bun scripts/bright-builds-check.ts all`
- `just test` (all 37 Bazel test targets passed)
- `just parity` (`validation_errors: none`)
- `just parity-progress`
- `just verify-redaction`
- `just verify-reference`
- `git diff --check`

Focused `bitaxe-parity` Cargo tests, strict Clippy, and
`bazel test //tools/parity:tests` also passed. The reference submodule remained
clean and pinned.

## Conclusion

The complete declarative upstream board-profile/defaults matrix is verified by
three independent links: direct checked-out reference comparison, the checked-in
golden fixture, and typed catalog cross-checks. The new direct link eliminates
the prior coordinated fixture/implementation drift gap, so hardware is neither
necessary nor relevant to verification of this static matrix row.

## Non-claims and residual risks

This result does not prove that a non-205 board boots, seeds NVS, selects a
profile at runtime, or operates correctly on hardware. It does not verify
non-205 ASIC, power, voltage, fan, thermal, sensor, display, self-test, mining,
network, OTA, or recovery behavior. Those remain owned by their dedicated
rows and hardware gates. No hardware, credentials, network, settings, mining,
controls, OTA, direct UART, or pins were accessed or changed.
