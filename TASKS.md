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

### task-ultra205-live-pool-share | 2026-07-28 | Prove one real BM1366 pool submission

Status: In progress — `attempt-001` proved an untyped hardware-preparation
failure that the host incorrectly sealed as `pool_configuration_missing`;
`attempt-002` is authorized only after the earliest typed preparation failure
is regression-backed, fully verified, committed, pushed, and rebuilt from
clean exact HEAD.

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
  4. `just mining-campaign stage=live-share profile=conservative board=205 port=<detector-port> manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json pool-credentials=<single-ignored-local-pool-file> evidence-dir=scratch/ultra205-live-pool-share/attempt-002 duration-seconds=600 redact-evidence=true`
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
  transport, ASIC, or safety result authorizes an unchanged retry. The user
  authorized sequential retries after changes until this task completes. A
  later ordinal requires a targeted regression-backed fix or authorized
  non-invasive remediation with objective boundary-change proof plus a task
  amendment naming its exact command and evidence root; one post-fix recurrence
  of the same authoritative boundary signature selects
  `stop_repeated_boundary`. `attempt-002` changes the authoritative evidence
  boundary: the firmware marker and sealed result must preserve the earliest
  closed hardware-preparation phase, step, adapter category, and any secondary
  rollback failure so the misleading `pool_configuration_missing` precedence
  cannot recur. Do not infer or change hardware behavior until that typed
  result identifies the failed boundary.
- Accepted terminal outcomes: `complete` only when every success and safe-stop
  criterion passes; otherwise `stop_repeated_boundary`,
  `stop_hardware_blocker`, `stop_authority_boundary`, or
  `stop_impossible_contract`.

Verification: `attempt-001` used exact clean-HEAD commit `0e84acc5`, one
detected Ultra 205, and exactly one ignored owner pool input. It stopped before
active mining and sealed package/runtime identity trusted, clean serial
framing, five supported observations fresh, `mineonboot=false`, zero active
milliseconds, no submit response, confirmed safe-stop, USB cleanup ready, and
no parity promotion. Its seven markers remain `pool_config=not_read`, move
directly from the pre-session projection to consumed safe-stop, and prove an
earlier preparation failure. Because marker v3 carries no typed preparation
failure, the host incorrectly selected `pool_configuration_missing`. Run the
new red diagnostic-precedence regression, all required software gates, the
exact permitted hardware commands, private-artifact permission checks,
redaction and secret denylist verification, lease/safe-stop validation, sealed
result validation, and final diff review.

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
