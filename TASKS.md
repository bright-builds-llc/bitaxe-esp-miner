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
or supply hardware evidence. The software prerequisite
`task-campaign-websocket-connection-stability` is now completed and archived;
the separately approved attempt-004 child contract below is now active and
retains the parent's bounded acceptance criteria.

Attempt-004 verification: `stop_repeated_boundary`; no retry was run. The
authorized exact package came from clean pushed source commit `760859ef`, all
required software gates and both GitHub workflows passed, and the manifest
recorded v3, six artifacts, matching source identity, and `source_dirty=false`.
Two private detector preflights each admitted exactly one Ultra 205 and cleaned
up successfully; the second was a bounded host-output diagnostic after the
local checker initially expected `port=` instead of the repository's `port:`
label. Only one mining campaign was launched. Its protected result sealed with
a valid digest and matching observation, serial-diagnostic, mining-diagnostic,
and v2 network-continuity digests. The mode-0700 attempt root contains six
mode-0600 artifacts, and the closed artifact denylist found no credential,
origin, URL, network-address, USB-path, or raw operational-path leakage. The
private mode-0600 monitor-admission summary recorded a 60-second deadline,
same-device/accessibility/holder-free truth, and 3-of-3 stable samples.

Attempt-004 completed 600.052 active seconds with 2,729 accepted serial
markers, a 341-ms maximum active-marker gap, fresh required safety truth, 24
accepted and zero rejected shares, advancing ASIC work, one completed block
transition, retained pool configuration, consumed lease, `mineonboot=false`,
valid final HTTP and reconstructed WebSocket paused state, confirmed
device-local safe-stop, and USB cleanup ready. The earliest typed campaign
failure remained `marker_invalid` with detail `marker_json_invalid`: exactly
one of 2,730 marker candidates contained invalid JSON. Independently, the
post-fix WebSocket signature recurred with 116 reconnects and a 6,455-ms
maximum WebSocket gap, exceeding the 5,000-ms contract; HTTP reached a
20,872-ms maximum gap and watchdog continuity was invalid. Although all twenty
windows received observations and work renewal remained valid, the v2 network
artifact correctly remained failed and the in-process observer issued its one
recovery pause request.

Attempt-004 closure: the repeated idle-reconnect signature after its targeted
software fix selects `stop_repeated_boundary` even though the preserved
earliest campaign failure is the distinct malformed-marker boundary. The
broader soak remains active and unverified. Attempt-004 authorization is
consumed; attempt-005, unchanged retry, parity promotion, and any expanded
hardware or diagnostic action are not authorized.

### task-ultra205-default-profile-soak-attempt-004 | 2026-08-01 | Run one bounded upstream-default soak retry

- [x] Reconfirm that the exact source HEAD is clean, pushed, and passes every
      software gate before building the exact Ultra 205 package.
- [x] Admit exactly one board 205 through `just detect-ultra205`, then run at
      most one upstream-default 600-active-second soak using the exact package
      and the private `scratch/ultra205-default-profile-soak/attempt-004`
      evidence destination.
- [ ] Require all twenty half-open 30-second continuity windows, a maximum
      WebSocket observation gap of 5,000 ms, valid sealed v2 continuity
      evidence, and no recurrence of the 109-idle-reconnect signature.
- [ ] Accept bounded same-origin WebSocket reconnects only when every active
      window and every terminal condition remains fully evidenced.
- [x] Preserve the previous board, profile, credential, safety, recovery,
      redaction, exact-package, single-attempt, and non-promotion boundaries.

Dependencies: `task-campaign-websocket-connection-stability` is completed and
archived. The active `task-ultra205-default-profile-soak` remains the parent
acceptance contract and is completed only if this attempt satisfies every
continuity and terminal requirement.

Hardware contract:

1. `just package`
2. `just detect-ultra205`
3. `just mining-campaign stage=soak profile=upstream-default board=205 port=<detector-port> manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json pool-credentials=<single-ignored-local-pool-file> evidence-dir=scratch/ultra205-default-profile-soak/attempt-004 duration-seconds=600 redact-evidence=true`
4. At most one in-process, same-origin recovery pause after the earliest network
   or watchdog failure, under the existing campaign observer contract.

