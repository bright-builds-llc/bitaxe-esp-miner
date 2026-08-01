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

- [x] Start a fresh exact-package attempt at 485 MHz, 1200 mV, and 100% fan
      only after the conservative live-share task completes.
- [x] Count 600 seconds from authorized active mining rather than boot or
      connection start.
- [ ] Require uninterrupted fresh safety truth, watchdog responsiveness,
      active work renewal, at least one new correlated nonce and pool response,
      and correlated HTTP/WebSocket state throughout the soak.
- [x] Confirm the device-local lease expires, hardware safe-stop completes,
      the lease is cleared, `mineonboot=false` persists, pool settings remain,
      and the new firmware remains installed in paused state.
- [x] Seal one private, redacted soak result without automatic parity
      promotion.

Dependencies: Complete `task-ultra205-live-pool-share` successfully.

Hardware contract:

- Permitted commands:
  1. `just detect-ultra205`
  2. `just package`
  3. `just mining-campaign stage=soak profile=upstream-default board=205 port=<detector-port> manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json pool-credentials=<single-ignored-local-pool-file> evidence-dir=scratch/ultra205-default-profile-soak/attempt-003 duration-seconds=600 redact-evidence=true`
  4. One recovery-only same-origin `POST /api/system/pause`, issued in-process
     by the admitted campaign observer only after its first network or watchdog
     failure. It is not an operator command or a general network-control grant.
- Objective: prove the exact package can mine for 600 active seconds at the
  Ultra 205 upstream-default profile with fresh safety, watchdog,
  work/result/share, HTTP, WebSocket, and final safe-stop evidence.
- Evidence: the ignored
  `scratch/ultra205-default-profile-soak/attempt-003` root is private,
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
- Attempt-003 continuation: run the exact command above once, only after the
  watchdog, HTTP, WebSocket, continuity-window, terminal-persistence, and
  recovery-pause regressions pass every required software gate on a clean,
  pushed exact HEAD. The private recovery summary must still report
  `phase=monitor_admission`, `deadline_seconds=60`, and 3 stable samples before
  runtime evidence can be trusted.
- Attempt-003 acceptance: divide the 600 device-reported active seconds into
  twenty half-open 30-second windows. Every window must contain successful HTTP
  and reconstructed WebSocket observations from the same boot and exact
  package, active mining, fresh bounded safety truth, non-regressing counters
  and snapshot revisions, healthy supervisor state, advancing task-watchdog
  feed and supervisor-checkpoint sequences, and advancing ASIC poll activity.
  Active serial markers may be no more than 5,000 ms apart, and the attempt
  must contain at least one new correlated nonce plus accepted pool response.
  Within ten seconds of the consumed marker, both HTTP and reconstructed
  WebSocket state must prove the same boot/package is paused with
  `mineonboot=false`, healthy watchdog participation, confirmed device-local
  safe-stop, and a terminal NVS reread must prove a valid configured pool still
  exists without exposing its values.
- Attempt-003 failure handling: preserve the earliest closed failure category,
  issue at most the one recovery-only pause request above when a trusted origin
  exists, and continue observation until device-local safe-stop is confirmed or
  terminal grace expires. Without a trusted origin, rely on lease expiry and
  leave safe-stop unconfirmed unless serial proves it. Any failure ends this
  targeted effort and is recorded without retry. No TLS, discovery, fault
  injection, parity promotion, or expanded hardware authority is allowed.
- Accepted terminal outcomes: `complete` only when the full active-duration,
  correlation, and safe-stop criteria pass; otherwise
  `stop_repeated_boundary`, `stop_hardware_blocker`,
  `stop_authority_boundary`, or `stop_impossible_contract`.

Verification: `stop_hardware_blocker` on attempt 001. `cargo fmt --all`, strict
Clippy, the all-target/all-feature Cargo build and tests, all 82 Bazel test
targets, the managed Bright Builds checks, `just package`, and detector
admission passed against exact source commit `8e75d046`. The campaign admitted
the package and completed both supervised writes, then failed before runtime
observation. Its protected result sealed with a matching digest and recorded
`observation_failed`, zero markers, `safe_stop=not_observed`, and successful USB
cleanup. The authoritative monitor-admission recovery summary observed the same
accessible, holder-free device but reached only 2 of 3 stable samples within
the 30-second bound; final cleanup later reached 3 of 3 within 60 seconds. The
one-shot campaign keys were consumed and erased before use, and the conservative
720-second lease-plus-stop margin elapsed without another device effect, but
elapsed time is not safe-stop evidence. No unchanged retry is authorized.

Attempt-002 verification: `complete` for the targeted monitor-admission fix.
All required software gates passed and the clean exact package from source
commit `0e3f19d5` was pushed before hardware use. Fresh detector admission found
exactly one Ultra 205. The protected monitor-admission recovery summary recorded
a 60-second deadline and reached 3 of 3 same-device, accessible, holder-free
stable samples. The sealed campaign result recorded `status=accepted`,
`terminal_category=soak_duration_complete`, 600.501 active seconds, trusted
package/runtime identity, 10 accepted and zero rejected shares, fresh safety,
`mineonboot=false`, confirmed safe-stop, and USB cleanup ready. Its result seal,
mode-0700 root, mode-0600 files and recovery summaries, redaction denylist, and
non-promotion state all passed. No retry was run and no evidence was promoted.

