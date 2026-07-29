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

### task-ultra205-mining-observation-baseline | 2026-07-28 | Re-establish a known-safe mining observation baseline

Status: Blocked — `stop_repeated_boundary`.

- [ ] Build and admit the exact current-HEAD package after its software
      dependencies complete.
- [ ] Detect exactly one Ultra 205 and run the observation campaign for 360
      seconds.
- [ ] Prove exact source/package runtime attestation, all six safety
      observations fresh, `mineonboot=false`, no campaign lease, no pool-secret
      read, and no fan, voltage, or ASIC actuation.
- [ ] Seal the private result with one accepted terminal outcome and preserve
      exact non-claims for mining, shares, soak, and parity promotion.

Dependencies: Complete
`task-ultra205-safety-observation-completeness` and
`task-production-mining-live-io`.

Hardware contract:

- Permitted commands:
  1. `just detect-ultra205`
  2. `just package`
  3. `just mining-campaign stage=observation board=205 port=<detector-port> manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json evidence-dir=scratch/ultra205-mining-observation-baseline/attempt-001 duration-seconds=360 redact-evidence=true`
- Objective: establish a fresh current-architecture, exact-package,
  observe-only baseline on the single detected board 205 without reopening or
  retrying the terminal Phase 36 lineage.
- Evidence: the ignored
  `scratch/ultra205-mining-observation-baseline/attempt-001` root is private,
  non-promoted `ProtectedOperational` evidence. Its parent is mode 0700 and
  artifacts are mode 0600. Only redacted closed categories, bounded counts,
  durations, and safe build provenance may be summarized.
- Preconditions: all dependency and software gates pass; the source tree and
  reference are clean; `just package` freezes an exact current-HEAD manifest;
  the detector admits exactly one ESP32-S3 board 205; the local Wi-Fi
  credential file is present but never printed; no pool credential is read or
  supplied. The completed current-architecture dependencies are required new
  diagnostic information relative to the archived Phase 36 boundary.
- Allowed effects: write and verify only the exact admitted factory image,
  inject the local Wi-Fi configuration through the existing private NVS path,
  persist `mineonboot=false`, perform repo-owned reset/re-enumeration,
  receive-only serial observation, use a fresh same-session origin-only device
  URL when uniquely available, and clean up supervisor-owned child processes.
- Prohibited effects: pool configuration, pool network connections, mining
  lease creation, fan/voltage/ASIC actuation, erase-flash, arbitrary raw
  writes, OTA, recovery upload, network discovery, foreign-process
  termination, evidence promotion, direct UART, pins, pads, headers, GPIO,
  probes, jumpers, soldering, or injected signals.
- Recovery/restoration: terminate and reap supervisor-owned process groups,
  release serial descriptors, persist `mineonboot=false`, leave no campaign
  lease, and prove the admitted device accessible and holder-free. Success
  leaves the exact admitted package installed. Identity drift, device absence,
  a foreign holder, or unproved cleanup stops without physical intervention.
- Retry bound: no unchanged retry. A fresh ordinal is allowed only after a
  targeted regression-backed fix or authorized non-invasive remediation
  objectively changes the authoritative boundary. One post-fix recurrence
  selects `stop_repeated_boundary`.
- Accepted terminal outcomes: `complete`, `stop_repeated_boundary`,
  `stop_hardware_blocker`, `stop_authority_boundary`, or
  `stop_impossible_contract`. Preserve the earliest typed failure.

Verification: Blocked at the one permitted post-fix attempt. All software,
package, parity, reference, redaction, and clean exact-HEAD gates passed at
`a6cc0a20`. The initial detector run preserved `recovery_not_observed` at final
cleanup: the same device was accessible and holder-free, but the 30-second
window admitted only two of three required stable samples. The targeted
60-second final-cleanup fix and its slow-sampler regression test passed and
were committed before the second detector run. That detector admitted exactly
one device and completed cleanup. The 360-second campaign then sealed
`marker_invalid` with zero accepted markers, runtime identity not trusted,
package admission true, and USB cleanup ready. Private attempt evidence
permissions and the result seal passed.

Completion review: Terminal blocker; do not archive or claim this baseline
complete. No pool credential was supplied and the sealed result makes no
mining, share, soak, or parity-promotion claim. The serial stop/parser path
returned `marker_invalid` before accepting any marker. Because the transcript
is intentionally ephemeral, the sealed result cannot distinguish a non-UTF-8
boot byte outside a marker from a malformed marker line. Resume requires
regression-backed byte-safe marker framing for both cases plus explicit
authorization for a new hardware ordinal because the current retry contract is
exhausted.

### task-ultra205-live-pool-share | 2026-07-28 | Prove one real BM1366 pool submission

- [ ] Freeze and admit the exact current-HEAD package, single detected board
      205, ignored local Wi-Fi credentials, and exactly one ignored local pool
      credential file.
