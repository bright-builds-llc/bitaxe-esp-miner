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

### task-ultra205-accepted-pool-share | 2026-07-31 | Obtain one accepted owner-pool share

Status: In progress — the deterministic diagnosis and software fix are
verified. `attempt-001` then proved local below-target filtering but ended on
an unclassified early safe stop. `attempt-002` classified the stop as an ASIC
bridge failure. `attempt-003` crossed that boundary and isolated the remaining
failure to the poll side; code review found an untracked in-flight poll path
capable of filling the worker queue. `attempt-004` crossed that boundary and
isolated a silent synchronous hardware-preparation stall after the fan-full
step started. `attempt-005` crossed that boundary, sustained mining, and then
isolated a false-stale observation race caused by waking the owner before
releasing the observation store, requiring an ordering fix and post-fix
`attempt-006`.

- [x] Reproduce the rejected-share path deterministically from a known BM1366
      nonce, reconstructed header, and pool difficulty.
- [x] Prove the rejection cause against the read-only reference behavior and
      preserve only closed privacy-safe diagnostics.
- [x] Fix the production correlation/submission boundary so only a share that
      satisfies the active pool target can be submitted.
- [ ] Verify the exact clean-HEAD package, detect exactly one Ultra 205, and
      run one bounded conservative accepted-share attempt.
- [ ] Seal an accepted share, fresh safety, trusted identity, confirmed safe
      stop, lease cleanup, `mineonboot=false`, and USB cleanup without parity
      promotion or secret retention.

Dependencies: Complete archived task `task-ultra205-live-pool-share` and its
accepted `submit_response_observed` proof at clean commit `9861f4c4`.

Hardware contract:

- Permitted commands:
  1. `just detect-ultra205`
  2. `just package`
  3. `just mining-campaign stage=live-share profile=conservative board=205 port=<detector-port> manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json pool-credentials=<single-ignored-local-pool-file> evidence-dir=scratch/ultra205-accepted-pool-share/attempt-001 duration-seconds=600 redact-evidence=true`
  4. After the diagnostic change is committed/pushed and rebuilt from exact
     clean HEAD, the same command once with
     `evidence-dir=scratch/ultra205-accepted-pool-share/attempt-002`.
  5. After the ASIC result-loop fix is committed/pushed and rebuilt from exact
     clean HEAD, the same command once with
     `evidence-dir=scratch/ultra205-accepted-pool-share/attempt-003`.
  6. After the poll in-flight invariant and queue diagnostics are
     committed/pushed and rebuilt from exact clean HEAD, the same command once
     with `evidence-dir=scratch/ultra205-accepted-pool-share/attempt-004`.
  7. After the fan preparation path is proven non-blocking, incomplete
     preparation is classified by the host, and the fix is committed/pushed
     and rebuilt from exact clean HEAD, the same command once with
     `evidence-dir=scratch/ultra205-accepted-pool-share/attempt-005`.
  8. After observation publication is proven to release storage before waking
     the mining owner, the false-stale read is removed, and the fix is
     committed/pushed and rebuilt from exact clean HEAD, the same command once
     with `evidence-dir=scratch/ultra205-accepted-pool-share/attempt-006`.
- Objective: obtain one pool-accepted Stratum V1 share derived from current
  owner-pool work and a correlated BM1366 nonce, then prove safe stop.
- Evidence: the ignored
  `scratch/ultra205-accepted-pool-share/attempt-001` root is private,
  non-promoted `ProtectedOperational` evidence with mode-0700 parent and
  mode-0600 artifacts. Persist only closed categories, bounded counts and
  durations, safe provenance, and digests. Never persist or summarize raw
  serial, pool responses, submit payloads, targets, difficulty values,
  endpoints, ports, users, workers, owner addresses, passwords, Wi-Fi values,
  device paths, IPs, MACs, tokens, NVS secrets, or secret-derived hashes.
- Preconditions: the deterministic regression fails before and passes after
  the fix; all required software gates pass; the fix is committed and pushed;
  the package is rebuilt from clean exact HEAD; exactly one board 205 is
  admitted; and exactly one ignored local pool input plus ignored Wi-Fi input
  exists without being read into output or evidence.
