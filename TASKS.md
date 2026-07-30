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

Status: In progress — `attempt-004` proved an incorrect Ultra 205 temperature
capability mapping; `attempt-005` is authorized after the verified correction.

- [x] Reproduce the zero-marker `marker_invalid` boundary with non-UTF-8
      non-candidate bytes surrounding valid runtime attestations and campaign
      markers.
- [x] Replace whole-stream UTF-8 conversion with incremental LF framing,
      candidate-only decoding, independent runtime-attestation assessment, and
      earliest-failure preservation.
- [x] Add sealed `mining-campaign-result-v2` and private
      `mining-campaign-serial-diagnostics-v1` evidence with bounded typed
      events, aggregate counts, and no raw serial or candidate content.
- [x] Reproduce the `attempt-002` false-to-true boot-preference loss at the
      pure reload boundary and make persistence load both upstream and
      project-owned settings schemas.
- [x] Add a closed per-source freshness projection so a five-of-six safety
      result names the unavailable observation without sensor values or raw
      serial.
- [x] Prove from sealed hardware evidence and the pinned board-205 reference
      that EMC2101 internal temperature is the supported ASIC-temperature
      source and the DS4432U path has no VR-temperature sensor.
- [x] Pass every required software, package, parity, reference, and redaction
      gate for the parser and diagnostics change before committing it.
- [x] Build and admit the exact current-HEAD package after its software
      dependencies complete.
- [x] Detect exactly one Ultra 205 and run the single authorized observation
      attempt.
- [ ] Prove exact source/package runtime attestation, all five supported
      Ultra 205 safety observations fresh, VR temperature explicitly
      unsupported, `mineonboot=false`, no campaign lease, no pool-secret read,
      and no fan, voltage, or ASIC actuation.
- [ ] Seal the private result with one accepted terminal outcome and preserve
      exact non-claims for mining, shares, soak, and parity promotion.

Dependencies: Complete
`task-ultra205-safety-observation-completeness` and
`task-production-mining-live-io`.

Hardware contract:

- Permitted commands:
  1. `just detect-ultra205`
  2. `just package`
  3. `just mining-campaign stage=observation board=205 port=<detector-port> manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json evidence-dir=scratch/ultra205-mining-observation-baseline/attempt-002 duration-seconds=360 redact-evidence=true`
  4. `just mining-campaign stage=observation board=205 port=<detector-port> manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json evidence-dir=scratch/ultra205-mining-observation-baseline/attempt-003 duration-seconds=360 redact-evidence=true`
  5. `just mining-campaign stage=observation board=205 port=<detector-port> manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json evidence-dir=scratch/ultra205-mining-observation-baseline/attempt-004 duration-seconds=360 redact-evidence=true`
  6. `just mining-campaign stage=observation board=205 port=<detector-port> manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json evidence-dir=scratch/ultra205-mining-observation-baseline/attempt-005 duration-seconds=360 redact-evidence=true`
- Objective: establish a fresh current-architecture, exact-package,
  observe-only baseline on the single detected board 205 without reopening or
  retrying the terminal Phase 36 lineage.
- Evidence: preserve the sealed ignored `attempt-001` root unchanged. The new
  ignored `scratch/ultra205-mining-observation-baseline/attempt-002` root is
  private, non-promoted `ProtectedOperational` evidence. Its parent is mode
  0700 and artifacts are mode 0600. Only redacted closed categories, bounded
  counts, durations, and safe build provenance may be summarized. The new
  typed diagnostic trace must never contain raw serial bytes, candidate
  payloads, excerpts, identifiers, credentials, endpoints, or secret-derived
  hashes.
