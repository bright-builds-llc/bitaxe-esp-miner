# Parity work plan

- Run ID: `20260817T082220Z-STAT-001`
- Parity row: `STAT-001`
- Initial status: `implemented`
- Source commit: `4e9f32820716df85c73783ced779791c6cdd972c`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat001-hashrate-monitor`

## Selection

The worktree and pinned reference are clean, `main` equals `origin/main`, and
the selector reports no open plan. It orders `SELF-001`, `BAP-002`, then
`STAT-001`. SELF-001 remains blocked by the absent production-safe self-test
route and required hardware regression. BAP-002 remains blocked by unfinished
BAP-001 firmware UART/request/subscription ownership and no authorized live
accessory path.

STAT-001 is first actionable. Attempt-014 ended with `watchdog_unproved`, but
its v14/v8 accumulator combined the earliest failure with phase/wait fields
overwritten by later terminal samples. Pushed source
`c3b0dcb997d503a4b6751dc35288bd53abcea8c5` fixes that real evidence boundary,
latches the complete tuple, and adds closed `stable`, `uninitialized`,
`retry_exhausted`, and `history_poisoned` outcomes through v15/v9 evidence.
The attempt-014-shaped regression fails before and passes after the correction;
all focused, real-firmware, package, privacy, reference, file-length, and
mandatory gates pass. Attempt-015 is therefore a progress-backed observation
of a materially corrected boundary, not an unchanged retry.

The active lesson set exceeds its loading budget; all headings were inventoried
and the full global file plus complete safety, authorization, evidence, retry,
unit, and watchdog-relevant blocks were loaded. Lower-priority historical
GSD/USB blocks were disclosed as omitted. The current audit baseline is under
90 days old, fewer than ten lessons are new, and no append is proposed, so no
audit trigger is due. Repo-local hardware/privacy/progress policy and Bright
Builds verification rules govern this plan.

## Scope and non-scope

Advance only STAT-001. Rebind the private-first workflow, independent Rust
validator, generated TypeScript contract, task/plan admission, Bazel inputs,
protected roots, and real-child fixtures from consumed attempt-014 to fresh
attempt-015. Preserve campaign result v15, network v9, the full watchdog
failure/read-outcome/phase/wait vocabulary, earliest atomic tuple, priority 5,
compiled five-second timeout, coherent store, 18-source evaluator identity,
public projection path, seals, precedence, redaction, and value-free reporting.
Add no firmware, scheduling, timeout, mining, sensor, hashrate, or control
behavior change.

After the rebind and every software/package gate pass at a clean pushed commit,
one exact board-205 package may be factory-flashed/reset and privately seeded
from ignored Wi-Fi and pool inputs. Run only the repo-owned conservative
`live-share` profile at 400 MHz, 1,100 mV ASIC core setpoint, and 100% fan for
exactly 600 accumulated active seconds. Join protected current-session serial,
HTTP, and reconstructed WebSocket evidence; pause, safe-stop, release USB/
process ownership, seal results, and permit at most one supervisor-owned exact-
package recovery flash after a post-flash failure. No human action is required.

ASIC core 1,100 mV and independently measured INA260 input-bus volts are
distinct domains. Require fresh 4.5-5.5 V input, power <=15 W, ASIC temperature
<75 C, and fresh nonzero fan RPM after the 100% command. Credentials,
endpoints, identities, exact rates/sensors, bodies, logs, commands, PIDs, and
traces remain private mode-0600 beneath fresh ignored mode-0700 roots. Only an
independently validated aggregate projection may be public. No overclock,
arbitrary controls, automatic fan, unbounded mining, OTA, erase, raw write,
fault injection, physical power action, external UART/BAP, or electrical
pin/pad/header/GPIO/probe/jumper/solder/signal work is authorized.

## Implementation

- [ ] Rebind ordinal, roots, immutable plan/task admission, Rust validator,
      generated contract, Bazel inputs, and fixtures from attempt-014 to
      attempt-015 without changing production behavior or v15/v9 schemas.
- [ ] Preserve every watchdog/read-outcome/phase/wait label, earliest atomic
      tuple, coherent store, prior-ordinal rejection, 18-source identity,
      seal/category/precedence/mode/unit/redaction regressions.
- [ ] Run focused and mandatory software, firmware, privacy, reference,
      package, exact-source, generated-contract, immutable-plan, and diff gates;
      commit and push before detector, credential handoff, or hardware.
- [ ] Execute only the frozen detector and conditional attempt-015 commands;
      promote only on the complete independently validated quorum.

## Authorized live commands and recovery

After the exact implementation is clean, fully gated, committed, pushed,
rebuilt, and package-validated, run only:

1. `test ! -e scratch/stat001-hashrate-monitor/wrapper-015 && (umask 077; mkdir -m 700 -p scratch/stat001-hashrate-monitor/wrapper-015 && just detect-ultra205 > scratch/stat001-hashrate-monitor/wrapper-015/detector.stdout 2> scratch/stat001-hashrate-monitor/wrapper-015/detector.stderr)`
2. Only after command 1 exits zero, repo-owned output admits exactly one Ultra
   205 with successful board-info/cleanup/holder checks, both ignored inputs
   are nonempty without being read, and child/projection remain absent:
   `test ! -e scratch/stat001-hashrate-monitor/attempt-015 && test ! -e docs/parity/evidence/stat001-hashrate-monitor/hashrate-monitor-projection.json && test -s wifi-credentials.json && test -s pool-credentials.json && (umask 077; just capture-hashrate-monitor-evidence --private-root scratch/stat001-hashrate-monitor/attempt-015 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --pool-credentials pool-credentials.json --detector-output scratch/stat001-hashrate-monitor/wrapper-015/detector.stdout --projection docs/parity/evidence/stat001-hashrate-monitor/hashrate-monitor-projection.json --duration-seconds 600 --capture-timeout-seconds 1500 > scratch/stat001-hashrate-monitor/wrapper-015/capture.stdout 2> scratch/stat001-hashrate-monitor/wrapper-015/capture.stderr)`

The caller owns only wrapper-015 and its distinct mode-0600 streams; the
supervisor exclusively creates the absent attempt-015 child. Starting command
2 consumes attempt-015. Preserve the earliest category plus read outcome,
phase, and wait state through bounded safe stop, optional exact-package
recovery, sealing, and cleanup. Never retry attempt-015, reuse attempt-014, or
start attempt-016. Stop on nonzero preflight/detector/capture exit, ambiguous/
non-205 identity, source/reference/package drift, missing input, unsafe state,
incomplete quorum, failed safe stop/recovery/cleanup/seal/mode/privacy, or a
complete projection. Record exactly one closed outcome: `complete`,
`stop_repeated_boundary`, `stop_hardware_blocker`, `stop_authority_boundary`,
or `stop_impossible_contract`.

Any precise failure tuple—especially retry exhausted, history poisoned, or
genuine active-session uninitialized—must be recorded without inference and
receive a separate source diagnosis before another ordinal. Attempt-014's
older mixed tuple cannot establish recurrence.

## Verification and promotion

Run focused hashrate automation, Rust validator, campaign watchdog/read-
outcome/phase/wait/schema, coherent-store, generated-contract, real-child,
source/reference, seal, mode, unit, redaction, and precedence tests. Run `just
verify-redaction`, `just verify-reference`, `just package`, then in order:
`cargo fmt --all`; `cargo clippy --all-targets --all-features -- -D warnings`;
`cargo build --all-targets --all-features`; `cargo test --all-features`; `bun
scripts/bright-builds-check.ts all`; `just test`; `just parity`; `just
parity-progress`.

Promotion requires board205/attempt15/exact source/reference/package/plan and
detector identity; one ASIC/four domains; one-second cadence; all twenty active
windows with work renewal; changing coherent positive HTTP/WebSocket current
hashrate and positive warm rolling rates; bounded error; watchdog failure
`none` with stable read outcome; terminal zero rates; safe stop; cleanup;
protected modes; seals; independent validation; and redaction. On success,
create RESULT, commit evidence as SOURCE_COMMIT, transition only STAT-001 to
`verified` with `unit,workflow,api-compare,hardware-smoke,hardware-regression`,
sync progress, archive the completed task, final-gate, and push. Missing facts
withhold the projection, create CLOSURE, leave `implemented`, and stop.

## Non-claims

This plan does not pre-claim the read outcome, watchdog success, hashrate
accuracy, profitability, arbitrary scheduling/pools/profiles, other boards/
ASICs, update/recovery, or release readiness.
