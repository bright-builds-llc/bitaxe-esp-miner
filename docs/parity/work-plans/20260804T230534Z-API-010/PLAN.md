# Parity work plan

- Run ID: `20260804T230534Z-API-010`
- Parity row: `API-010`
- Initial status: `implemented`
- Source commit: `bfcd42343666cf8f347df7888bc7085859a93980`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api010-live-theme-durability-attempt-008`

## Selection

The clean synchronized `main` branch and deterministic selector resume only
the `API-010` lineage. Attempt-007 passed detector admission and completed the
exact-package flash, then withheld evidence as `evidence_invalid` because its
terminal baseline closed `runtime_origin_missing`.

This plan directly continues
`docs/parity/work-plans/20260804T224128Z-API-010/PLAN.md`.

Protected trace reduction proves 51 distinct panic-reset boot sessions, 52
stack-overflow markers, and no runtime-origin or Wi-Fi-state marker. Startup
ordering reaches the rendered operator display immediately before every
overflow. Exact-package ELF disassembly identifies the deterministic budget
violation: the operator-sensor task has an 8 KiB stack and a 2 KiB entry frame,
while its startup screen collector reaches a 7,872-byte full API-snapshot frame
through `collect_operator_snapshot_candidate` and `complete_api_snapshot`.
The prior observer-stack hypothesis targeted the wrong thread.

No protected device, port, USB/network/process identity, credential, origin,
theme value, or raw trace is copied into Git. Attempt-007 is pushed at
`bfcd42343666cf8f347df7888bc7085859a93980`; no other row is eligible while
this open plan lineage remains.

## Scope and non-scope

Replace the physical-screen collector's full API-snapshot dependency with a
narrow read-only projection that obtains only the facts represented by
`ScreenSnapshot`. Project command-visible mining/display values atomically to
small owned primitives; collect runtime-health, safe telemetry, Wi-Fi,
catalog/build identity, and the selected pool-host value without constructing
`ApiSnapshot`, `OperatorSnapshotCandidate`, or `RuntimeTelemetryProjection` in
the screen task.

Preserve screen behavior, cadence, privacy, state ownership, operator-snapshot
publication, HTTP/WebSocket behavior, retained evidence, statistics, and the
8 KiB operator-sensor task budget. Do not solve this by increasing a stack
constant. Remove the now-unjustified 16 KiB boot-observer change and its
size-only regression, restoring the observer's original 8 KiB budget after the
new evidence proved it was unrelated.

Add a red/green source regression that fails while the screen collector calls
the full API path and passes only with the narrow projection. Add a pure parser
for Xtensa disassembly and integrate a canonical firmware-build audit requiring
the demangled `operator_sensor_runtime::run` and
`runtime_snapshot::screen::collect_screen_snapshot` entry frames to be present,
individually bounded, and jointly no larger than 4 KiB. Missing, malformed,
duplicated, or oversized symbols fail the build.

After the software fix is clean, verified, committed, and pushed, authorize
exactly one attempt-008 package, protected detector, and conditional theme
durability capture. No manual electrical action is required. The capture may
perform one exact-package flash-monitor transaction, read the original theme,
POST one generated non-secret alternate theme, confirm immediate readback,
request one normal software restart through the typed device session, prove
same-device exact-build ordinal `N+1`, confirm persistence, restore the
original theme, and use at most one built-in exact-package recovery flash.

The supervisor-owned child is
`scratch/api010-theme-durability/attempt-008`, absent before launch and mode
`0700`, with only mode-`0600` private artifacts. The caller-owned sibling is
`scratch/api010-theme-durability/wrapper-008`, mode `0700`, with mode-`0600`
detector/stdout/stderr files. The only eligible public artifact is
`docs/parity/evidence/api010-theme-durability/theme-durability-projection.json`
after complete success and semantic redaction.

Do not discover network origins; read or expose credentials; change Wi-Fi or
pool configuration; mine; enable ASIC work; change voltage, frequency, fan,
thermal, or power controls; exercise display input; perform OTA, erase, fault
injection, or raw partition writes; terminate foreign processes; use direct
UART, pins, pads, headers, GPIO, probes, jumpers, soldering, or injected
signals; or run a second detector or capture.

## Implementation

- [ ] Commit and push this immutable plan/task checkpoint before editing
      firmware, automation, or build files.
- [ ] Turn the existing screen source-ownership assertion red against the full
      API path, then implement the narrow screen-only projection and remove the
      unrelated observer-stack workaround.
- [ ] Add and integrate the strict ELF entry-frame audit with focused parser,
      missing-symbol, duplicate-symbol, malformed-frame, and oversize tests.
- [ ] Build the canonical firmware and confirm the source guard plus binary
      audit prove the screen task no longer reaches the oversized frame.
- [ ] Run the complete ordered repository gate, commit and push the clean fix,
      then run exactly one detector-gated attempt-008 capture.
- [ ] Create `RESULT.md` and transition only `API-010` to `verified` on
      complete evidence; otherwise withhold evidence, keep `implemented`, and
      stop.

## Verification and promotion

Before hardware, run focused display-source and automation tests, canonical
firmware build/package, then:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`
9. semantic redaction, pinned-reference cleanliness, immutable-plan,
   sensitive-output, private-mode, selector, and diff checks

Commit and push the software fix. The only authorized effectful sequence is:

1. `just package`
2. `test ! -e scratch/api010-theme-durability/wrapper-008 && (umask 077; mkdir -m 700 scratch/api010-theme-durability/wrapper-008 && just detect-ultra205 > scratch/api010-theme-durability/wrapper-008/detector.stdout 2>&1)`
3. Only after command 2 succeeds:
   `test ! -e scratch/api010-theme-durability/attempt-008 && test ! -e docs/parity/evidence/api010-theme-durability/theme-durability-projection.json && (umask 077; just verify-theme-durability --private-root scratch/api010-theme-durability/attempt-008 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/api010-theme-durability/wrapper-008/detector.stdout --projection docs/parity/evidence/api010-theme-durability/theme-durability-projection.json --capture-timeout-seconds 360 > scratch/api010-theme-durability/wrapper-008/verify.stdout 2> scratch/api010-theme-durability/wrapper-008/verify.stderr)`

Any detector failure consumes attempt-008 and forbids capture. Capture launch
failure, timeout, malformed or missing projection, panic loop, non-ready
session, persistence mismatch, restoration uncertainty, cleanup/privacy
failure, or safety-invariant violation ends without retry and preserves the
earliest typed category.

Promotion requires `bitaxe-theme-durability-evidence-v1` to bind the exact
clean package and reference, one admitted board 205, a ready session for the
same physical device, one software restart, exact build recovery, changed boot
session, ordinal `N+1`, persisted theme equality, exact restoration, disabled
mining and hardware control, complete cleanup, and passed redaction. Otherwise
withhold evidence and `RESULT.md`, keep `API-010` at `implemented`, and stop.