- Diagnostic contract: `campaign-diagnostics.private.json` is mode 0600 and
  records only aggregate byte, line, candidate, accepted-marker, encoding,
  JSON, schema, and trailing-partial counts plus the first and last 32 typed
  events. `campaign-result.json` uses `mining-campaign-result-v2`, binds the
  diagnostic artifact digest, keeps `marker_invalid` compatible, records one
  closed `serial_outcome_detail`, and records runtime attestation independently.
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
- Retry bound: the user authorized sequential hardware retries after changes
  until the task completes. `attempt-003` is authorized only after a
  deterministic regression proves and a clean committed/pushed fix repairs the
  `mineonboot=false` state boundary exposed by `attempt-002`. Never run an
  unchanged retry. `attempt-004` is authorized only after the v2 per-source
  freshness diagnostic is regression-backed, fully verified, committed,
  pushed, and rebuilt from clean exact HEAD. `attempt-005` is authorized only
  after the board-205 temperature source and capability correction is
  regression-backed, fully verified, committed, pushed, and rebuilt from clean
  exact HEAD. After any later failure, diagnose its closed boundary,
  verify one targeted fix or objective non-invasive boundary change, and amend
  this contract with the exact next ordinal and command before hardware use.
  A recurrence of the same authoritative boundary signature after its targeted
  verified fix selects `stop_repeated_boundary`. Observation completion also
  authorizes the existing conservative live-share task under its own
  change-gated retry contract.
- Accepted terminal outcomes: `complete`, `stop_repeated_boundary`,
  `stop_hardware_blocker`, `stop_authority_boundary`, or
  `stop_impossible_contract`. Preserve the earliest typed failure.

Verification: In progress under the newly authorized parser diagnosis and
`attempt-002` contract. The deterministic pre-fix regression sealed
`marker_invalid` with zero markers when otherwise valid observation input was
surrounded by non-UTF-8 noise. The byte-safe implementation passes the focused
194-test `bitaxe-flash` suite, the ordered Rust format/Clippy/build/test gates,
`just verify-production-session`, all 82 Bazel tests, `just package`, Bright
Builds checks, parity with no validation errors, reference cleanliness, and
redaction verification. All prior software, package, parity, reference,
redaction, and clean exact-HEAD gates passed at
`a6cc0a20`. The initial detector run preserved `recovery_not_observed` at final
cleanup: the same device was accessible and holder-free, but the 30-second
window admitted only two of three required stable samples. The targeted
60-second final-cleanup fix and its slow-sampler regression test passed and
were committed before the second detector run. That detector admitted exactly
one device and completed cleanup. The 360-second campaign then sealed
`marker_invalid` with zero accepted markers, runtime identity not trusted,
package admission true, and USB cleanup ready. Private attempt evidence
permissions and the result seal passed.

`attempt-002` admitted the clean exact-HEAD package at `44a85c4d` and exactly
one Ultra 205, then stopped on the first accepted observation marker before the
360-second window completed. The sealed v2 result records
`mineonboot_enabled`, package admission true, runtime attestation missing, one
accepted marker, `mineonboot=true`, safety stale with five of six fresh
observations, no pool read, no actuation, and USB cleanup ready. The sealed
serial diagnostic records clean framing, no invalid bytes or malformed
candidates, and `serial_outcome_detail=clean`. This disproves a repeated parser
failure for the new attempt, does not identify the exact historical byte-level
trigger, and selects the no-retry stop required by the authorized contract.
The host stop predicate also needed a follow-up regression-backed correction
so observation contract failures retain the full diagnostic window; that
software correction does not authorize another hardware ordinal.

For `attempt-003`, the exact host regression
`cargo test -p bitaxe-config persistence_reload_preserves_project_boot_preference`
failed before the fix because stored `mineonboot=0` reloaded as no typed value,
which made firmware callers use their fail-safe `true` fallback. The same test
passes after `load_values` chains the deliberately separate project-owned
schema, and all 48 `bitaxe-config` plus all 195 focused `bitaxe-flash` tests
pass. The ordered Rust format, warnings-denied Clippy, all-target build, and
all-feature test gates pass; `just verify-production-session`, all 82 Bazel
tests, `just package`, Bright Builds checks, parity validation with no errors,
reference cleanliness, and redaction verification also pass. Clean-HEAD
commit `5cd7ff02`, push, exact-HEAD rebuild, and `attempt-003` hardware
verification also pass.

