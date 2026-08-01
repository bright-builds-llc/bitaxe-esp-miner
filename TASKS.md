# Tasks

This is the repository's sole active work tracker. Use one stable, timestamped
task block per unit of work. Update only that block as work progresses, record
the verification performed, and finish with a concise completion review.

`TASKS.archive.md` is the append-only historical store, not an active tracker.
After a task is completed, explicitly cancelled, or superseded, append its full
final record there and remove it from this file in the same commit. Keep
blocked, deferred, terminal-blocker, future, and otherwise unresolved tasks
here. Stable task IDs must be unique across both files. Never restore or select
an archived task; create a new active task with a new ID and an archived-task
reference for follow-up work.

Task blocks under `## Future — Explicit Only` remain incomplete but are
excluded from automatic task selection, including repeated top-task loops.
Only a current request that names the exact stable task ID makes one future
task a selection candidate; that opt-in does not bypass dependencies,
environment, authorization, verification, or safety gates. When only future
tasks remain, the automatic queue is exhausted, but the tracker is not fully
complete.

Historical plans, milestones, debug sessions, and task records under
`.planning/milestones/` are evidence and context only. They do not authorize
new work.

## Active

### task-ultra205-default-profile-soak | 2026-07-28 | Run the bounded upstream-default mining soak

- [ ] Start a fresh exact-package attempt at 485 MHz, 1200 mV, and 100% fan
      only after the conservative live-share task completes.
- [ ] Count 600 seconds from authorized active mining rather than boot or
      connection start.
- [ ] Require uninterrupted fresh safety truth, watchdog responsiveness,
      active work renewal, at least one new correlated nonce and pool response,
      and correlated HTTP/WebSocket state throughout the soak.
- [ ] Confirm the device-local lease expires, hardware safe-stop completes,
      the lease is cleared, `mineonboot=false` persists, pool settings remain,
      and the new firmware remains installed in paused state.
- [ ] Seal one private, redacted soak result without automatic parity
      promotion.

Dependencies: Complete `task-ultra205-live-pool-share` successfully.

Hardware contract:

- Permitted commands:
  1. `just detect-ultra205`
  2. `just package`
  3. `just mining-campaign stage=soak profile=upstream-default board=205 port=<detector-port> manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json pool-credentials=<single-ignored-local-pool-file> evidence-dir=scratch/ultra205-default-profile-soak/attempt-001 duration-seconds=600 redact-evidence=true`
- Objective: prove the exact package can mine for 600 active seconds at the
  Ultra 205 upstream-default profile with fresh safety, watchdog,
  work/result/share, HTTP, WebSocket, and final safe-stop evidence.
- Evidence: the ignored
  `scratch/ultra205-default-profile-soak/attempt-001` root is private,
  non-promoted `ProtectedOperational` evidence with mode-0700 parent and
  mode-0600 artifacts. Only `pool_config: local-owner-supplied`, closed
  categories, bounded counts/durations, and safe provenance may be summarized;
  the same secret and device-identifier denylist as the live-share task
  applies.
- Preconditions: the live-share task completed with one correlated submit
  response and confirmed safe-stop; all software gates pass against fresh
  current HEAD; the exact package is frozen; exactly one board 205 is admitted;
  and the ignored Wi-Fi and single pool credential inputs remain available
  without being printed.
- Allowed effects: private NVS credential injection, persistence of
  `mineonboot=false`, installation of one 600-active-second default-profile
  lease, exact package flash, repo-owned USB reset/re-enumeration, fan 100%,
  DS4432U 1200 mV, ASIC enable/reset, BM1366 initialization and work/result
  traffic, Stratum V1 TCP connection and submissions, fresh-session
  HTTP/WebSocket observation, and bounded device-local safe-stop.
- Safety and stop limits: the live-share limits remain unchanged: fresh
  observations, 4.5-5.5 V input, at most 15 W, ASIC temperature below 75 C,
  and fresh nonzero fan RPM after the 100% command. Any safety, watchdog,
  transport, generation, actuation, lease, telemetry-correlation, or evidence
  fault blocks work and begins safe-stop.
- Prohibited effects: TLS, Stratum V2, automatic fan mode, mining beyond the
  lease, non-205 hardware, erase-flash, arbitrary raw writes, OTA, recovery
  upload, network discovery, foreign-process termination, raw secret output,
  parity promotion, direct UART, pins, pads, headers, GPIO, probes, jumpers,
  soldering, injected signals, voltage/fan stress, or fault injection.
- Recovery/restoration: preserve the earliest typed failure; block and
  invalidate submissions; close owned pool transports; frequency-down and
  reset the ASIC; set core voltage off and ASIC enable off; keep fan at 100%
  until fresh temperature is at or below 45 C, then set 30%; clear the lease;
  persist `mineonboot=false`; retain pool settings; and release owned USB and
  process resources. If device-local stop cannot be confirmed, one
  predeclared exact-baseline reflash is allowed only after same-device
  re-admission; otherwise stop.
- Retry bound: one fresh attempt only and no unchanged retry. A later ordinal
  requires a targeted regression-backed fix or authorized non-invasive
  remediation with objective boundary-change proof; one post-fix recurrence
  selects `stop_repeated_boundary`.
