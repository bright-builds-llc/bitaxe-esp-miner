# Parity work plan

- Run ID: `20260817T095432Z-STAT-001`
- Parity row: `STAT-001`
- Initial status: `implemented`
- Source commit: `17a2c263dff42833f7580afbbb68120c899ce09b`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat001-hashrate-monitor`

## Selection

The worktree and pinned reference are clean, `main` equals `origin/main`, and
the selector reports no open plan. It orders `SELF-001`, `BAP-002`, then
`STAT-001`. SELF-001 remains blocked by the absent production-safe self-test
route and required hardware regression. BAP-002 remains blocked by unfinished
BAP-001 firmware UART/request/subscription ownership and no authorized live
accessory path.

STAT-001 is first actionable. Attempt-015 ended at the trustworthy
`watchdog_feed_stale/stable/handling_inbox/not_waiting` tuple, but could not
distinguish inbox mapping, session evaluation, or effect execution. Pushed
source `177fffe9554944cf1c57b7e595735bb20ba6be84` fixes that real boundary:
one closed value-free subphase reaches coherent firmware/runtime-health and
private v16/v10 evidence, entry feeds remove inherited timeout age, and the
regression proves a genuinely blocking operation still becomes stale. All
focused, real-firmware, package, privacy, reference, file-length, and mandatory
gates pass. Attempt-016 is therefore a progress-backed observation of a
materially corrected and more discriminating boundary, not an unchanged retry.

The active lesson set exceeds its loading budget; all headings were inventoried
and the full global file plus complete safety, authorization, evidence, retry,
unit, legacy-unit, and watchdog-relevant blocks were loaded. Lower-priority
historical GSD and older USB-session blocks were disclosed as omitted. The
current audit baseline is under 90 days old, fewer than ten lessons are new,
and no append is proposed, so no audit trigger is due. Repo-local hardware,
privacy, progress, and ESP-IDF guidance plus Bright Builds architecture,
testing, code-shape, and verification rules govern this plan.

## Scope and non-scope

Advance only STAT-001. Rebind the private-first workflow, independent Rust
validator, generated TypeScript contract, task/plan admission, Bazel inputs,
protected roots, and real-child fixtures from consumed attempt-015 to fresh
attempt-016. Preserve campaign result v16, network v10, the full watchdog
failure/read-outcome/phase/subphase/wait vocabulary, earliest atomic tuple,
priority 5, compiled five-second timeout, coherent store, 18-source evaluator
identity, public projection path, seals, precedence, redaction, and value-free
reporting. Add no firmware, scheduling, timeout, mining, sensor, hashrate, or
control behavior change.

After the rebind and every software/package gate pass at a clean pushed commit,
one exact board-205 package may be factory-flashed/reset and privately seeded
from ignored Wi-Fi and pool inputs. Run only the repo-owned conservative
`live-share` profile at 400 MHz, 1,100 mV ASIC core setpoint, and 100% fan for
exactly 600 accumulated active seconds. Join protected current-session serial,
HTTP, and reconstructed WebSocket evidence; pause, safe-stop, release USB and
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
      generated contract, Bazel inputs, and fixtures from attempt-015 to
      attempt-016 without changing production behavior or v16/v10 schemas.
- [ ] Preserve every watchdog/read-outcome/phase/subphase/wait label, earliest
      atomic tuple, coherent store, prior-ordinal rejection, 18-source identity,
      seal/category/precedence/mode/unit/redaction regressions.
- [ ] Run focused and mandatory software, firmware, privacy, reference,
      package, exact-source, generated-contract, immutable-plan, and diff gates;
      commit and push before detector, credential handoff, or hardware.
- [ ] Execute only the frozen detector and conditional attempt-016 commands;
      promote only on the complete independently validated quorum.

## Authorized live commands and recovery

After the exact implementation is clean, fully gated, committed, pushed,
rebuilt, and package-validated, run only:

1. `test ! -e scratch/stat001-hashrate-monitor/wrapper-016 && (umask 077; mkdir -m 700 -p scratch/stat001-hashrate-monitor/wrapper-016 && just detect-ultra205 > scratch/stat001-hashrate-monitor/wrapper-016/detector.stdout 2> scratch/stat001-hashrate-monitor/wrapper-016/detector.stderr)`
2. Only after command 1 exits zero, repo-owned output admits exactly one Ultra
   205 with successful board-info/cleanup/holder checks, both ignored inputs
   are nonempty without being read, and child/projection remain absent:
   `test ! -e scratch/stat001-hashrate-monitor/attempt-016 && test ! -e docs/parity/evidence/stat001-hashrate-monitor/hashrate-monitor-projection.json && test -s wifi-credentials.json && test -s pool-credentials.json && (umask 077; just capture-hashrate-monitor-evidence --private-root scratch/stat001-hashrate-monitor/attempt-016 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --pool-credentials pool-credentials.json --detector-output scratch/stat001-hashrate-monitor/wrapper-016/detector.stdout --projection docs/parity/evidence/stat001-hashrate-monitor/hashrate-monitor-projection.json --duration-seconds 600 --capture-timeout-seconds 1500 > scratch/stat001-hashrate-monitor/wrapper-016/capture.stdout 2> scratch/stat001-hashrate-monitor/wrapper-016/capture.stderr)`

The caller owns only wrapper-016 and its distinct mode-0600 streams; the
supervisor exclusively creates the absent attempt-016 child. Starting command
2 consumes attempt-016. Preserve the earliest category plus read outcome,
phase, subphase, and wait state through bounded safe stop, optional exact-
package recovery, sealing, and cleanup. Never retry attempt-016, reuse attempt-
015, or start attempt-017. Stop on nonzero preflight/detector/capture exit,
ambiguous/non-205 identity, source/reference/package drift, missing input,
unsafe state, incomplete quorum, failed safe stop/recovery/cleanup/seal/mode/
privacy, or a complete projection. Record exactly one closed outcome:
`complete`, `stop_repeated_boundary`, `stop_hardware_blocker`,
`stop_authority_boundary`, or `stop_impossible_contract`.

Any precise failure tuple must be recorded without inference and receive a
separate source diagnosis before another ordinal. Recurrence of attempt-015's
exact coarse tuple is impossible to establish because it predates subphase;
recurrence requires equality across the new v16/v10 tuple.

## Verification and promotion

Run focused hashrate automation, Rust validator, campaign watchdog/read-
outcome/phase/subphase/wait/schema, coherent-store, generated-contract, real-
child, source/reference, seal, mode, unit, redaction, and precedence tests. Run
`just verify-redaction`, `just verify-reference`, `just package`, then in order:
`cargo fmt --all`; `cargo clippy --all-targets --all-features -- -D warnings`;
`cargo build --all-targets --all-features`; `cargo test --all-features`; `bun
scripts/bright-builds-check.ts all`; `just test`; `just parity`; `just
parity-progress`.

Promotion requires board205/attempt16/exact source/reference/package/plan and
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

This plan does not pre-claim the owner subphase, watchdog success, hashrate
accuracy, profitability, arbitrary scheduling/pools/profiles, other boards or
ASICs, update/recovery, or release readiness.