- Allowed effects: private NVS injection of Wi-Fi and owner pool settings,
  persistence of `mineonboot=false`, one conservative campaign lease, exact
  package flash, repo-owned USB reset/re-enumeration, fan 100%, DS4432U 1100
  mV, ASIC enable/reset, BM1366 initialization and work/result traffic,
  Stratum V1 TCP connection and qualified-share submission, fresh-session
  HTTP/WebSocket observation, and bounded device-local safe stop.
- Safety and stop limits: observations must remain fresh; input must remain
  4.5-5.5 V; power must not exceed 15 W; ASIC temperature must remain below
  75 C; fan RPM must remain fresh and nonzero after the qualified 100% command.
  Any sensor, watchdog, actuation, validity, generation, lease, transport, or
  evidence fault blocks submission and begins safe stop immediately.
- Prohibited effects: submission of locally known below-target work, TLS,
  Stratum V2, automatic fan mode, mining beyond 600 seconds, non-205 hardware,
  erase-flash, arbitrary raw writes, OTA, recovery upload, network discovery,
  foreign-process termination, raw secret output, parity promotion, direct
  UART, pins, pads, headers, GPIO, probes, jumpers, soldering, injected
  signals, voltage/fan stress, or fault injection.
- Recovery/restoration: preserve the earliest typed failure; block and
  invalidate submissions; close owned pool transports; frequency-down and
  reset the ASIC; set core voltage off and ASIC enable off; keep fan at 100%
  until fresh temperature is at or below 45 C, then set 30%; clear the lease;
  persist `mineonboot=false`; retain pool settings; and release owned USB and
  process resources. If device-local stop cannot be confirmed, one exact
  baseline reflash is allowed only after same-device re-admission; otherwise
  stop.
- Retry bound: `attempt-001` is sealed and immutable; it may not be repeated.
  The owner-authorized non-invasive remediation added a closed terminal reason
  and aligned status freshness with the authoritative runtime clock, providing
  objective change proof for `attempt-002`. That attempt selected a new ASIC
  failure boundary. The pinned upstream result loop treats invalid receives as
  a dropped iteration and continues, while the Rust poll path terminalized a
  complete malformed frame. A deterministic soft-discard regression plus that
  boundary-changing fix authorized `attempt-003`. That attempt produced the
  new refined signature `production_asic_poll_unavailable`; four valid
  candidates preceded it and the closed malformed-discard count remained
  zero. The owner could enqueue another poll on unrelated wakeups while one
  was in flight, allowing bounded worker-queue backpressure to surface as the
  same terminal poll category. A deterministic one-poll-in-flight regression,
  the invariant fix, and distinct queue-full/worker-disconnected categories
  authorize exactly one `attempt-004`. That attempt emitted the typed
  `set_fan_duty_to_100_percent` started boundary but neither completion nor
  failure while runtime attestations remained live for the full capture. A
  deterministic deferred-reply fan-enqueue regression, non-blocking observation
  access, and host classification of incomplete preparation authorize exactly
  one `attempt-005`. Any later ordinal requires another new deterministic
  regression and verified boundary change. `attempt-005` completed every
  preparation boundary and sustained active mining, but the newly introduced
  non-blocking observation read could synthesize stale truth when the producer
  woke the owner while still holding the observation mutex. A deterministic
  release-before-wakeup regression and removal of that false-stale path
  authorize exactly one `attempt-006`. A recurrence of the same refined
  authoritative signature selects `stop_repeated_boundary`.
- Accepted terminal outcome: `complete` only for `submit_response_observed`
  with `submit_outcome=accepted`, trusted exact-package identity, clean serial
  diagnostics, fresh supported safety, `mineonboot=false`, confirmed safe
  stop, lease removal, USB cleanup ready, valid artifact seals and modes, and
  no parity promotion. A rejected response or any other failure stops without
  retry and returns to diagnosis under the retry-bound rule.