- Accepted terminal outcomes: `complete` only when the full active-duration,
  correlation, and safe-stop criteria pass; otherwise
  `stop_repeated_boundary`, `stop_hardware_blocker`,
  `stop_authority_boundary`, or `stop_impossible_contract`.

Verification: Pending. Run all required software gates, the exact permitted
hardware commands, private-artifact permission checks, redaction and secret
denylist verification, active-duration/correlation checks, lease/safe-stop
validation, sealed result validation, and final diff review.

Completion review: Pending. This bounded soak does not authorize or verify
automatic fan control, unbounded mining, complete statistics/hashrate parity,
release readiness, or checklist promotion.

### task-ultra205-job-transition-poll-liveness | 2026-07-31 | Restore post-transition ASIC polling

- [x] Capture a deterministic red Production Mining Session regression proving
      that a stale old-generation poll completion suppresses replacement-
      generation polling after a clean changed-block notify.
- [x] Fix successful generation advances by invalidating only stale bridge
      orchestration state before replacement work is queued, without disarming
      the ASIC listener or allowing an old completion to mutate a newer poll.
- [x] Add typed, privacy-safe ASIC bridge and BM1366 parser diagnostics,
      version the campaign status/result/observation contracts, and seal a
      bounded private mining-diagnostics artifact.
- [x] Document behavioral parity with the pinned upstream independent job
      dispatch and continuous ASIC result tasks without copying GPL expression.
- [ ] Complete the required ordered software verification, commit and push the
      verified fix, then rebuild the exact clean-HEAD package.
- [ ] Run one authorized 1,800-active-second conservative real-pool attempt,
      with one conditional no-transition retry only under the gate below.

Dependencies: Supersedes archived `task-ultra205-job-transition-soak`, whose
two safe full-duration attempts proved changed-block detection, generation
advance, and replacement dispatch but exhausted their retry budget without a
post-transition correlated result.

Hardware contract:

- Permitted repo-owned commands:
  1. `just package`
  2. `just detect-ultra205`
  3. `just mining-campaign stage=job-transition profile=conservative board=205 port=<detector-port> manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json pool-credentials=<single-ignored-local-pool-file> evidence-dir=scratch/ultra205-job-transition-poll-liveness/attempt-001 duration-seconds=1800 redact-evidence=true`
  4. Only if attempt-001 seals `job_transition_not_observed` and every other
     gate passes: read the public Bitcoin tip height until it strictly advances,
     persist only `public_tip_advanced=true`, rebuild the same clean-HEAD
     package, re-detect the board, and run the same command once with
     `evidence-dir=scratch/ultra205-job-transition-poll-liveness/attempt-002`.
- Objective: prove at least one clean changed-block notify advances the work
  generation, invalidates the old bridge poll state, dispatches replacement
  BM1366 work, rearms and completes replacement-generation polling, and
  decodes and correlates at least one replacement-generation nonce.
- Evidence: each ignored attempt root is mode 0700 with mode-0600
  `ProtectedOperational` artifacts. Persist only closed states, monotonic
  counts/durations, bounded typed traces, safe provenance, and digests. Never
  persist raw serial or UART bytes, block hashes, job IDs, generations as raw
  values, nonces, pool messages, submit payloads, targets, difficulty,
  credentials, endpoints, workers, owner addresses, device identifiers,
  network values, tokens, NVS secrets, or secret-derived hashes. Evidence is
  private, redacted, sealed, and never automatically promoted.
- Preconditions: the red poll-starvation regression fails on the historical
  implementation and passes after the fix; all required software gates pass;
  changes are committed and pushed; the exact package is rebuilt from clean
  HEAD; exactly one Ultra 205 is admitted; and ignored Wi-Fi plus exactly one
  ignored pool input exist without their contents being printed or retained.
- Allowed effects: private NVS injection of Wi-Fi and owner pool settings,
  persistence of `mineonboot=false`, one conservative 400 MHz / 1100 mV /
  100% fan campaign lease per admitted attempt, exact package flash,
  repo-owned USB reset and re-enumeration, BM1366 initialization/work/result
  traffic, Stratum V1 pool traffic and locally qualified submissions, the
  bounded public tip-height retry gate, and device-local safe stop.
- Safety and stop limits: all five supported Ultra 205 safety observations
  must remain fresh; input must remain 4.5-5.5 V; power must not exceed 15 W;
  ASIC temperature must remain below 75 C; fan RPM must remain fresh and
  nonzero after the 100% command. Any safety, watchdog, transport, parser,
  protocol-consistency, generation, dispatch, correlation, actuation, lease,
  evidence, or cleanup fault blocks submissions and begins safe stop.
- Acceptance: require the full 1,800 active seconds, trusted exact-package
  identity, no active-marker gap above 5,000 ms, at least one clean changed-
  block notify and generation advance, replacement dispatch followed by a
  post-transition poll request and completion, at least one decoded and
  correlated replacement-generation nonce, zero rejected shares, stale
  submissions, reconnects, or unresolved in-flight poll at safe stop,
  `mineonboot=false`, confirmed safe stop, lease removal, sealed diagnostics,
  and USB cleanup ready.