Attempt-003 verification: `stop_hardware_blocker` at the distinct closed
`network_correlation_failed` boundary; no retry was run. All required software
gates passed, source commit `da32b67d` was pushed to `main`, and the clean exact
package was built before hardware use. Fresh detector admission found exactly
one Ultra 205. Monitor admission used its 60-second deadline and reached 3 of 3
same-device, accessible, holder-free stable samples. The sealed result proved
trusted package/runtime identity, 600.081 active seconds, 21 qualified and
accepted correlated shares with zero rejected shares, fresh safety, a maximum
344-ms active-marker gap, consumed lease, `mineonboot=false`, retained pool
configuration, final HTTP and reconstructed WebSocket paused-state evidence,
confirmed device-local safe-stop, and USB cleanup ready. The in-process
observer issued its single recovery pause request and preserved the earliest
failure. Its continuity artifact recorded `active_state_valid=false`, zero
credited active HTTP/WebSocket samples and zero complete windows; watchdog and
work-renewal acceptance therefore also remained uncredited. Result/artifact
digests, mode-0700 root, mode-0600 files, redaction, and non-promotion checks
passed. This is a distinct observer-correlation boundary, not recurrence of
the attempt-001 monitor-admission boundary.

Software-only startup recovery verification: `cargo fmt --all`, strict Clippy,
the all-target/all-feature Cargo build and tests, all 82 Bazel test targets, the
managed Bright Builds checks, and `just verify-redaction` passed. Deterministic
regressions prove the production campaign becomes active before work submission
changes public mining activity from `safe_blocked` to `active`; HTTP and
WebSocket then establish independently, uncredited startup samples cannot alter
counts, baselines, gaps, or pause behavior, and the exact 30,000-ms boundary
still fails an incomplete window. The private v2 continuity artifact remains
sealed, mode-0600, aggregate-only, and identifier/secret-free. No hardware,
package, credential, discovery, or attempt-004 action was performed.

Completion review: Attempt 003 closes the previously missing terminal pool,
HTTP/WebSocket paused-state, and persistence evidence, but the broader task
remains active and is not archived because none of the twenty active continuity
windows was credited after the first active-state correlation rejection.
Continuous HTTP/WebSocket state, watchdog sequence advancement, and ASIC work
renewal therefore remain unverified even though the serial mining/share and
terminal paths succeeded. The exact-one-attempt authorization is consumed; a
later attempt requires a new targeted regression-backed fix and fresh task
contract. The task still does not authorize or verify automatic fan control,
unbounded mining, complete statistics/hashrate parity, release readiness, or
checklist promotion.

Software-only continuation: the campaign observer now treats a valid-identity,
safe, watchdog-fresh non-active sample as an uncredited startup transition until
HTTP and WebSocket independently observe their first active sample. Window 0
remains half-open at `[0, 30000)` and retains its full two-sample and sequence
advancement contract; all identity, safety, watchdog, regression, terminal,
and post-establishment mining-state failures remain fail-closed. This change
does not alter or renew the consumed attempt-003 command, authorize attempt-004,
or supply hardware evidence. Any future soak hardware contract is additionally
blocked on `task-campaign-websocket-connection-stability`.

### task-campaign-websocket-connection-stability | 2026-08-01 | Stabilize campaign WebSocket observation

- [ ] Reproduce the attempt-003 signature of 109 reconnects with real loopback
      TCP connections and representative idle intervals.
- [ ] Verify Tungstenite's transport contract that `WouldBlock` permits reuse
      while `TimedOut` and other I/O errors are generally fatal.
- [ ] Implement a bounded, non-busy persistent read strategy that preserves the
      existing 64-KiB message cap, plain-`ws://` restriction, omitted `Origin`
      header, and privacy boundary.
- [ ] Prove idle connections remain open, genuine peer closes reconnect with
      1-5-second bounded backoff, and sockets, threads, and related resources
      are released on every terminal path.
- [ ] Re-run the campaign transport, continuity, evidence, and redaction suites
      without weakening window completeness or earliest-failure precedence.

Dependencies: Complete the software-only startup-state recovery recorded in
`task-ultra205-default-profile-soak`. Completion of this task is a prerequisite
for drafting or authorizing any future soak hardware contract.

Authorization boundary: software and real loopback TCP tests only. This task
does not authorize hardware use, package flashing, Wi-Fi or pool credentials,
device discovery, raw device origins, attempt-004, or any other network target.

Verification: Pending.

Completion review: Pending. The implementation must resolve the 109-reconnect
signature before a future hardware attempt can be considered; tracking this
work does not itself renew or expand the consumed soak authorization.

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