- [ ] Run the `conservative` profile at 400 MHz, 1100 mV, and 100% fan until
      the first accepted/rejected submit response or the 600-second lease
      expires.
- [ ] Prove pool authorization, notify-derived work, BM1366 dispatch, parsed
      nonce, matching generation/job correlation, share submission, and an
      accepted or rejected pool response.
- [ ] Confirm device-local safe-stop, lease removal, persisted
      `mineonboot=false`, retained owner-supplied pool configuration, and the
      new firmware paused after the attempt.
- [ ] Seal one private, redacted result without automatic parity promotion.

Dependencies: Complete tasks
`task-production-mining-hardware-lifecycle`,
`task-ultra205-safety-observation-completeness`,
`task-ultra205-mining-actuation-adapter`,
`task-production-mining-live-io`, and
`task-ultra205-mining-observation-baseline`.

Hardware contract:

- Permitted commands:
  1. `just detect-ultra205`
  2. `just package`
  3. `just mining-campaign stage=live-share profile=conservative board=205 port=<detector-port> manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json pool-credentials=<single-ignored-local-pool-file> evidence-dir=scratch/ultra205-live-pool-share/attempt-001 duration-seconds=600 redact-evidence=true`
- Objective: obtain one real BM1366 nonce correlated to owner-pool work and
  one accepted or rejected Stratum V1 submit response under the conservative
  profile, then prove safe stop.
- Evidence: the ignored `scratch/ultra205-live-pool-share/attempt-001`
  root is private, non-promoted `ProtectedOperational` evidence with mode-0700
  parent and mode-0600 artifacts. Committed or summarized output may record
  `pool_config: local-owner-supplied`, closed result categories, bounded
  counts, durations, and safe provenance only. It must never contain raw pool
  URL, port, user, worker, owner address, password, endpoint, token, NVS secret,
  Wi-Fi value, device URL, IP, MAC, or unredacted logs.
- Preconditions: all five dependencies and software gates pass; exact
  current-HEAD package identity is frozen; detector admission finds exactly
  one board 205; the prior observation baseline completed; exactly one ignored
  pool credential file and the ignored Wi-Fi credential file exist; the pool
  declares Stratum V1 over ordinary TCP; TLS is rejected as out of scope.
- Allowed effects: private NVS injection of Wi-Fi and owner pool settings,
  persistence of `mineonboot=false`, installation of one conservative
  campaign lease, exact package flash, repo-owned USB reset/re-enumeration,
  fan 100%, DS4432U 1100 mV, ASIC enable/reset, BM1366 initialization and
  work/result traffic, Stratum V1 TCP connection and submission, fresh-session
  HTTP/WebSocket observation, and bounded device-local safe-stop.
- Safety and stop limits: observations must remain fresh; input must remain
  4.5-5.5 V; power must not exceed 15 W; ASIC temperature must remain below
  75 C; fan RPM must remain fresh and nonzero after the qualified 100% command.
  Any sensor, watchdog, actuation, generation, lease, transport, or evidence
  fault blocks submissions and begins safe-stop immediately.
- Prohibited effects: TLS, Stratum V2, automatic fan mode, unbounded mining,
  non-205 hardware, erase-flash, arbitrary raw writes, OTA, recovery upload,
  network discovery, foreign-process termination, raw secret output, parity
  promotion, direct UART, pins, pads, headers, GPIO, probes, jumpers,
  soldering, injected signals, voltage/fan stress, or fault injection.
- Recovery/restoration: preserve the earliest typed failure; block and
  invalidate submissions; close owned pool transports; frequency-down and
  reset the ASIC; set core voltage off and ASIC enable off; keep fan at 100%
  until fresh temperature is at or below 45 C, then set 30%; clear the lease;
  persist `mineonboot=false`; retain pool settings; and release owned USB and
  process resources. If device-local stop cannot be confirmed, one
  predeclared exact-baseline reflash is allowed only after same-device
  re-admission; otherwise stop.
- Retry bound: one fresh attempt only. No timeout, no-share, rejected-share,
  transport, ASIC, or safety result authorizes an unchanged retry. A later
  ordinal requires a targeted regression-backed fix or authorized
  non-invasive remediation with objective boundary-change proof; one post-fix
  recurrence selects `stop_repeated_boundary`.
- Accepted terminal outcomes: `complete` only when every success and safe-stop
  criterion passes; otherwise `stop_repeated_boundary`,
  `stop_hardware_blocker`, `stop_authority_boundary`, or
  `stop_impossible_contract`.

Verification: Pending. Run all required software gates, the exact permitted
hardware commands, private-artifact permission checks, redaction and secret
denylist verification, lease/safe-stop validation, sealed result validation,
and final diff review.

Completion review: Pending. An accepted or rejected response proves the
end-to-end submitted-share path, not profitability, unbounded stability,
release readiness, or parity promotion.

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