Evidence and acceptance boundary: use a mode-0700 ignored private root
with mode-0600 artifacts; expose no credentials, identifiers, origins, URLs,
frames, bodies, or operational paths. Require trusted package/runtime identity,
the 60-second monitor-admission policy with 3-of-3 stable samples, fresh safety
and watchdog truth, advancing ASIC and supervisor activity, non-regressing
counters and revisions, correlated nonce and accepted share evidence, retained
pool configuration, consumed lease, `mineonboot=false`, final paused HTTP and
WebSocket state, confirmed safe-stop, USB cleanup, valid seals and digests, and
no parity promotion. The prior thermal, voltage, power, fan, identity,
ownership, credential, discovery, recovery, retry, and prohibited-effect limits
remain unchanged. Any incomplete window, observation gap above 5,000 ms,
repeated idle-reconnect signature, or terminal-proof failure stops without
retry.

Safety and effects: only the parent contract's board-205 USB admission, exact
package flash, private NVS credential injection, 600-active-second lease,
upstream-default 485-MHz/1200-mV/100%-fan mining, Stratum V1 traffic,
same-session HTTP/WebSocket observation, and bounded safe-stop are allowed.
Fresh input voltage must remain 4.5-5.5 V, power at most 15 W, ASIC temperature
below 75 C, and fan RPM fresh and nonzero. TLS, discovery, non-205 hardware,
mining beyond the lease, automatic fan mode, erase-flash, raw writes, OTA,
fault injection, parity promotion, foreign-process termination, direct UART,
pins, pads, headers, GPIO, probes, jumpers, soldering, and injected signals
remain prohibited.

Recovery and retry: preserve the earliest typed failure; block submissions;
close owned pool transports; frequency-down and reset the ASIC; turn core
voltage and ASIC enable off; keep the fan at 100% until fresh temperature is at
or below 45 C, then set 30%; clear the lease; persist `mineonboot=false`; retain
pool settings; and release owned USB and process resources. If device-local
stop cannot be confirmed, stop with `stop_hardware_blocker`; this child does not
authorize an operator reflash command. Exactly one attempt-004 run is
authorized. Do not retry an unchanged or repeated boundary.

Authorization boundary: the user explicitly named this stable task ID and
approved this exact hardware contract on 2026-08-01. That authorization covers
only the four command/effect surfaces above and expires when attempt-004 reaches
one terminal outcome. It does not authorize any later ordinal or broader work.

Accepted terminal outcomes: `complete` only when all twenty windows, the
5,000-ms WebSocket gap limit, v2 continuity evidence, share, persistence,
safe-stop, identity, cleanup, sealing, and privacy requirements pass. Otherwise
record `stop_repeated_boundary`, `stop_hardware_blocker`,
`stop_authority_boundary`, or `stop_impossible_contract` and stop without retry.

Verification: `stop_repeated_boundary`. Exact clean pushed commit `760859ef`,
all seven software gates, both GitHub workflows, manifest/package admission,
two successful private detector preflights, and exactly one campaign invocation
passed their respective boundaries. The sealed result proved the full active
duration, fresh safety, work/share progress, terminal persistence, safe-stop,
USB cleanup, modes, digests, redaction, and non-promotion. It failed closed on
one invalid-JSON serial marker and independently recorded 116 WebSocket
reconnects, a 6,455-ms WebSocket gap, a 20,872-ms HTTP gap, and invalid watchdog
continuity. No retry was run.

Completion review: Closed at `stop_repeated_boundary`. The targeted WebSocket
fix did not eliminate the real-device idle-reconnect signature, and the
5,000-ms continuity ceiling did not pass. The earliest `marker_invalid` failure
is preserved rather than overwritten by the later network evidence. This task
remains active and unarchived as a terminal blocker under the tracker rules;
its authorization is consumed and it cannot be selected for another hardware
run.

### task-parity-sys004-version-reporting | 2026-08-02 | Reconcile implemented version reporting

- [x] Audit the upstream firmware, AxeOS, and ESP-IDF version surface against
      the current canonical Rust build/platform/API projection.
- [x] Run focused build-identity, system-info wire, package-manifest, and
      runtime-attestation regressions plus every mandatory repository gate.
- [x] Commit the row evidence before transitioning only `SYS-004` from
      `in-progress` to `implemented`.
- [x] Synchronize parity progress and retain this task with the exact later
      live-evidence gate required for `verified`.
- [x] Correct `next-item` so a non-verified plan closes after its checklist
      status advances beyond the plan's recorded initial status.
- [x] Prove an unchanged-status plan still resumes while the completed
      `SYS-004` implementation plan yields the next candidate queue.