Verification: The pre-fix regression reproduced submission of a known
below-pool-target reference nonce, and the BM1366 parser regression reproduced
the nonce byte swap. The pinned upstream vectors prove the corrected header
reconstruction and difficulty calculation, while the production wire-frame
test proves the submit nonce is preserved. The fix also keeps distinct
candidates from one job eligible and ignores exact duplicates. `cargo fmt
--all`, warnings-as-errors Clippy, all-target/all-feature build, all-feature
tests, focused ASIC/Stratum/flash tests, production-session verification,
`just test`, `just package`, Bright Builds checks, parity, reference
cleanliness, redaction, and diff checks pass. Hardware evidence remains
pending. Clean commit `db1974ac` `attempt-001` admitted the exact package and
one board, preserved trusted runtime identity and clean serial framing, and
sealed fresh supported observations, `mineonboot=false`, confirmed safe stop,
and ready USB cleanup. It correctly counted one below-pool-target ASIC
candidate and submitted none, but consumed after `5,835` active milliseconds
with `submit_response_missing`. Because v5 did not retain the session blocker
and status freshness used a different clock origin than the authoritative
safety gate, the exact early-stop cause remains unclassified. Ephemeral raw
detector and console logs were deleted after extracting these closed facts;
the sealed private attempt remains ignored and non-promoted.
Clean commit `04de47c9` `attempt-002` reproduced one below-pool-target
candidate and no submit, then sealed `terminal_reason=production_asic_unavailable`
after `5,644` active milliseconds with the same trusted identity, clean serial
framing, fresh supported observations, safe stop, and cleanup guarantees. This
rules out the host parser, pool response, and safety gate as the immediate
stop. The result-loop fix now soft-discards malformed complete frames like the
pinned upstream loop and adds closed version-mask/dispatch/poll terminal
subtypes for any remaining ASIC failure. Ephemeral attempt-002 detector and
console logs were deleted; sealed private evidence remains ignored and
non-promoted.
Clean commit `7d871de8` `attempt-003` retained trusted identity, clean serial
framing, fresh supported observations, `mineonboot=false`, confirmed safe
stop, and ready USB cleanup. It classified four below-pool-target candidates,
zero qualified candidates, and no submit before
`terminal_reason=production_asic_poll_unavailable` after `5,765` active
milliseconds. No malformed-frame discard occurred, disproving that trigger
for this attempt. The shell/core review then identified untracked concurrent
poll requests as a concrete worker-queue backpressure path. Ephemeral detector
and console logs were deleted after extracting only these closed facts; the
sealed private attempt remains ignored and non-promoted.
Clean commit `a80060fe` `attempt-004` retained trusted identity, clean serial
framing, fresh supported observations, `mineonboot=false`, ready USB cleanup,
and valid private artifact modes and seals. It never reached active mining or
read the local pool input. Three typed preparation events ended at
`step=set_fan_duty_to_100_percent,outcome=started`; no completion or failure
followed even though runtime attestations continued through the full bounded
capture. This disproves the poll invariant as the immediate attempt-004
boundary and isolates a synchronous fan-preparation stall. The host's
`pool_configuration_missing` result and retained `network_unavailable` marker
were pre-preparation state, not evidence of a new network failure. Ephemeral
detector and console logs were deleted after extracting only these closed
facts; the sealed private attempt remains ignored and non-promoted.
Clean commit `ec23da41` `attempt-005` crossed the preparation and ASIC failure
boundaries: all nine preparation steps completed, active mining continued for
`439,041` milliseconds, and 60 valid candidates were classified below the
active pool target with zero duplicates, qualified candidates, or submissions.
The attempt then safe-stopped on `terminal_reason=safety_prerequisites_stale`
with trusted identity, clean serial framing, local owner pool input admitted,
fresh supported terminal observations, `mineonboot=false`, confirmed safe
stop, ready USB cleanup, and valid private artifact modes and seals. The
producer called the owner wakeup while still holding the observation-store
mutex, and the new non-blocking reader converted that transient contention
into an empty stale snapshot. This identifies a deterministic false-stale race
rather than a physical safety observation failure. The sealed private attempt
remains ignored and non-promoted; no raw serial or credential material was
retained.

Completion review: Pending. An accepted share proves this bounded conservative
owner-pool path only; it does not prove profitability, default-profile safety,
unbounded stability, release readiness, or parity promotion.

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
