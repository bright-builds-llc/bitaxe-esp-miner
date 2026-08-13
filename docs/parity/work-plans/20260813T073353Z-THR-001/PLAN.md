# Parity work plan

- Run ID: `20260813T073353Z-THR-001`
- Parity row: `THR-001`
- Initial status: `implemented`
- Source commit: `4af352a38f828c2ba0c3b3fe3754d3d0cf5a2fad`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-thr001-emc2101-live-thermal`
- Continues plan: `docs/parity/work-plans/20260813T015631Z-THR-001/PLAN.md`

## Selection

The clean synchronized selector reports no open plan and ranks API-009 first.
That row is skipped because its sole attempt-008 closed at the explicit
`stop_authority_boundary`, its rendered checkpoint expired without a physical
observation, and attempt-009 is not authorized. THR-001 is next.

THR-001 already owns valid exact-package read-only EMC2101 evidence, but the
authoritative parity policy requires `hardware-regression` for this active
safety-control row. Its attempt-003 closure permits a distinct plan only after
the stimulus, expected fault, abort conditions, restore path, projections,
safe-state proof, privacy, and fresh ordinal are frozen.

Source audit confirms that deliberately heating the board is unnecessary and
would be less safe. The production operator-sensor owner already reduces typed
EMC2101 `InvalidSample` outcomes into a `thermal_reading_invalid` fault and
recovers on a later real successful acquisition. A one-shot private NVS
admission can therefore keep performing real EMC2101 reads, overlay exactly
five invalid temperature outcomes at the producer boundary, retain a closed
fault marker, and return automatically to real fresh readings. This exercises
the real device, firmware reducer, API projection, retained-log path, cleanup,
and restoration while mining and hardware control remain disabled. The claim
is explicitly an injected acquisition-fault regression, not physical overheat
or electrical sensor failure.

The active lesson hashes remain at the audited 2026-08-03 baseline with no new
audit trigger. Complete safety, authorization, evidence, retry, redaction,
real-process, ESP-IDF, device-session, and host-stall blocks are loaded; the
disclosed caption/VTT, small-table deduplication, legacy GSD separator, and
manual-removal blocks remain unrelated omissions. Repo-local task, hardware,
privacy, architecture, code-shape, verification, testing, Rust, and TypeScript
rules govern this plan.

## Scope and non-scope

Add a private `esp-thermal-fault-stimulus-intent-v1` admitted only for board
205, attempt ordinal 4, the exact package source/reference/app ELF, this
immutable plan digest, the closed `emc2101_invalid_sample` stimulus, and exactly
five injected samples. The intent must be a mode-`0600` regular non-symlink
inside the fresh mode-`0700` attempt root. The flash tool must reject missing,
malformed, misplaced, wrong-mode, wrong-package, wrong-plan, wrong-board,
wrong-ordinal, or non-exact sample-count intents before USB effects.

Extend the private NVS seed mode to add only the one-shot stimulus kind, a
nonzero lease, and the fixed sample count while preserving ordinary Wi-Fi,
all board defaults, and `mineonboot=0`. Firmware must load, validate, erase, and
commit the complete tuple before returning it; reboot cannot replay it. Normal
boots and all existing Wi-Fi/campaign modes remain unchanged.

Add a pure bounded stimulus state machine to the sole operator-sensor owner.
It must require a real successful EMC2101 baseline, continue performing a real
successful EMC2101 read for every injected sample, replace only the temperature
outcome with `InvalidSample` for exactly five one-second sweeps, prove the
producer published `fault/thermal_reading_invalid`, then restore the actual
successful reading and prove a fresh recovery. It fails closed if baseline,
real reads, fault projection, exact count, recovery, or deadline differs. Emit
only ordered redaction-safe retained markers for `baseline_ready`,
`fault_observed`, and `recovered`; no temperature, lease, stamp, identity, path,
or raw trace may enter them.

Add one repo-owned `capture-emc2101-thermal-fault-evidence` transaction and the
independently validated public
`bitaxe-emc2101-thermal-fault-evidence-v1` projection. It must run one exact
fault-seeded flash-monitor, validate the closed marker sequence privately, then
always restore the same exact package with an ordinary Wi-Fi NVS seed and
confirm fresh same-package HTTP/WebSocket thermal truth, disabled mining and
hardware control, safe boot, cleanup, modes, redaction, and no stimulus replay.
Preserve the earliest typed failure through restoration.

No physical heating, lowered fan duty, fan stop, voltage/frequency/power change,
mining, pool input, ASIC work, raw I2C/GPIO command, public diagnostic setter,
long-lived test mode, arbitrary NVS mutation, erase, OTA, rollback, power
cycle, direct UART, pin/pad/header work, probes, jumpers, soldering, injected
electrical signal, non-205 device, or claim of physical overheat/open/short
fault is in scope. Prior attempts and their protected artifacts are not runtime
inputs.

## Implementation

- [ ] Add strict private intent parsing, package/plan/mode admission, the
      one-shot NVS tuple, consume-before-use firmware loading, and normal-mode
      non-regressions.
- [ ] Add the bounded pure stimulus state machine, production-owner integration,
      retained marker sequence, real-read preservation, exact fault projection,
      recovery, timeout, abort, and replay tests.
- [ ] Add the private-first host transaction, v1 contract and independent
      validator, real-child/process tests, restoration/failure precedence,
      protected-layout checks, atomic publication, and sensitive-output tests.
- [ ] Run every focused and mandatory gate; simplify and review the complete
      diff; commit and push the implementation before package or detector use.
- [ ] Run exactly one fresh detector-gated attempt-004 and publish only if the
      full injected-fault, recovery, restoration, identity, safety, cleanup,
      mode, and redaction quorum passes.
- [ ] Promote only THR-001 with `hardware-regression`, synchronize progress,
      complete the result/task review, and archive the task atomically on a
      complete quorum; otherwise keep `implemented` and stop without retry.

## Verification and promotion

Focused tests must cover strict intent fields and modes; package/plan/app-ELF
binding; ordinary/network/campaign NVS non-regression; exact one-shot tuple
consumption; malformed and replay rejection; baseline wait; five real-read-
backed injected samples; actual-read failure; fault state/reason; exact marker
order; fresh recovery; bounded deadline; normal boot; post-fault ordinary
restore; fake and real children; missing/malformed artifacts; primary failure
precedence; public withholding; and absence of sensitive values. Build the real
firmware and exact package, then run:

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

Also require generated contracts, source ownership, immutable-plan digest,
unique task binding, selector closure, reference cleanliness, exact package,
fresh-root/projection absence, mode and no-symlink checks, sensitive-output
review, `git diff --check`, and full diff review. If the full Bazel suite again
leaves macOS at the confirmed `EAGAIN` spawn-pressure boundary, cleanly stop the
idle Bazel server before rerunning only parity and downstream read-only gates.

After the complete implementation is separately committed, pushed, clean, and
package-admitted, the only hardware sequence is:

1. Prove `scratch/thr001-emc2101-fault/wrapper-004`,
   `scratch/thr001-emc2101-fault/attempt-004`, and the public projection are
   absent; create only the private wrapper/detector streams.
2. Run `just detect-ultra205` once and continue only if exactly one board-205
   ESP32-S3 is admitted and no holder or cleanup blocker exists.
3. Invoke exactly once:
   `just capture-emc2101-thermal-fault-evidence --private-root scratch/thr001-emc2101-fault/attempt-004 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/thr001-emc2101-fault/wrapper-004/detector.stdout --projection docs/parity/evidence/thr001-emc2101-thermal/thermal-fault-projection.json --capture-timeout-seconds 120`.

Starting command 3 consumes attempt-004. Allowed effects are one exact-package
USB flash/reset with the private one-shot NVS stimulus, five seconds of typed
temperature-outcome overlay while real EMC2101 reads continue, ordinary exact-
package restoration, normal USB reset/re-enumeration, same-origin read-only
HTTP/WebSocket/log capture, and cleanup. The workflow must restore even after a
post-flash primary failure. Non-ready device results map to `hardware_blocked`,
malformed/incomplete/invalid evidence to `evidence_invalid`, child timeout to
`timeout`, and launch failure to `process_failed`; recovery remains secondary.

Promote only if the v1 projection binds board 205, attempt ordinal 4, exact
clean source/reference/package/app ELF and plan, detector admission, consumed
one-shot intent, real healthy sensor reads before/during/after stimulus, exactly
five injected invalid temperature outcomes, ordered fault/recovery markers,
`fault/thermal_reading_invalid`, final fresh safe below-throttle HTTP/WebSocket
truth, no replay after ordinary restoration, disabled mining and hardware
control throughout, cleanup, protected modes, independent validation,
redaction, and the existing exact-package read-only projection. Change only
THR-001 to `verified` with `unit,workflow,hardware-smoke,hardware-regression`.
Any missing or unsafe member withholds the new projection, preserves
`implemented`, creates a truthful closure, and stops without attempt-005.