- [x] Run all mandatory repository gates, keep the checklist and progress
      history unchanged, and push the audited continuation.
- [x] Generate the canonical build label as package-owned SPIFFS
      `version.txt` and read that installed file for `axeOSVersion`.
- [x] Add a typed exact-package live version projection with focused
      regression coverage and a closed commit-safe schema.
- [x] Commit and push the software fix, build the exact package, and perform at
      most one detector-gated Ultra 205 verification attempt.
- [ ] Verify or conservatively retain only `SYS-004`, synchronize progress only
      if its checklist fields change, and push the audited result.

Plan: `docs/parity/work-plans/20260802T233821Z-SYS-004/PLAN.md`

Verification plan:
`docs/parity/work-plans/20260803T001834Z-SYS-004/PLAN.md`

Initial-plan authorization boundary: software-only evidence reconciliation. No detector,
hardware, credential, network, flash, monitor, HTTP, WebSocket, OTA, mining,
safety-control, direct-UART, or pin action is authorized.

Verification: The focused build-identity (8), system-info wire (1), package
manifest (8), and runtime-attestation (11) tests passed; the focused Bazel
targets passed; API compare checked 99 schema, 47 captured-response, and 36
static-route facts without validation errors; and the complete Rust, Bright
Builds, Bazel, parity, progress, redaction, reference-integrity, and whitespace
gates passed before the evidence commit.

Completion review: The stale checklist state was caused by evidence-accounting
drift, not a missing firmware path. The existing canonical version projection
passed focused and repository-wide checks, its evidence was committed before
the guarded one-row transition, and progress was hash-chain synchronized.
`SYS-004` is now accurately `implemented`. Residual risk is explicit:
exact-current-package live API version evidence and a decision on static-asset
version semantics remain required before `verified`, so this task stays active
and unarchived.

Continuation: The next `advance-parity` preflight incorrectly reopened this
completed implementation plan because `next-item` treats every plan without a
`RESULT.md` as open. `RESULT.md` is reserved for verified completion, so that
rule deadlocks every intentionally conservative transition to `implemented`.
This software-only continuation will make plan openness depend on whether the
authoritative checklist status still equals the plan's recorded initial status.
It does not alter `SYS-004` evidence, authorize hardware, or weaken the later
verification gate.

Continuation verification: Focused isolated-target Cargo tests passed all seven
`parity_work` cases, including unchanged-status resumption, non-verified status
advance closure, and regression rejection. The Bazel parity test target passed.
Running the newly built CLI against the real repository returned
`maybe_open_plan: null` and restored the ordered candidate queue with `SYS-004`
first at `implemented`; the checklist and progress history remain unchanged.

Continuation completion review: Commit
`2ba235e661a628a66be035f329e75d1de82da80f` contains the targeted selector fix.
All Rust, Bazel, Bright Builds, parity, redaction, and reference-integrity gates
passed. The ordinary macOS Cargo target again stalled while launching the newly
linked parity test binary, so the complete Cargo suite was rerun successfully
from the clean isolated target that had already crossed that host boundary.
No checklist transition, progress-history append, `RESULT.md`, or hardware
interaction occurred. Residual risk is limited to future plan metadata/schema
evolution; missing, invalid, and regressed states fail closed.

Verification continuation: Source inspection resolved the static-semantics
question as a concrete defect. Upstream reports the version stored in the
flashed static filesystem and compares it with the application version. Rust
currently reports the generic checked-in fallback UI name, and its SPIFFS
package contains no `version.txt`. The new immutable verification plan permits
the minimum package/runtime repair, a typed version-only evidence projection,
and one exact-package Phase 36 broker attempt after the software source is
clean, fully verified, committed, and pushed.

Hardware contract: only the five exact commands and paths listed in the
verification plan are permitted. They allow package creation, two detector
admissions (one explicit and one broker-owned), one exact-package factory
flash with local Wi-Fi NVS input, passive receive-only serial capture,
read-only same-origin HTTP/WebSocket observation, typed same-package recovery
if required, cleanup, and a redaction-safe version projection. The private
root is `scratch/sys004-version-reporting/attempt-001`; it must be mode `0700`
with mode-`0600` artifacts. The Wi-Fi file contents, USB identity, device
origin, IP/MAC/SSID, raw response, and operational paths are never printed,
summarized, or committed.

