# REL-002 work result

- Parity row: `REL-002`
- Final status: `verified`
- Implementation and package commit:
  `e6b260da5717bf807eb85b9cfdbb20fe54b7b3a6`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Board: Ultra 205
- Hardware attempts after the retained-log correction: one

## Evidence and verification

The detector-gated attempt-005 used the exact normal and isolated rollback-
probe artifacts built from the clean pushed implementation commit. Before the
effect, source and reference provenance, clean-build status, distinct normal
and probe application identities, probe image digest, rollback SDK settings,
credential opacity, and fresh private/public paths all passed. The detector
then admitted exactly one Ultra 205.

The committed
[`bitaxe-sdkconfig-rollback-evidence-v1`](../../evidence/rel002-sdkconfig-rollback/sdkconfig-rollback-projection.json)
projection has SHA-256
`2c4387346d91ae4f265c149ab32b66ffa032cfa641f82ae6772f9b8ce0533c0d`.
The independent Rust validator and aggregate admission checks prove:

- one reset-aborted partial application upload retained the canonical protocol
  error while the factory boot session, ordinal, and exact build stayed fixed;
- the same physical board booted the exact pending-validation probe in `ota_0`
  at ordinal `N+1`, with one request, exact build identity, a changed boot
  session, software-reset attribution, correlated pre/post serial delivery,
  and complete cleanup;
- the probe retained the exact pending-validation and passive safe-state boot
  lines even though the post-re-enumeration serial fragment was intentionally
  not used as their semantic evidence source;
- one normal software restart caused native ESP-IDF rollback to the exact
  factory build at the next ordinal, on the same device, with the same typed
  request, identity, reset, serial-correlation, postcondition, and cleanup
  guarantees;
- the final retained log contained the exact passive safe-state line;
- mining and hardware control remained disabled; and
- the normal package was restored without a recovery flash, all private modes
  passed, cleanup completed, and public redaction passed.

The corrected host transaction isolates exact retained-line matching in
`sdkconfig-rollback-retained-log.ts`. The orchestration fetches protected
same-origin probe and final logs only after exact HTTP identity admission,
while both authoritative device-session projections still require reader
arming, correlated serial delivery, same-device reacquisition, exact build and
ordinal transitions, postconditions, and cleanup. Regression coverage proves
success with late serial fragments that contain no boot markers, and typed
failure for missing or unavailable probe/final retained evidence, recovery,
primary-failure precedence, withheld output, and privacy.

The focused automation target and the following mandatory gates passed before
hardware use and after final record updates:

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
- the canonical parity selector, immutable-plan digest, task uniqueness,
  reference-cleanliness, private-mode, redaction, fresh-path, and diff checks

## Conclusion

The exact pushed Rust firmware and host workflow proved both interrupted-
application-update safety and native post-OTA rollback on one detector-
admitted Ultra 205. The evidence binds the same physical device, exact normal
and probe identities, both required ordinal advances, retained boot semantics,
disabled mining/control, cleanup, restoration, and redaction. This satisfies
the narrow `REL-002` SDK rollback and interrupted-update behavior claim.

## Privacy, recovery, and non-claims

USB identities and paths, serial and HTTP traces, device origins, hostnames,
network identifiers, commands, firmware bytes, process identities, and Wi-Fi
credentials remain only in ignored mode-`0700` roots with mode-`0600` files.
The committed projection contains closed booleans, bounded counts, category
labels, and cryptographic build identities. The credential file was passed as
an opaque local input and was never read, printed, summarized, or copied.
Normal restoration passed, so the conditional recovery flash did not run.
Earlier failed attempts remain recorded and are not reinterpreted as success.

This result does not claim OTAWWW or SPIFFS update, recovery-page upload, large
erase, power-loss rollback, anti-rollback/eFuse behavior, mining, ASIC work,
pool access, voltage, frequency, fan, thermal or power control, other boards,
release readiness, direct UART, or pin manipulation.