- Prohibited effects: mining beyond the exact lease, upstream-default
  actuation, TLS, Stratum V2, automatic fan mode, non-205 hardware, erase-
  flash, arbitrary raw writes, OTA, recovery upload, network discovery,
  foreign-process termination, raw secret output, raw serial persistence,
  parity promotion, fresh upstream firmware flashing, direct UART, pins,
  pads, headers, GPIO, probes, jumpers, soldering, injected signals, stress,
  or fault injection.
- Recovery/restoration: preserve the earliest typed failure; block and
  invalidate submissions; close owned pool transports; frequency-down and
  reset the ASIC; set core voltage and ASIC enable off; keep fan at 100% until
  fresh temperature is at or below 45 C, then set 30%; clear the lease;
  persist `mineonboot=false`; retain pool settings; and release USB/process
  resources. If safe stop cannot be confirmed, one exact baseline reflash is
  allowed only after same-device re-admission; otherwise stop.
- Retry bound: attempt-002 is permitted only when attempt-001 seals
  `job_transition_not_observed` and every other gate passes, and only after a
  public tip-height advance. Parser, correlation, safety, transport, evidence,
  rejection, or `job_transition_evidence_incomplete` outcomes stop without
  retry. A second clean no-transition attempt ends inconclusive. No further
  ordinal is authorized.
- Accepted terminal outcomes: `complete` only for the full acceptance
  contract; `job_transition_not_observed` may open the single conditional
  retry; otherwise classify with the typed diagnostics and stop.

Verification: The pre-fix regression failed at
`stale_old_generation_poll_completion_rearms_replacement_generation_poll`
because no replacement-generation `PollAsic` effect followed the stale
old-generation completion. It passes after the bridge invalidation fix, as do
the companion stale-nonce, same-block clean, repeated-clean, parser subtype,
strict v8 marker, bounded first/last trace, v4/v3 evidence, private-artifact
digest, permission, and privacy tests. The original nine job-transition
regressions and all new liveness/diagnostic cases pass. The ordered Rust gates
(`cargo fmt --all`, warnings-denied Clippy, all-target and all-feature build,
and all-feature tests), focused ASIC/Stratum/flash suites,
`just verify-production-session`, all 82 `just test` Bazel targets,
`just package`, Bright Builds checks, parity, reference cleanliness, redaction,
artifact mode/seal/digest tests, privacy denylisting, `git diff --check`, and
final diff review all pass. Pending: commit/push this verified state, rebuild
the exact clean-HEAD package, and execute the authorized hardware validation.

Completion review: Pending. This task does not authorize or verify
profitability, upstream-default stability, unbounded mining, automatic fan
control, release readiness, direct electrical access, or parity promotion.

## Future — Explicit Only

### task-cross-platform-device-session-adapters | 2026-07-22 | Qualify Linux and Windows ESP device sessions

- [ ] Implement Linux physical/enumeration identity, exclusive ownership,
  receive-only observation, and bounded reacquisition behind the canonical
  device-session contract.
- [ ] Implement the corresponding Windows adapter without weakening
  exclusive ownership, request-once, or private-artifact guarantees.
- [ ] Add platform-native real-process tests.
- [ ] Keep unsupported platforms fail-closed until each exact adapter and its
  separately authorized hardware evidence qualify.

Verification: Pending.

Completion review: Pending. macOS remains the only production adapter. This
task does not itself authorize hardware, credentials, network discovery, direct
UART or pin work, evidence promotion, or push operations.

## Effectful Hardware Task Gate

Standing permission for safe USB interaction remains subject to `AGENTS.md` and
`docs/hardware/hardware-attempt-policy.md`. Before any effectful hardware run,
move or add one task block under `Active` that explicitly records:

- the exact permitted repo-owned command and objective;
- the evidence destination, privacy class, and redaction policy;
- recovery, restoration, and cleanup procedures;
- retry bounds, including the unchanged-boundary stop rule; and
- accepted terminal categories and stop conditions.

If any field is missing, hardware work is not authorized. A task entry never
expands the direct-UART, pin-manipulation, privacy, safety, or archived-lineage
boundaries in `AGENTS.md`.

## Accepted Debt and Constraints

- Milestone v1.2 is administratively closed with gaps and is not a release.
- Phase 36 stopped after 8 of 10 plans. Plans 36-07 and 36-04 did not complete.
- SYS-02, EVD-11, EVD-12, and EVD-14 remain blocked. EVD-15 is satisfied by
  exact preservation, typed demotion, and explicit non-claims.
- The sole final Phase 36 hardware attempt sealed `sealed_non_promotion`,
  produced no candidate, and left device restoration unresolved.
- Do not repeat the unchanged hardware attempt. A future attempt requires new
  diagnostic information, a targeted regression-backed fix or objectively
  verified non-invasive remediation, and a complete task-scoped hardware
  contract under the gate above.
- Administrative closure, software verification, or task completion alone is
  never hardware or parity evidence.