`attempt-003` used the clean exact-HEAD package at `5cd7ff02` and completed the
full 360-second window. Its sealed result and bound diagnostics pass mode,
digest, and result-seal checks. They record exact-package runtime identity
trusted, `mineonboot=false`, 719 accepted markers, clean serial framing, no pool
read, no actuation, safe-stop not required, and USB cleanup ready. The distinct
terminal boundary is `safety_stale`: every marker reports exactly five of six
fresh observations. The aggregate marker does not identify the missing source,
so the firmware/host marker contract is being advanced to
`mining-campaign-status-v2` with six closed Boolean freshness fields and a
count-consistency check. The firmware marker test failed red against v1, then
passed with the v2 projection. All 196 focused `bitaxe-flash` tests, the
ordered Rust format, warnings-denied Clippy, all-target build, all-feature
tests, production-session verification, all 82 Bazel tests, packaging, Bright
Builds checks, parity validation with no errors, reference cleanliness, and
redaction verification pass for the v2 diagnostic change.

`attempt-004` used clean exact-HEAD commit `2d6a8e73` and completed the full
360-second window. Its sealed result and bound diagnostics pass mode, digest,
and result-seal checks. All 719 accepted markers agree that only
`chip_temp_celsius` is stale while power, bus voltage, current, VR temperature,
and fan tach are fresh. Package identity and runtime attestation are trusted,
`mineonboot=false`, serial framing is clean, no pool was read, no actuation
occurred, and USB cleanup is ready. Comparison with pinned board-205 reference
configuration and thermal/power selection proves the Rust mapping was
backwards: Ultra 205 sets `emc_internal_temp=true`, so EMC2101 internal
temperature is ASIC temperature, while its DS4432U power path exposes no
VR-temperature source. The pre-fix readiness regression rejects an otherwise
safe Ultra 205 when VR temperature is explicitly unavailable, and the pre-fix
adapter regression fails because the board-specific internal-temperature
acquisition does not exist. Both regressions pass after mapping ASIC
temperature to EMC2101 internal, representing the unsupported VR source
without a fabricated stamp, requiring the other five independent facts, and
binding that exact requirement set in `mining-campaign-status-v3`. The host
rejects a contradictory requirement set or freshness count.

All 63 `bitaxe-safety`, 217 `bitaxe-api`, and 197 `bitaxe-flash` tests pass.
The ordered Rust format, warnings-denied Clippy, all-target build, all-feature
tests, production-session verification, all 82 Bazel tests, packaging, Bright
Builds checks, parity validation with no errors, reference cleanliness,
redaction verification, and diff checks also pass. A parity mutation test
exposed a same-process temporary-path collision at nanosecond clock
resolution; its atomic fixture suffix regression passes 20 consecutive runs
and the full parity suite passes afterward.

Completion review: Reopened by explicit user authorization for change-gated
hardware retries until completion. Do not run `attempt-005` until the
board-capability correction is fully verified, committed, pushed, and rebuilt
from clean exact HEAD. No pool credential was supplied to any observation attempt,
and their sealed results make no mining, share, soak, or parity-promotion
claim. The
`attempt-001` serial stop/parser path returned `marker_invalid` before accepting
any marker. Because that transcript is intentionally ephemeral, its sealed
result cannot distinguish a non-UTF-8 boot byte outside a marker from a
malformed marker line. The parser ambiguity is now fixed and the new typed
evidence isolates `attempt-002` at an independent device-state boundary.
The new authorization does not permit unchanged retries, direct UART or pin
work, raw serial persistence, evidence promotion, or any expansion of the
existing observation effects.

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
  transport, ASIC, or safety result authorizes an unchanged retry. The user
  authorized sequential retries after changes until this task completes. A
  later ordinal requires a targeted regression-backed fix or authorized
  non-invasive remediation with objective boundary-change proof plus a task
  amendment naming its exact command and evidence root; one post-fix recurrence
  of the same authoritative boundary signature selects
  `stop_repeated_boundary`.
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
