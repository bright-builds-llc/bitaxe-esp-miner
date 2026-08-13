# Parity work plan

- Run ID: `20260813T015631Z-THR-001`
- Parity row: `THR-001`
- Initial status: `implemented`
- Source commit: `f5190e234d954356c4fd3b310a85600840128d31`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-thr001-emc2101-live-thermal`

## Selection

The synchronized selector reports no open plan and ranks `API-009` first.
That row remains ineligible at its sealed repeated boundary. `THR-001` is next
and actionable because its attempt-002 closure identified a new discriminating
software cause after the attempt-001 source-admission defect was fixed.

Attempt-002 proved every source, exact-package, boot, safety, privacy, and live
thermal member up to acquisition-stamp parsing. Its acquisition-stamp
`bootSession` is a valid nonnegative JSON integer wider than JavaScript's exact
integer range. The TypeScript shell parsed it as `number` and rejected it with
`Number.isSafeInteger` before equality comparison. This plan moves exact
private-input thermal validation to Rust/serde_json, where all stamp members
remain `u64`, and leaves TypeScript responsible only for orchestration,
surrounding typed API validation, and aggregate evidence publication.

The active lesson hashes still match the 2026-08-03 audit baseline; no audit
trigger changed. The previously loaded safety, authorization, evidence,
privacy, retry, redaction, real-process, ESP-IDF, and host-policy lessons and
the repo/Bright Builds architecture, code-shape, verification, testing, Rust,
and TypeScript standards remain in force. The previously disclosed unrelated
lesson blocks remain unloaded.

## Scope and non-scope

Add a private Rust `validate_emc2101_thermal_inputs` binary accepting exactly
the protected HTTP snapshot and WebSocket envelope paths. It must deserialize
the temperature and acquisition-stamp members with exact types, require one
`update` envelope, finite plausible below-throttle equal temperatures, fresh
equal states, and exactly equal `u64` boot-session, sequence, and acquisition-
time fields. It emits no values and succeeds only on the complete quorum.

Refactor the TypeScript shell to invoke this validator after protected layout,
source, package, and system-info validation. Remove JavaScript numeric stamp
validation and comparison while preserving its string package/boot identity
checks and every publication, cleanup, redaction, and typed-failure rule.
Advance the closed final evidence contract, validator, generated binding,
paths, and task binding to attempt ordinal 3.

After the fix is clean, committed, pushed, and packaged, run one fresh
detector-gated attempt-003. It may factory-flash one exact clean board-205
package, perform normal USB reset/re-enumeration, derive one same-session origin
only from protected serial evidence, and issue read-only same-origin HTTP,
WebSocket, and retained-log requests. At most one exact-package recovery flash
is permitted after an initial flash effect when safe recovery cannot otherwise
be proved.

No settings mutation, restart request, mining, pool credential, ASIC work,
voltage, frequency, fan, power, raw I2C/GPIO, OTA, erase, fault injection,
physical manipulation, direct UART, pin, pad, header, probe, jumper, solder,
or injected signal is in scope. Attempts 001 and 002 remain sealed and are not
runtime inputs to attempt-003 or public evidence.

NeverPersistRaw values remain memory-only. The Rust validator receives only
protected file paths and emits no raw data. Hostnames, origins, ports, USB and
network identities, settings, HTTP bodies, acquisition stamps, boot sessions,
raw temperatures, logs, commands, PIDs, and traces remain mode `0600` below an
ignored mode `0700` parent. Public evidence remains limited to closed schemas,
commits, opaque digests, fixed public reference constants, counts, categories,
and booleans.

## Implementation

- [ ] Add the private Rust input validator with exact `u64` stamp parsing and
      focused wide, maximum, mismatched, negative, fractional, malformed,
      stale, unsafe-temperature, and wrong-envelope tests.
- [ ] Replace TypeScript's lossy stamp-number path with the Rust validator
      subprocess while preserving typed categories, protected paths, cleanup,
      package/boot identity checks, aggregate evidence, and zero raw output.
- [ ] Add fake-process and real-child regressions proving both protected input
      paths reach the validator, wide equal tokens pass, mismatches withhold
      evidence, and sensitive values never enter public output.
- [ ] Advance the final contract, independent validator, generated binding,
      protected paths, plan/task binding, and tests to attempt ordinal 3.
- [ ] Commit and push the complete software fix, build and admit an exact clean
      package, then run exactly one detector-gated read-only attempt-003.
- [ ] Publish and independently validate the THR-001 projection only if every
      source, package, thermal, safety, cleanup, and privacy member passes;
      otherwise preserve `implemented`, the earliest typed failure, recovery
      facts, evidence withholding, and the accepted terminal outcome.
- [ ] Transition only THR-001, synchronize progress, complete the result/task
      review, and archive the task atomically only after the full quorum passes.

## Verification and promotion

Before hardware, run focused Rust private-input and final-contract tests,
TypeScript orchestration/invocation/typed-failure/redaction/protected-file
tests, real-child tests, source-admission tests, the real firmware build and
exact package, then:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`
9. `just verify-redaction`
10. `just verify-reference`
11. immutable-plan, unique-task, generated-contract, exact-package,
    source/reference, candidate-absence, mode, sensitive-output, and diff checks

