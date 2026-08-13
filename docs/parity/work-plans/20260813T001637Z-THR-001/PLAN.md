# Parity work plan

- Run ID: `20260813T001637Z-THR-001`
- Parity row: `THR-001`
- Initial status: `implemented`
- Source commit: `6bd4cacc85ea0d68da3b435732aa0bcc4da87c0a`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-thr001-emc2101-live-thermal`

## Selection

The synchronized selector reports no open plan and ranks `API-009` first.
That row is not actionable: its sealed attempt-007 closure selected
`stop_repeated_boundary` after the same authoritative
`network_correlation_failed` / `safety_prerequisites_stale` resume boundary
recurred following the targeted producer-wakeup fix. Attempt-008 is expressly
prohibited until a separate software-only diagnostic identifies and fixes a
new discriminating cause.

`THR-001` is the next candidate and the first actionable row. Its current
checklist note says no fresh EMC2101 reading exists, but the accepted API-002
private capture proves that the production Ultra 205 sensor path returned one
fresh safe chip-temperature sample coherently through HTTP and WebSocket.
Pre-promotion source review found a real remaining parity defect: upstream
board 205 enables the EMC2101 internal-temperature path with `temp_offset = 5`,
and `EMC2101_get_internal_temp` adds that offset. The Rust Ultra 205 adapter
currently decodes register `0x00` without applying the board offset. Existing
telemetry therefore cannot verify the corrected behavior and will not be
reused as final evidence.

The active lesson inputs exceed the deterministic loading budget, but their
SHA-256 values exactly match the 2026-08-03 audit baseline, so no new audit is
triggered. Complete safety, authorization, evidence, privacy, hardware-retry,
redaction, exact-boundary, real-process, ESP-IDF, and host-policy lesson blocks
informed this plan. The unrelated caption/VTT, small-table deduplication,
legacy GSD separator, and manual-removal blocks were not loaded. Repo-local
guidance, the Bright Builds sidecar, standards overrides, and the architecture,
code-shape, verification, testing, Rust, and TypeScript standards were reviewed.

## Scope and non-scope

Correct the production Ultra 205 EMC2101 internal-temperature conversion by
applying the pinned board-205 `+5 C` offset in a pure validated reducer before
the observation enters the firmware store. Add one typed
`bitaxe-emc2101-thermal-evidence-v1` contract, independent Rust validator,
generated TypeScript binding, and one repo-owned
`capture-emc2101-thermal-evidence` orchestration command. Reuse the existing
system-info capture transaction for exact-package flash, private serial origin
admission, same-origin HTTP/WebSocket reads, recovery, cleanup, and base
evidence validation; then independently validate the fresh correlated
temperature sample and publish the row-specific projection atomically.

The sole hardware attempt may factory-flash one exact clean board-205 package,
perform the normal USB reset/re-enumeration, derive one origin only from that
capture's protected serial evidence, and issue read-only same-origin
HTTP/WebSocket/retained-log requests. If the initial flash effect occurs and
safe recovery cannot otherwise be proved, the existing capture transaction may
perform at most one exact-package recovery flash. No settings mutation,
restart request, mining, pool credential, ASIC work, voltage, frequency, fan,
power, raw I2C/GPIO, OTA, erase, fault injection, physical manipulation,
direct UART, pin, pad, header, probe, jumper, solder, or injected signal is in
scope.

NeverPersistRaw values remain memory-only. Hostnames, origins, ports, USB and
network identities, settings, HTTP bodies, acquisition stamps, boot sessions,
raw temperatures, logs, commands, PIDs, and traces remain mode `0600` below an
ignored mode `0700` parent. Public evidence may contain only closed schemas,
commits, opaque artifact digests, fixed public reference constants, counts,
categories, and booleans. This row will not claim absolute sensor calibration,
thermal offset configurability on other boards, thermal/fan actuation,
overheat or fault stimulus, cool/restart behavior, fan RPM, long-duration
drift, another sensor family, or another board.

## Implementation

- [ ] Add and unit-test the pure validated Ultra 205 `+5 C` temperature-offset
      reduction, then route the production EMC2101 internal-register read
      through it without changing fan or other-board behavior.
- [ ] Add the closed Rust evidence contract, independent validator, generated
      binding, composite capture command, human command surface, redaction
      admission, and behavior-focused regressions across real child processes
      and protected files.
- [ ] Commit and push the complete software implementation, build and admit an
      exact clean package, then run exactly one detector-gated read-only
      `attempt-001` capture with the protected paths and command below.
- [ ] Publish and independently validate one redacted THR-001 projection only
      if every source, package, thermal, safety, cleanup, and privacy member
      passes; otherwise preserve `implemented`, the earliest typed failure,
      recovery facts, evidence withholding, and the accepted terminal outcome.
- [ ] Transition only THR-001, synchronize progress, complete the result/task
      review, and archive the task atomically only after the full quorum passes.

## Verification and promotion

Before hardware, run focused thermal reducer, firmware adapter, source-owner,
evidence-contract, automation, invocation, redaction, validator-boundary, and
real-child tests; build the real ESP32-S3 firmware and exact package; and run
the mandatory sequence in order:

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
clean, committed, and pushed, the only authorized hardware sequence is:

1. `test ! -e scratch/thr001-emc2101/wrapper-001 && (umask 077; mkdir -m 700 -p scratch/thr001-emc2101/wrapper-001 && just detect-ultra205 > scratch/thr001-emc2101/wrapper-001/detector.stdout 2> scratch/thr001-emc2101/wrapper-001/detector.stderr)`
2. Only after command 1 admits exactly one Ultra 205 and the local ignored
   Wi-Fi credential file exists:
   `test ! -e scratch/thr001-emc2101/attempt-001 && test ! -e docs/parity/evidence/thr001-emc2101-thermal/thermal-projection.json && (umask 077; just capture-emc2101-thermal-evidence --private-root scratch/thr001-emc2101/attempt-001 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/thr001-emc2101/wrapper-001/detector.stdout --projection docs/parity/evidence/thr001-emc2101-thermal/thermal-projection.json --capture-timeout-seconds 360 > scratch/thr001-emc2101/wrapper-001/capture.stdout 2> scratch/thr001-emc2101/wrapper-001/capture.stderr)`

The wrapper root must be mode `0700`; detector and capture streams must be
distinct mode-`0600` files; the supervisor-owned attempt child and final
projection must be absent immediately before launch. The capture begins only
from exact current clean pushed HEAD and consumes attempt-001. Preserve the
earliest typed failure. Non-ready hardware maps to `hardware_blocked`,
malformed or incomplete evidence to `evidence_invalid`, child timeout to
`timeout`, and launch failure to `process_failed`; recovery is secondary. No
unchanged retry is permitted. Success selects `complete`; a hardware boundary
selects `stop_hardware_blocker`; an authority or impossible-proof boundary
selects its matching closed outcome.

Promote THR-001 only if the typed projection binds board 205, exact clean
source/reference/package identity, the pinned EMC2101 address `0x4c`, internal
temperature register `0x00`, board offset `+5 C`, the production read-only
acquisition and observation path, one finite plausible and below-throttle
fresh sample, exact HTTP/WebSocket value/state/stamp/boot/package correlation,
detector admission, stable boot, disabled mining and hardware control, cleanup,
private modes, independent validation, source cleanliness, and passed
redaction. Any missing, malformed, unsafe, incoherent, drifted, or
privacy-invalid member withholds the projection, keeps THR-001 at
`implemented`, creates a truthful CLOSURE.md, and stops without retry.