Safety, recovery, and stop contract: board 205 only; 360-second capture and
420-second effect wall-clock bounds; safe boot with mining, work submission,
and hardware control disabled; no pool input or mining/control action; and no
manual reset, erase, raw write, OTA, discovery, fault injection, direct UART,
or pin access. The broker preserves the earliest typed failure, permits only
its same-package typed recovery after a confirmed flash effect, and proves
cleanup. One attempt only, with no unchanged retry. The accepted terminal
outcomes are `complete`, `stop_repeated_boundary`, `stop_hardware_blocker`,
`stop_authority_boundary`, or `stop_impossible_contract`.

Implementation continuation: The package now copies the checked-in static
tree into an ephemeral staging directory, writes the exact canonical build
label to `version.txt`, and feeds only that staged tree to ESP-IDF
`spiffsgen.py`. The firmware reads `/www/version.txt` after the existing SPIFFS
mount and classifies absent, malformed, or noncanonical content as explicitly
unavailable. The new `project-sys004-version-evidence` classifier requires the
mode-`0600` Phase 36 handle, capture, and eligible seal; revalidates exact
manifest, package, source, reference, ELF, capability, board, ASIC, and
same-device joins; requires identical HTTP/WebSocket JSON; and emits only the
closed commit-safe version/provenance projection. Focused API/parser,
packager, source-boundary, projection, stale-version, and private-mode tests
pass. The remaining unchecked work is the mandatory full gate, clean software
commit/push, and single hardware attempt.

Attempt-001 outcome: clean source `0a4475f232cc7d944e69c6425955994bbfc12a9e`
was packaged and the standalone detector passed. The broker admitted the exact
package, but its internal board-205 detector failed before credential access or
flash, then cleanup passed. The sealed categorical record reports
`detector_failed`, `recovery_disposition: not_authorized`, no secondary
failure, no candidate, and no private capture. Root cause is deterministic:
the canonical `tools/flash detect` output uses `port: <value>`, while the
broker accepts only the nonexistent `port=<value>` spelling. This is a host
parser defect, not device evidence, and the attempt changed no device state.

Attempt-002 authorization: add a pure detector-output parser that accepts
exactly one canonical `port: ` line and rejects missing, duplicate, legacy
`port=`, empty, and invalid UTF-8 inputs. Run all mandatory gates, commit and
push the fix, and create a new clean exact package before the retry. Exactly
one retry is permitted at
`scratch/sys004-version-reporting/attempt-002` using the same 360-second
capture, 420-second effect, safety, privacy, recovery, cleanup, and stop
contract as attempt 001. The only permitted retry workflow is:

1. `just package`
2. `just detect-ultra205`
3. `just phase36-substantive-evidence mode=preflight board=205 private-parent=scratch/sys004-version-reporting/attempt-002 attempt-handle-file=scratch/sys004-version-reporting/attempt-002/handle.json candidate-output=scratch/sys004-version-reporting/attempt-002/candidate.json capture-timeout-seconds=360 package-manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`
4. `just phase36-substantive-evidence mode=hardware board=205 private-parent=scratch/sys004-version-reporting/attempt-002 attempt-handle-file=scratch/sys004-version-reporting/attempt-002/handle.json candidate-output=scratch/sys004-version-reporting/attempt-002/candidate.json capture-timeout-seconds=360 wifi-credentials=wifi-credentials.json`
5. `bazel run //tools/parity:report -- project-sys004-version-evidence --private-parent scratch/sys004-version-reporting/attempt-002 --attempt-handle-file scratch/sys004-version-reporting/attempt-002/handle.json --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --output docs/parity/evidence/sys004-version-reporting/version-projection.json`

No unchanged attempt, alternate command, or later ordinal is authorized. A
retry may proceed only after the parser regression, all gates, and clean push
prove the objectively changed boundary.

Attempt-002 outcome: clean source `9f4d56700c42a318e1aef61ee99bffcaf06e4231`
was packaged; the standalone detector and exact-package preflight passed. The
broker again stopped before credential access or flash with the same sealed
categorical detector failure, then cleanup passed with no candidate or private
capture. The first repair proved the stdout grammar but exposed the remaining
invocation cause: the broker starts nested `just detect-ultra205` without
setting its working directory to Bazel's `BUILD_WORKSPACE_DIRECTORY`, so the
process cannot reliably resolve the repository Justfile from the runfiles
working directory. The targeted follow-up sets only that command working
directory and has a pure command-construction regression. A third attempt is
not authorized. `SYS-004` must remain `implemented` with exact-package live
version evidence pending until a future explicitly authorized ordinal can
exercise the corrected broker.

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