The local default Cargo target cache has a confirmed macOS inode/provenance
stall unrelated to source; use the already-proven fresh temporary
`CARGO_TARGET_DIR=/tmp/bitaxe-thr001-cargo.ppwaWB` for Cargo and Bazel
workspace-status processes throughout this plan. Do not delete or broaden work
around the unhealthy cache during parity work.

After immutable plan/task and complete implementation are clean, separately
committed, and pushed, the only authorized hardware sequence is:

1. `test ! -e scratch/thr001-emc2101/wrapper-003 && (umask 077; mkdir -m 700 -p scratch/thr001-emc2101/wrapper-003 && just detect-ultra205 > scratch/thr001-emc2101/wrapper-003/detector.stdout 2> scratch/thr001-emc2101/wrapper-003/detector.stderr)`
2. Only after command 1 admits exactly one Ultra 205 and the ignored local
   Wi-Fi credential file exists:
   `test ! -e scratch/thr001-emc2101/attempt-003 && test ! -e docs/parity/evidence/thr001-emc2101-thermal/thermal-projection.json && (umask 077; just capture-emc2101-thermal-evidence --private-root scratch/thr001-emc2101/attempt-003 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/thr001-emc2101/wrapper-003/detector.stdout --projection docs/parity/evidence/thr001-emc2101-thermal/thermal-projection.json --capture-timeout-seconds 360 > scratch/thr001-emc2101/wrapper-003/capture.stdout 2> scratch/thr001-emc2101/wrapper-003/capture.stderr)`

The wrapper must be mode `0700`; its streams mode `0600`; the attempt child,
projection, and candidate absent before launch. Starting command 2 consumes
attempt-003. Preserve the earliest typed failure. Non-ready hardware maps to
`hardware_blocked`, malformed/incomplete inputs or validator rejection to
`evidence_invalid`, child timeout to `timeout`, and launch failure to
`process_failed`; recovery is secondary. No unchanged retry is permitted.

Promote THR-001 only if the typed projection binds board 205, attempt ordinal
3, exact clean source/reference/package identity, EMC2101 address `0x4c`,
register `0x00`, board offset `+5 C`, production read-only acquisition, one
finite plausible below-throttle fresh sample, exact lossless HTTP/WebSocket
temperature/state/u64-stamp/boot/package correlation, detector admission,
stable safe boot, disabled mining/hardware control, cleanup, private modes,
both independent validators, source cleanliness, and passed redaction. Any
missing, malformed, unsafe, incoherent, drifted, or privacy-invalid member
withholds the projection, keeps THR-001 `implemented`, creates a truthful
closure, and stops without retry.
