# Parity work plan

- Run ID: `20260813T011207Z-THR-001`
- Parity row: `THR-001`
- Initial status: `implemented`
- Source commit: `70329dabef817d63eb5590a24b12a3a7be80e113`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-thr001-emc2101-live-thermal`

## Selection

The synchronized selector reports no open plan and ranks `API-009` first.
That row remains ineligible: its sealed attempt-007 closure selected
`stop_repeated_boundary`, and attempt-008 is prohibited until a separate
software-only diagnostic proves and fixes a new discriminating cause.

`THR-001` is next and actionable. Its preceding immutable plan at
`docs/parity/work-plans/20260813T001637Z-THR-001/PLAN.md` consumed attempt-001
and closed as `stop_impossible_contract`. Protected boolean-only diagnosis
proved that the detector, exact package, stable safe boot, private modes, and
fresh finite below-throttle HTTP/WebSocket thermal sample and correlation all
passed. Publication failed only because the host source-admission map expected
an intermediate `let adjusted` statement removed by the implementation's
final simplification. This is a new, exact, software-owned cause and permits a
fresh plan after a regression-backed fix; attempt-001 will never be retried or
reused.

The active lesson inputs still exceed the deterministic loading budget and
their hashes match the 2026-08-03 audit baseline, so no new audit is triggered.
Complete safety, authorization, evidence, privacy, retry, redaction,
real-process, ESP-IDF, and host-policy lesson blocks informed this plan. The
previously disclosed unrelated caption/VTT, small-table deduplication, legacy
GSD separator, and manual-removal blocks remain unloaded. Repo-local guidance,
the Bright Builds sidecar, standards overrides, and the architecture,
code-shape, verification, testing, Rust, and TypeScript standards remain in
force.

## Scope and non-scope

Replace the stale THR-001 source fragment with the actual production reducer
boundary and make checked-in source admission independently testable. Update
the closed evidence contract, validator, generated TypeScript binding,
protected paths, task binding, and tests to attempt ordinal 2. Preserve the
existing production `+5 C` reducer and firmware route; no device behavior is
changed by this plan.

After the corrected host path is clean, committed, pushed, and packaged, run
one fresh detector-gated `attempt-002`. It may factory-flash one exact clean
board-205 package, perform normal USB reset/re-enumeration, derive one origin
only from that capture's protected serial evidence, and issue read-only
same-origin HTTP/WebSocket/retained-log requests. If the initial flash effect
occurs and safe recovery cannot otherwise be proved, the existing transaction
may perform at most one exact-package recovery flash.

No settings mutation, restart request, mining, pool credential, ASIC work,
voltage, frequency, fan, power, raw I2C/GPIO, OTA, erase, fault injection,
physical manipulation, direct UART, pin, pad, header, probe, jumper, solder,
or injected signal is in scope. Attempt-001 artifacts remain sealed and are
not inputs to attempt-002 or public evidence.

NeverPersistRaw values remain memory-only. Hostnames, origins, ports, USB and
network identities, settings, HTTP bodies, acquisition stamps, boot sessions,
raw temperatures, logs, commands, PIDs, and traces remain mode `0600` below an
ignored mode `0700` parent. Public evidence is limited to closed schemas,
commits, opaque artifact digests, fixed public reference constants, counts,
categories, and booleans.

## Implementation

- [ ] Replace the stale source-fragment admission with the actual production
      reducer boundary and expose the smallest test seam needed to validate
      checked-in source semantics without weakening the runtime quorum.
- [ ] Add a regression that fails against the exact stale attempt-001 fragment
      and passes against current checked-in production source; retain the
      existing real-child, privacy, typed-failure, and withholding coverage.
- [ ] Advance the closed evidence contract, Rust validator, generated binding,
      protected paths, command tests, and task contract to attempt ordinal 2.
- [ ] Commit and push the complete software fix, build and admit an exact clean
      package, then run exactly one detector-gated read-only attempt-002.
- [ ] Publish and independently validate the THR-001 projection only if every
      source, package, thermal, safety, cleanup, and privacy member passes;
      otherwise preserve `implemented`, the earliest typed failure, recovery
      facts, evidence withholding, and the accepted terminal outcome.
- [ ] Transition only THR-001, synchronize progress, complete the result/task
      review, and archive the task atomically only after the full quorum passes.

## Verification and promotion

Before hardware, run focused source-admission, checked-in-source, thermal
reducer, firmware adapter, evidence-contract, validator, automation,
invocation, redaction, typed-failure, protected-file, and real-child tests;
build the real ESP32-S3 firmware and exact package; and run:

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

After the immutable plan/task checkpoint and complete implementation are both
clean, separately committed, and pushed, the only authorized hardware sequence
is:

1. `test ! -e scratch/thr001-emc2101/wrapper-002 && (umask 077; mkdir -m 700 -p scratch/thr001-emc2101/wrapper-002 && just detect-ultra205 > scratch/thr001-emc2101/wrapper-002/detector.stdout 2> scratch/thr001-emc2101/wrapper-002/detector.stderr)`
2. Only after command 1 admits exactly one Ultra 205 and the ignored local
   Wi-Fi credential file exists:
   `test ! -e scratch/thr001-emc2101/attempt-002 && test ! -e docs/parity/evidence/thr001-emc2101-thermal/thermal-projection.json && (umask 077; just capture-emc2101-thermal-evidence --private-root scratch/thr001-emc2101/attempt-002 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/thr001-emc2101/wrapper-002/detector.stdout --projection docs/parity/evidence/thr001-emc2101-thermal/thermal-projection.json --capture-timeout-seconds 360 > scratch/thr001-emc2101/wrapper-002/capture.stdout 2> scratch/thr001-emc2101/wrapper-002/capture.stderr)`

The wrapper root must be mode `0700`; detector and capture streams must be
distinct mode-`0600` files; the attempt-002 child and final projection must be
absent immediately before launch. The capture begins only from exact current
clean pushed HEAD. Starting command 2 consumes attempt-002. Preserve the
earliest typed failure. Non-ready hardware maps to `hardware_blocked`, malformed
or incomplete evidence to `evidence_invalid`, child timeout to `timeout`, and
launch failure to `process_failed`; recovery is secondary. No unchanged retry
is permitted.

Promote THR-001 only if the typed projection binds board 205, attempt ordinal
2, exact clean source/reference/package identity, EMC2101 address `0x4c`,
register `0x00`, board offset `+5 C`, the production read-only acquisition and
observation path, one finite plausible below-throttle fresh sample, exact
HTTP/WebSocket value/state/stamp/boot/package correlation, detector admission,
stable boot, disabled mining and hardware control, cleanup, private modes,
independent validation, source cleanliness, and passed redaction. Any missing,
malformed, unsafe, incoherent, drifted, or privacy-invalid member withholds the
projection, keeps THR-001 `implemented`, creates a truthful `CLOSURE.md`, and
stops without retry.
