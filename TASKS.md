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

### task-ultra205-job-transition-soak | 2026-07-31 | Prove a 30-minute new-block transition

- [x] Add a closed `job-transition` campaign at the conservative Ultra 205
      profile with an exact 1,800-active-second lease and 1,980-second host
      observation budget.
- [x] Replace cumulative campaign serial capture with bounded chunk-fed
      analysis and aggregate observations that cannot retain raw serial.
- [x] Prove a changed previous-block notify invalidates old work, advances the
      generation, dispatches replacement work, and correlates a replacement
      result without submitting stale work.
- [x] Run `attempt-001` once from clean pushed commit `e732ca4b`; preserve its
      fail-closed result and do not open the conditional retry gate.
- [x] Land and verify the two regression fixes exposed by `attempt-001`:
      incremental typed runtime-attestation classification and in-flight
      transition lineage across same-block clean generation refreshes.
- [x] Run exactly one newly authorized post-fix `attempt-002` from the clean,
      pushed tracker-amendment HEAD; never run `attempt-003` under this task.
- [x] Seal one full-duration hardware attempt with continuous fresh safety,
      trusted identity, safe stop, lease cleanup, `mineonboot=false`, and USB
      cleanup using the newly authorized post-fix ordinal below.

Dependencies: Complete archived `task-ultra205-accepted-pool-share` with its
accepted clean-HEAD owner-pool share and confirmed safe stop.

Hardware contract:

- Permitted repo-owned commands:
  1. `just package`
  2. `just detect-ultra205`
  3. `just mining-campaign stage=job-transition profile=conservative board=205 port=<detector-port> manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json pool-credentials=<single-ignored-local-pool-file> evidence-dir=scratch/ultra205-job-transition-soak/attempt-001 duration-seconds=1800 redact-evidence=true`
  4. After the 2026-07-31 post-fix authorization, the same command once with
     `evidence-dir=scratch/ultra205-job-transition-soak/attempt-002`; this is
     the final attempt authorized by this task.
- Objective: prove at least one in-session Bitcoin previous-block transition
  from a clean pool notify through old-generation invalidation, replacement
  BM1366 dispatch, and a correlated replacement-generation nonce while mining
  for the full 1,800 active seconds.
- Evidence: each ignored attempt root is mode 0700 with mode-0600
  `ProtectedOperational` artifacts. Persist only closed states, counts,
  bounded durations, safe provenance, and digests. Never persist raw serial,
  block hashes, job IDs, pool messages, submit payloads, targets, difficulty,
  credentials, endpoints, workers, owner addresses, device identifiers,
  network values, tokens, NVS secrets, or secret-derived hashes. Evidence is
  private, redacted, sealed, and never automatically promoted.
- Preconditions: deterministic regressions fail before and pass after the
  implementation; all required software gates pass; changes are committed and
  pushed; the exact package is rebuilt from clean HEAD; exactly one board 205
  is admitted; and ignored Wi-Fi plus exactly one ignored pool input exist
  without their contents being printed or retained.
- Allowed effects: private NVS injection of Wi-Fi and owner pool settings,
  persistence of `mineonboot=false`, one conservative 400 MHz / 1100 mV /
  100% fan campaign lease, exact package flash, repo-owned USB reset and
  re-enumeration, BM1366 initialization/work/result traffic, Stratum V1 pool
  traffic and locally qualified submissions, bounded public Bitcoin-tip reads
  for the conditional retry gate, and device-local safe stop.
- Safety and stop limits: all five supported Ultra 205 safety observations
  must remain fresh; input must remain 4.5-5.5 V; power must not exceed 15 W;
  ASIC temperature must remain below 75 C; fan RPM must remain fresh and
  nonzero after the 100% command. Any safety, watchdog, transport, parser,
  protocol-consistency, generation, dispatch, correlation, actuation, lease,
  evidence, or cleanup fault blocks submissions and begins safe stop.
- New-block acceptance: require at least one `clean_jobs=true` notify with a
  changed previous-block value, matching new-block generation invalidation,
  replacement dispatch, and a correlated result under that replacement
  generation. Require no active-marker gap greater than 5,000 ms, zero rejected
  shares, and zero stale-generation submissions. An accepted share is optional;
  a valid below-target replacement nonce satisfies result correlation.
- Prohibited effects: mining beyond 1,800 active seconds, upstream-default
  actuation, TLS, Stratum V2, automatic fan mode, non-205 hardware,
  erase-flash, arbitrary raw writes, OTA, recovery upload, local network
  discovery, foreign-process termination, raw secret output, raw serial
  persistence, parity promotion, direct UART, pins, pads, headers, GPIO,
  probes, jumpers, soldering, injected signals, stress, or fault injection.
- Recovery/restoration: preserve the earliest typed failure; block and
  invalidate submissions; close owned pool transports; frequency-down and
  reset the ASIC; set core voltage and ASIC enable off; keep fan at 100% until
  fresh temperature is at or below 45 C, then set 30%; clear the lease;
  persist `mineonboot=false`; retain pool settings; and release USB/process
  resources. If safe stop cannot be confirmed, one exact baseline reflash is
  allowed only after same-device re-admission; otherwise stop.
- Retry bound: `attempt-001` sealed `job_transition_evidence_incomplete` and
  did not open its original conditional retry gate. The user subsequently
  authorized exactly one post-fix `attempt-002` after the two boundary defects
  were reproduced, fixed, and fully software-verified in pushed commit
  `5d530464`. Rebuild from the clean pushed tracker-amendment HEAD, re-detect
  exactly one board, and run that ordinal once. Any safety, identity, parser,
  protocol, generation, dispatch, correlation, rejection, transport, evidence,
  lease, safe-stop, or cleanup failure stops without retry. If no transition is
  observed, stop inconclusive without a public-tip wait. Never run
  `attempt-003` under this task.
- Accepted terminal outcomes: `complete` only for full-duration
  `job_transition_complete` plus every identity, safety, transition, rejection,
  safe-stop, seal, mode, and cleanup requirement. The only non-failure
  conditional outcome is `job_transition_not_observed`; every other category
  stops without retry and returns to diagnosis.

Verification: Software gates passed on 2026-07-31 before hardware: the exact
Rust pre-commit sequence, focused Stratum/campaign/device-session regressions,
`just verify-production-session`, `just test`, `just package`, Bright Builds
checks, parity, reference cleanliness, and redaction. `attempt-001` then ran
for 1,800,133 active ms and sealed `job_transition_evidence_incomplete` with
five previous-block changes, five matching generation advances, five
replacement dispatches, zero credited post-transition results, zero rejected
shares, zero stale-generation submissions, zero reconnects, a 519 ms maximum
active-marker gap, fresh required safety, `mineonboot=false`, confirmed safe
stop, and USB cleanup ready. Its private artifacts are mode 0600 under a mode
0700 ignored root and their result-bound digests verify. The conditional retry
gate did not open. Red regressions reproduced two host-accounting defects:
partial retention of the final runtime attestation at the old text-byte cap,
and loss of an in-flight transition lineage after a same-block clean generation
refresh. Both red regressions pass after the fixes, as do the exact Rust
sequence, focused API/Stratum/campaign suites, production-session verification,
all 82 Bazel test targets, package build, Bright Builds checks, parity,
reference cleanliness, redaction, artifact mode/seal/digest checks, and the
private evidence denylist. The newly authorized, no-retry post-fix
`attempt-002` then ran from clean pushed commit `cea568dd` for 1,800,120 active
ms and safely sealed `job_transition_evidence_incomplete`. It observed one
changed-previous-block notify, one matching new-block generation, and 761
replacement-work dispatches, but zero post-transition correlated results and
therefore zero completed transitions. The run otherwise recorded 90
below-pool-target results, zero rejected shares, zero stale-generation results
or submissions, zero reconnects, a 532 ms maximum active-marker gap, trusted
runtime identity and attestation, fresh required safety, `mineonboot=false`,
confirmed safe stop, and USB cleanup ready. All four artifacts have the
required owner-only modes, both private-artifact digests and the result seal
verify, and the private evidence denylist passes. One trailing partial serial
candidate was conservatively classified as `marker_truncated`; the accepted
terminal marker and trusted attestation remained intact, so this was not the
campaign failure. The authorized retry budget is exhausted and `attempt-003`
is prohibited.

Completion review: Incomplete. Two bounded full-duration attempts safely
reached lease expiry and proved changed-block detection, generation advance,
and replacement dispatch, but neither proved a correlated result under the
replacement generation. The remaining blocker is the post-transition ASIC
result-correlation link; further hardware execution requires a new task and
fresh authorization after additional deterministic diagnosis. This task does
not prove new-block transition completion, profitability, upstream-default
stability, unbounded mining, automatic fan control, release readiness, or
parity promotion.

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
