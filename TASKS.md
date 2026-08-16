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

Task blocks under `## Future` remain incomplete and become ordinary automatic
selection candidates after higher-priority active work. Move the selected task
to `## Active` before implementation. Dependencies, environment, verification,
safety, and evidence gates still apply; no task or fresh progress-backed
attempt ordinal requires repeated user confirmation.

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

### task-parity-api010-theme-route | 2026-08-04 | Implement theme API persistence

- [x] Model upstream theme defaults, GET projection, and POST planning with
      bounded request handling and exact public response behavior.
- [x] Register firmware `/api/theme` GET/POST handlers and persist confirmed
      theme writes through the existing serialized NVS snapshot owner.
- [x] Extend route/API comparison and golden regressions, run all mandatory
      gates, and transition only `API-010` to `implemented`.

Plan: `docs/parity/work-plans/20260804T140400Z-API-010/PLAN.md`

Authorization: public upstream theme data and pure/local software only. No
hardware, credentials, network requests, mining, controls, OTA, direct UART,
or pins.

Verification: Focused config/API, parity, API-compare, strict Clippy, and the
real ESP-IDF firmware target passed. The mandatory ordered Rust sequence,
Bright Builds checks, all 28 Bazel test targets, parity/progress, redaction,
reference cleanliness, and diff checks passed on implementation commit
`b65d19c2`.

Completion review: The bounded route and confirmed-persistence implementation
is complete, and `API-010` is `implemented` with `unit,golden,api-compare`
evidence. The task remains active and unarchived because detector-gated live
route/reboot durability and installed AxeOS browser behavior remain unverified.
No hardware, credentials, mining, controls, OTA, direct UART, or pins were used.

### task-parity-io002-adc-observation | 2026-08-04 | Implement calibrated ADC observation

- [x] Add a pure stamped core-voltage acquisition path with explicit unavailable,
      stale, and fault truth.
- [x] Own ESP32-S3 ADC1 channel 1 on GPIO2 through the ESP-IDF oneshot curve-
      calibrated adapter and the sole operator sensor producer.
- [x] Project only fresh values to `coreVoltageActual`, add focused regressions,
      build firmware, run mandatory gates, and transition only `IO-002` to
      `implemented`.

Plan: `docs/parity/work-plans/20260804T140933Z-IO-002/PLAN.md`

Authorization: read-only local software work. No hardware, credentials,
network requests, mining, voltage/fan/power effects, OTA, direct UART, or pins.

Verification: Four focused reducer tests, public fresh/stale/fault projection
tests, source-ownership guards, and the real ESP32-S3 release build passed. The
mandatory ordered Rust sequence, Bright Builds, all 29 Bazel test targets,
parity/progress, redaction, reference cleanliness, and diff checks passed on
implementation commit `4d7c8486`.

Completion review: The exact read-only ADC adapter, stamped producer truth, and
fresh-only `coreVoltageActual` projection are implemented without widening the
mining or hardware-effect gates. The task remains active and unarchived because
live ADC calibration accuracy, millivolt values, cadence, failure behavior, and
API correlation remain below verified and need a separately task-gated detector
capture. No hardware, credentials, mining, controls, OTA, UART, or pins ran.

Verification continuation plan:
`docs/parity/work-plans/20260815T210711Z-IO-002/PLAN.md`

- [x] Add the closed `bitaxe-adc-observation-evidence-v1` contract,
      independent validator, and repo-owned capture command without changing
      production ADC, safety, mining, or control behavior.
- [x] After the implementation is committed, pushed, clean, and packaged, run
      only detector-gated IO-002 `attempt-001` and publish evidence atomically
      only after independent validation and redaction pass.
- [x] Promote only IO-002 on the complete exact-package ADC/API quorum;
      otherwise preserve `implemented`, record the earliest typed blocker and
      accepted stop outcome, and do not retry.

Hardware contract: exact objective, scope, evidence/privacy rules, allowed and
prohibited effects, recovery, retry, and promotion criteria are immutable in
the continuation plan above. The only authorized device sequence is its exact
`just detect-ultra205` command followed, on successful single-device admission
and local Wi-Fi credential availability, by its exact one-shot
`just capture-adc-observation-evidence ... --capture-timeout-seconds 360`
command. Attempt-001 permits one exact-package factory flash/reset, protected
read-only same-origin observation, bounded cleanup, and at most one exact-
package recovery flash after a post-flash failure. It prohibits settings or
restart requests, mining, pool input, ASIC work, voltage/frequency/fan/power
control, raw ADC/GPIO/I2C, OTA, erase, fault injection, physical power actions,
direct UART, and all pin/pad/header/probe/jumper/solder/signal manipulation.
The attempt is consumed when the capture command starts; no unchanged retry or
attempt-002 is authorized. Detector ambiguity/failure, missing credentials,
unsafe state, cleanup or recovery failure, incomplete proof, or privacy failure
stops the run and withholds promotion.

Attempt-001 verification: `blocked` at the closed `evidence_invalid` boundary.
Exact clean pushed package `0bd2dfff` admitted one Ultra 205, observed the safe
boot, kept mining and hardware control disabled, completed cleanup, and passed
the independent system-info and redaction contracts. The lossless ADC input
validator rejected both fresh live samples because they were outside the
immutable 400–2,000 mV admission range. No public projection was published,
no retry ran, and IO-002 remains `implemented`. See
`docs/parity/work-plans/20260815T210711Z-IO-002/CLOSURE.md`.

Completion review: The typed evidence workflow is implemented and verified in
software, but live ADC verification remains unresolved. The plan did not prove
that a passive safe-state core rail must exceed 400 mV, so weakening the range
after observing the device would be circular. A fresh task and immutable plan
must establish the expected disabled-rail reading from authoritative electrical
evidence or separately authorized safe stimulus before any retry eligibility.
Residual gaps remain live calibration accuracy, values, cadence under load,
failure behavior, exact numeric API correlation, and energized-rail behavior.

Corrective software work (2026-08-15):

- [x] Bind the lossless ADC input validator to the independently validated
      disabled mining and hardware-control state instead of applying an
      energized-rail range to every observation.
- [x] Admit the calibrated adapter's actual `u16` millivolt wire domain,
      including a fresh zero sample, while preserving freshness, boot-session,
      sequence, acquisition-time, and equal-sequence coherence checks.
- [x] Replace the misleading positive/plausible-range evidence claims with
      nonnegative wire-domain and disabled-state-bound claims; add Rust and
      real child-process regressions and run the mandatory software gates.

Authorization: local software, contract, test, and task-record changes only.
Do not run hardware, consume another attempt ordinal, alter the closed plan or
closure, publish the protected failed attempt, or promote IO-002.

Correction verification: The focused regression first reproduced the exact
`ADC observation values are outside the admitted range` failure for fresh
zero-millivolt samples. Eight focused Rust validator tests, the zero-value
TypeScript orchestration fixture, the real child-process handoff, the original
protected input replay, the ordered Rust checks, Bright Builds, the real
ESP32-S3 firmware build, all 45 Bazel tests, parity/progress, redaction,
reference cleanliness, and diff checks then passed.

Correction review: ADC/API units remain millivolts. The validator now consumes
the already-independent system-info evidence, requires its disabled mining and
hardware-control state, and admits only finite integral values in the producer's
`u16` millivolt domain. Its public aggregate claims no longer imply positive or
energized-range voltage. No hardware ran, no evidence was published, the closed
attempt and closure were not rewritten, and IO-002 remains `implemented`.
Energized-rail target/tolerance proof and a new hardware attempt still require a
fresh immutable plan.

### task-parity-net002-provisioning-network | 2026-08-04 | Implement configuration AP and captive DNS

- [x] Add a pure bounded wildcard IN/A captive-DNS response contract matching
      the pinned configuration-network behavior.
- [x] Configure the firmware AP-only/mixed-mode lifecycle and a single UDP/53
      owner without exposing credentials or network identifiers.
- [x] Add focused and ownership regressions, build the real firmware, run all
      mandatory gates, and transition only `NET-002` to `implemented`.

Plan: `docs/parity/work-plans/20260804T160000Z-NET-002/PLAN.md`

Authorization: local software and build work only. No hardware attempt,
credentials, external network request, mining, voltage/fan/power effect, OTA,
recovery, direct UART, or pins.

Verification: Eight focused DNS/SSID tests, four firmware source-ownership
tests, the real ESP32-S3 firmware build, the ordered Rust gate, Bright Builds,
all 30 Bazel test targets, parity/progress, redaction, reference cleanliness,
and diff checks pass against the implementation tree.

Completion review: The bounded configuration AP and captive-DNS implementation
is complete, and `NET-002` is `implemented` with `unit,workflow` evidence under
transition `20260804T163500Z-NET-002`. This task remains active and unarchived
because live SSID visibility, client association, DHCP, wildcard DNS, captive
redirect, settings access, station handoff, and fallback behavior remain below
verified and require separate detector-gated evidence.

### task-parity-bap002-protocol | 2026-08-04 | Implement the pure BAP protocol core

- [x] Add exact bounded BAP command/parameter framing, checksum, parsing, and
      compatibility admission.
- [x] Add duplicate suppression plus pure request, subscription, and setting
      decisions without UART, persistence, restart, or hardware effects.
- [x] Add synthetic golden regressions, run every mandatory gate, and transition
      only `BAP-002` to `implemented`.

Plan: `docs/parity/work-plans/20260804T180000Z-BAP-002/PLAN.md`

Authorization: local pure software and build work only. No accessory, hardware
attempt, credentials, external request, UART, pins, persistence, restart,
mining, ASIC traffic, frequency/voltage/fan/power effect, OTA, or recovery.

Verification: Twelve focused Cargo BAP tests and the `bitaxe-core` Bazel target
pass, including synthetic golden frames, checksum compatibility, malformed
input categories, exact request projections, AP errors, setting decisions, and
diagnostic redaction. The ordered Rust sequence, Bright Builds, all 30 Bazel
tests, parity/progress, redaction, reference cleanliness, sensitive-value
review, file-size review, and diff checks also pass.

Completion review: The pure protocol implementation is complete, and `BAP-002`
is `implemented` with `unit,golden` evidence under transition
`20260804T185000Z-BAP-002`. This task remains active and unarchived because
`BAP-001` continues to own the firmware UART and task lifecycle, and live
accessory interoperability remains below verified.

### task-parity-ui004-operator-workflows | 2026-08-04 | Implement scoped AxeOS operator workflows

- [x] Add independent responsive navigation plus dashboard, network, pool,
      settings, logs, update, and theme pages against existing API contracts.
- [x] Keep credentials write-only, require confirmation for restart/update, and
      preserve the fail-closed OTAWWW and hardware-control boundaries.
- [x] Add SPA route, pure UI, static-contract, and real-browser regressions;
      run every mandatory gate and transition only `UI-004` to `implemented`.

Plan: `docs/parity/work-plans/20260804T190000Z-UI-004/PLAN.md`

Authorization: local static UI, synthetic same-origin API fixture, real local
browser, and build work only. No device hardware attempt, real credentials,
external service, mining, ASIC traffic, frequency/voltage/fan/power effect,
firmware upload, OTAWWW, recovery, direct UART, or pins.

Verification: Nine focused static-route tests, the complete `bitaxe-api` and
automation targets, pure/static UI contracts, headed Playwright desktop/mobile
workflows with a clean console, the ordered Rust sequence, Bright Builds, all
30 Bazel tests and the ESP32-S3 artifact, parity/progress, redaction, reference
cleanliness, deterministic gzip, sensitive-value, provenance, and diff checks
pass against implementation commit `89564440`.

Completion review: The scoped operator interface is complete, and `UI-004` is
`implemented` with `unit,workflow,static-route` evidence under transition
`20260804T195000Z-UI-004`. This task remains active and unarchived because live
embedded serving, real device mutation, upload/reboot, OTAWWW, scoreboard/swarm
population, and responsive operator UAT remain below verified.

### task-parity-stat001-hashrate-monitor | 2026-08-04 | Implement the hashrate monitor

- [x] Add exact bounded counter conversion, reset behavior, error percentage,
      and hierarchical 1-minute, 10-minute, and 1-hour averages.
- [x] Carry parsed register values through the sole production owner and admit
      passive reads only while its ASIC session is already active.
- [x] Publish all hashrate windows through the existing runtime/API projection,
      add focused ownership regressions, and run every mandatory gate.

Plan: `docs/parity/work-plans/20260804T200000Z-STAT-001/PLAN.md`

Authorization: local software, synthetic register observations, and build work
only. No hardware attempt, credentials, external service, mining campaign,
pool connection, frequency/voltage/fan/power effect, OTA, recovery, direct UART,
or pins.

Verification: The full Rust sequence, Bright Builds checks, all 31 Bazel tests,
the real firmware build, parity validation/progress, redaction, reference
cleanliness, and diff checks passed on the implementation tree.

Completion review: Implementation commit
`e0c3b1e9043e033b24135b31a1293bf22afe8759` and typed transition
`20260804T205500Z-STAT-001` establish `implemented` with `unit,workflow`
evidence. The task remains active because live BM1366 counter accuracy and
hardware/API/UI behavior remain below verified.

Verification-promotion plan:
`docs/parity/work-plans/20260816T005443Z-STAT-001/PLAN.md`

- [x] Add a private-first closed hashrate quorum to the existing conservative
      600-second campaign, with independent validation and no raw-value
      publication.
- [x] Run the plan's full pre-hardware suite, commit and push the implementation,
      then execute only detector command 1 and the sole attempt-001 command 2.
- [ ] Promote only STAT-001 on the complete exact-package HTTP/WebSocket,
      BM1366 topology, rolling-window, terminal-zero, safe-stop, cleanup, mode,
      seal, and redaction quorum; otherwise preserve `implemented`, withhold the
      projection, record the earliest typed blocker, and do not retry.

Attempt-001 authorization: one exact clean board-205 package may be factory-
flashed and normally reset/re-enumerated; ignored local Wi-Fi and pool inputs
may be seeded privately; and the repo-owned conservative 400 MHz / 1100 mV /
100% fan profile may mine for exactly 600 active seconds while the current
session's serial-derived origin is observed through HTTP and WebSocket. The
campaign must pause, safe-stop, clean up USB ownership, and may use at most one
exact-package recovery flash after a post-flash failure. No upstream-default or
overclock profile, arbitrary control target, unbounded mining, OTA, erase,
fault injection, physical power action, direct UART, or electrical pin/pad/
header/probe/jumper/solder/signal manipulation is authorized.

Evidence/privacy/retry: `scratch/stat001-hashrate-monitor/wrapper-001` and
`attempt-001` are ignored mode-`0700` roots with mode-`0600` files. Credentials,
pool/owner/worker fields, origins, ports, USB/network identity, exact hashrates,
sensors, HTTP/WebSocket bodies, serial, commands, PIDs, and traces stay private.
Only the plan-named closed projection may become public after independent
validation. Starting command 2 consumes attempt-001. Every post-flash failure
preserves the earliest typed category and performs bounded safe stop, recovery,
and cleanup. Detector ambiguity/failure, missing inputs, unsafe state,
malformed/incomplete proof, cleanup/recovery/privacy failure, or nonzero command
stops without retry and leaves STAT-001 `implemented`.

Attempt-001 outcome: detector command 1 admitted exactly one Ultra 205. Capture
command 2 then failed its immutable pre-effect source/reference admission with
typed category `evidence_invalid`; the broad `update_hash_counter` fragment had
eight legitimate upstream occurrences while the wrapper required one. No
attempt root, flash, mining, or public projection was created. Attempt-001 is
consumed, no retry was run, and STAT-001 remains `implemented`. The faulty
fragment and a second ambiguous source fragment were narrowed and a current-
repository admission regression was added after the failure. See
`docs/parity/work-plans/20260816T005443Z-STAT-001/CLOSURE.md`.

Next safe action: a future immutable STAT-001 plan may authorize fresh
attempt-002 after the admission correction is committed, pushed, fully gated,
and bound to a newly built exact package. It must repeat detector admission and
may not reuse attempt-001 or infer hardware evidence from this pre-effect
failure.

Attempt-002 verification-promotion plan:
`docs/parity/work-plans/20260816T020135Z-STAT-001/PLAN.md`

- [x] Rebind the closed `bitaxe-hashrate-monitor-evidence-v1` workflow,
      independent validator, generated contract, current task/plan admission,
      and protected paths to fresh attempt-002 without changing production
      hashrate behavior or the evidence quorum.
- [x] Run every focused and mandatory software, firmware, privacy, reference,
      package, and exact-source gate; commit and push before device access.
- [x] Run only the exact plan detector and one conditional attempt-002 command,
      then promote STAT-001 only if the complete independently validated
      exact-package hashrate quorum passes.

Attempt-002 authorization: after the immutable plan/task checkpoint and exact
implementation are clean, fully gated, committed, and pushed, one exact
board-205 package may be factory-flashed/reset; ignored local Wi-Fi and pool
credentials may be seeded privately; and the repo-owned conservative 400 MHz /
1100 mV / 100% fan profile may mine for exactly 600 accumulated active seconds
while protected current-session HTTP, WebSocket, and serial observations are
joined. The campaign must pause, safe-stop, clean up USB/process ownership, and
may use at most one exact-package recovery flash after a post-flash failure.

Attempt-002 evidence, privacy, recovery, and retry: only fresh ignored mode-
`0700` `scratch/stat001-hashrate-monitor/wrapper-002` and `attempt-002` roots
with mode-`0600` files are permitted. Credentials, pool/owner/worker fields,
origins, ports, USB/network/process identities, exact hashrates, sensors,
HTTP/WebSocket bodies, serial, commands, PIDs, and traces remain private. The
sole public aggregate projection is written only after independent validation.
Starting the capture consumes attempt-002; preserve the earliest typed failure,
run bounded safe stop/recovery/cleanup after post-flash failures, and do not run
an unchanged retry or attempt-003. Detector ambiguity/failure, missing inputs,
unsafe state, malformed/incomplete proof, cleanup/recovery/privacy failure, or
nonzero command stops with STAT-001 `implemented` and evidence withheld.

Attempt-002 prohibited effects and acceptance: no upstream-default/overclock
profile, arbitrary control target, unbounded mining, OTA, erase, fault
injection, physical power action, direct UART, or pin/pad/header/probe/jumper/
solder/signal manipulation is permitted. Promotion requires the immutable
plan's exact clean source/reference/package and detector identity; one ASIC,
four domains, one-second cadence and pinned register semantics; twenty active
windows with work renewal; changing coherent positive HTTP/WebSocket current
hashrate and positive rolling windows after warmup; bounded error; terminal
zero current rate; safe stop, cleanup, protected modes, seal, independent
validation, and redaction. Any missing fact withholds promotion.

Attempt-002 completion review: the fresh detector admitted exactly one Ultra
205, but the sole capture stopped with closed category `hardware_blocked` and
a sealed `admission_failed` result before package admission or campaign USB
execution. Source inspection found that the wrapper paired `soak` with
`conservative`, although campaign admission permits the conservative
600-second profile only for `live-share`. The wrapper and acceptance boundary
now use `live-share`, and the real-child regression rejects the former invalid
stage/profile pair. No public projection exists, attempt-002 was not retried,
and STAT-001 remains `implemented`. A future immutable plan may authorize
fresh attempt-003 only from the committed, pushed, fully gated correction and
a newly built exact package. See
`docs/parity/work-plans/20260816T020135Z-STAT-001/CLOSURE.md`.

Attempt-003 verification-promotion plan:
`docs/parity/work-plans/20260816T022946Z-STAT-001/PLAN.md`

- [x] Rebind `bitaxe-hashrate-monitor-evidence-v1`, the independent validator,
      generated contract, task/plan admission, Bazel runfiles, and protected
      paths to fresh attempt-003.
- [x] Prove the corrected `live-share` plus `conservative` child command and
      reject the former `soak` plus `conservative` pair at the real boundary.
- [x] Pass every focused and mandatory software, firmware, privacy, reference,
      package, and exact-source gate; commit and push before device access.
- [ ] Run only the exact plan detector and one conditional attempt-003 command,
      then promote only on the complete independently validated quorum.

Attempt-003 authorization: pushed commit `0d058a66` materially fixes the exact
attempt-002 pre-package campaign-admission boundary and regression-guards the
real child command. After this immutable task/plan checkpoint and the rebound
implementation are clean, fully gated, committed, and pushed, one exact board-
205 package may be factory-flashed/reset; ignored local Wi-Fi and pool inputs
may be seeded privately; and the repo-owned `live-share` conservative 400 MHz /
1100 mV / 100% fan profile may mine for exactly 600 accumulated active seconds
while protected current-session HTTP, WebSocket, and serial observations are
joined. The campaign must pause, safe-stop, clean up USB/process ownership, and
may use at most one exact-package recovery flash after a post-flash failure.

Attempt-003 evidence, privacy, recovery, and retry: use only fresh ignored
mode-`0700` `scratch/stat001-hashrate-monitor/wrapper-003` and `attempt-003`
roots with mode-`0600` files. Credentials, pool/owner/worker fields, origins,
ports, USB/network/process identities, exact hashrates, sensors, HTTP/WebSocket
bodies, serial, commands, PIDs, and traces remain private. The sole public
aggregate projection is written only after independent validation. Starting
the capture consumes attempt-003; preserve the earliest typed failure, run
bounded safe stop/recovery/cleanup after post-flash failures, and do not run an
unchanged retry or attempt-004. Detector ambiguity/failure, missing inputs,
unsafe state, malformed/incomplete proof, cleanup/recovery/privacy failure, or
nonzero command stops with STAT-001 `implemented` and evidence withheld.

Attempt-003 prohibited effects and acceptance: no upstream-default/overclock
profile, arbitrary control target, unbounded mining, OTA, erase, fault
injection, physical power action, direct UART, or pin/pad/header/probe/jumper/
solder/signal manipulation is permitted. Promotion requires the linked plan's
exact clean source/reference/package and detector identity; one ASIC, four
domains, one-second cadence and pinned register semantics; twenty active
windows with work renewal; changing coherent positive HTTP/WebSocket current
hashrate and positive rolling windows after warmup; bounded error; terminal
zero current rate; safe stop, cleanup, protected modes, seal, independent
validation, and redaction. Any missing fact withholds promotion.

### task-parity-stat002-statistics-history | 2026-08-04 | Implement production statistics history

- [x] Add the exact bounded 720-sample history, timestamp admission, configured
      retention decision, zero-frequency clearing, and focused regressions.
- [x] Start one absolute-cadence firmware producer that records confirmed
      runtime snapshots independently of HTTP request timing.
- [x] Return the complete owned history through the existing API projection,
      prove sole ownership and request-time immutability, and run every gate.

Plan: `docs/parity/work-plans/20260804T211000Z-STAT-002/PLAN.md`

Authorization: local software, synthetic snapshots, and build work only. No
hardware attempt, credentials, external service, mining campaign, pool
connection, frequency/voltage/fan/power effect, OTA, recovery, direct UART, or
pins.

Verification: The focused Cargo and Bazel tests, real firmware build, ordered
Rust format/Clippy/build/test sequence, Bright Builds checks, all 32 Bazel test
targets, parity validation/progress, redaction, reference cleanliness, and diff
checks passed on the implementation tree.

Completion review: Implementation commit
`35f8bb676b91bdb702dd9026cb0379f5b12e45e6` and typed transition
`20260804T215500Z-STAT-002` establish `implemented` with
`unit,workflow,api-compare` evidence. The task remains active because live
cadence, telemetry accuracy, long-duration retention, device API, and browser
behavior remain below verified.

### task-parity-stat003-scoreboard | 2026-08-04 | Implement production scoreboard

- [x] Add the exact stable top-20 valid-nonce scoreboard and bounded indexed
      persistence codec with focused regression coverage.
- [x] Carry one redacted candidate from current-generation nonce correlation
      through a typed production-session effect without changing submit policy.
- [x] Add transactional indexed-NVS ownership, boot load, read-only API
      projection, production ownership tests, and every mandatory gate.

Plan: `docs/parity/work-plans/20260804T220000Z-STAT-003/PLAN.md`

Authorization: local software, synthetic nonce/work fixtures, and build work
only. No hardware attempt, credentials, external service, mining campaign, pool
connection, frequency/voltage/fan/power effect, OTA, recovery, direct UART, or
pins.

Verification: Ten focused API tests, three production-session tests, five
firmware ownership tests, the complete Cargo suite, real firmware build, Bright
Builds checks, all 33 Bazel tests, parity validation/progress, redaction,
reference cleanliness, and diff checks passed on the implementation tree.

Completion review: Software implementation is complete at
`0f3d46a77f5b2492880921cf524bc052d2283bc4` and the typed transition
`20260804T225500Z-STAT-003` records `implemented` with
`unit,workflow,api-compare`. Live nonce difficulty, device persistence, API and
browser behavior, mining, and accepted/rejected share outcomes remain below
verified, so this task remains active rather than archived.

### task-parity-ui001-display-behavior | 2026-08-04 | Complete Ultra 205 display driver behavior

- [x] Add the pure exact-panel, rotation, inversion, timeout, wake/priority,
      and edge-triggered power contract with boundary regressions.
- [x] Load confirmed display settings with upstream defaults and fail closed on
      malformed or unsupported stored values.
- [x] Retain one configured firmware display owner across runtime frames and
      prove configuration/render/power ordering plus sensor-failure isolation.

Plan: `docs/parity/work-plans/20260804T230000Z-UI-001/PLAN.md`

Authorization: local software, synthetic display/settings fixtures, and build
work only. No hardware attempt, credentials, external service, mining, pool
connection, frequency/voltage/fan/power effect, OTA, recovery, direct UART,
pins, or physical button interaction.

Verification: Focused core/config tests, display adapter and ownership tests,
the canonical firmware build, the mandatory Rust sequence, Bright Builds,
`just test`, parity/progress, redaction, reference cleanliness, immutable-plan,
and diff checks pass on the implementation tree.

Completion review: The exact panel settings and runtime power behavior are
software-implemented with closed configuration handling and one logical owner.
UI-002 carousel content, UI-003 physical input, live panel
orientation/inversion/timeout, and operator-visible behavior remain below
verified, so this implemented task remains active rather than archived.

### task-parity-ui002-screen-flow | 2026-08-04 | Implement bounded Ultra 205 screen flow

- [x] Add the pure priority, overlay, intro, carousel, notification, and
      four-line frame contract with exact timing and privacy regressions.
- [x] Project existing firmware runtime facts without operator-publication,
      statistics-drain, retained-log, mining-state, or credential side effects.
- [x] Retain one screen owner beside the display owner, use the absolute 500 ms
      cadence, redraw only changed frames, and preserve sensor isolation.

Plan: `docs/parity/work-plans/20260805T001000Z-UI-002/PLAN.md`

Authorization: local software, synthetic screen/runtime fixtures, and build
work only. No hardware attempt, credentials, external service, mining, pool
connection, frequency/voltage/fan/power effect, OTA, recovery, direct UART,
pins, or physical button interaction.

Verification: Twelve focused pure screen-flow tests, both firmware display
adapter/source-ownership targets, and the real ESP-IDF firmware Bazel target
pass. The ordered full Rust sequence, Bright Builds checks, all 34 Bazel test
targets, parity validation/progress, redaction, reference cleanliness,
immutable-plan, and diff checks also pass.

Completion review: Implementation commit
`9b2f37945b34a0e9fece56c8aa90703afda3ac63` and the commit-bound `RESULT.md`
support the typed `implemented` transition with `unit,workflow` evidence. The
task remains active because UI-002 is below `verified`; UI-003 physical input,
live screen content, animation/bitmap parity, mining, and hardware-control
behavior remain separate evidence gaps.

### task-parity-ui003-boot-button | 2026-08-04 | Implement bounded Ultra 205 boot-button input

- [x] Add a pure active-low debounce and exact 2,000 ms short/long press
      classifier with bounce, regression, and one-shot long-press coverage.
- [x] Retain one GPIO0 input owner and route normal short clicks to identify
      cancellation or screen advance, normal long presses to configuration-AP
      toggle, and self-test long presses to an explicit unavailable boundary.
- [x] Wake the display on admitted short input, preserve display/sensor/Wi-Fi
      failure isolation, and expose only redaction-safe input status categories.

Plan: `docs/parity/work-plans/20260805T020000Z-UI-003/PLAN.md`

Authorization: local software, synthetic input/runtime fixtures, and build
work only. No hardware attempt, physical button press, credentials, external
service, mining, pool connection, frequency/voltage/fan/power effect, OTA,
recovery, direct UART, pins, or physical electrical manipulation.

Verification: Six focused pure input tests, fourteen screen-flow tests, both
firmware display/Wi-Fi source-ownership targets, and the real ESP32-S3 firmware
target pass. The mandatory ordered Rust sequence, Bright Builds checks, all 34
Bazel test targets, parity validation/progress, redaction, reference
cleanliness, immutable-plan, sensitive-log, and diff checks also pass.

Completion review: The bounded active-low classifier, retained GPIO0 pull-up
owner, atomic identify cancellation, manual screen advance/display wake, and
typed configuration-AP toggle are software-implemented with closed failure
categories. Physical button observation, exact LVGL event timing, self-test
cancellation, live configuration-AP toggling, and all hardware behavior remain
below verified, so this implemented task remains active rather than archived.

### task-parity-api010-live-theme-durability | 2026-08-04 | Verify live theme route durability

- [x] Add a typed private-first `/api/theme` capture that binds the exact
      package, admitted Ultra 205, one normal software restart, and restored
      original appearance settings.
- [x] Require live GET/POST/readback, same-device reboot identity, post-restart
      persistence, exact restoration, cleanup, and a closed redacted public
      projection before emitting evidence.
- [x] Add unit and real-child-process regressions, run every mandatory gate,
      push the clean implementation, and execute at most one `attempt-001`.
- [x] Record the first typed terminal category and stop `attempt-001` without
      an unchanged retry when its pre-effect detector transcript fails privacy
      admission.

Plan: `docs/parity/work-plans/20260804T185605Z-API-010/PLAN.md`

Authorization: standing task authorization permits exactly these effectful
commands after the implementation is clean and pushed:

1. `just package`
2. `mkdir -p scratch/api010-theme-durability/detector-001 && just detect-ultra205 2>&1 | tee scratch/api010-theme-durability/detector-001/detector.stdout`
3. `just verify-theme-durability --private-root scratch/api010-theme-durability/attempt-001 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/api010-theme-durability/detector-001/detector.stdout --projection docs/parity/evidence/api010-theme-durability/theme-durability-projection.json --capture-timeout-seconds 360`

The capture may POST one generated non-secret theme value, perform one normal
software restart through the canonical device-session transaction, restore the
exact original theme, and use an exact-package recovery flash only if normal
restoration cannot be confirmed. It may not change Wi-Fi or pool credentials,
mining, voltage, frequency, fan, thermal, power, ASIC, display-input, OTA, raw
partitions, direct UART, pins, or other electrical state.

Evidence and privacy: `scratch/api010-theme-durability/attempt-001` is an
absent-before-use mode-`0700` private root whose files are mode `0600`; detector
output remains private. The committed projection may contain only schema and
cryptographic identities, bounded counts/categories, the closed device-session
projection, and safe booleans. It must never contain origins, theme values,
hostnames, ports, USB identities, network identifiers, credentials, raw HTTP,
serial, or process traces.

Recovery, retry, and stop: preserve the earliest typed failure; attempt normal
POST restoration and exact readback first, then use the exact admitted package
for one recovery flash if required. Recovery is secondary and public only as
safe booleans. `attempt-001` is the sole authorized invocation. Stop on any
detector failure, launch failure, timeout, malformed evidence, non-ready device
session, restoration uncertainty, cleanup failure, privacy failure, or safety
invariant violation. No unchanged retry is authorized.

Verification: Focused generated-contract and automation tests pass, including
the real child-process transaction. The ordered Rust checks, Bright Builds,
all 34 Bazel test targets including the real firmware build, parity/progress,
redaction, reference cleanliness, immutable-plan, and diff checks pass. The
first `just parity` launch encountered transient host resource exhaustion after
all tests passed; the bounded rerun completed with `validation_errors: none`.

Completion review: `attempt-001` stopped before package admission, flashing,
HTTP, restart, or device mutation because the task-recorded `tee` command
created the detector transcript as mode `0644`, while the verifier correctly
requires mode `0600`. The earliest category is `process_failed`; no public
evidence was emitted and no recovery was required. A separately task-gated
`attempt-002` may remediate only that objectively confirmed file-mode defect.

### task-parity-api010-live-theme-durability-retry | 2026-08-04 | Retry live theme durability after detector-mode remediation

- [x] Preserve the pushed implementation and immutable plan; create a fresh
      detector transcript as mode `0600` before capture.
- [x] Run exactly one detector-gated `attempt-002` with the same bounded theme,
      reboot, restoration, recovery, privacy, and safety contract.
- [x] Record the earliest typed terminal category, withhold evidence, keep
      `API-010` at `implemented`, and stop without another retry when the
      baseline trace contains multiple boot sessions.

Plan: `docs/parity/work-plans/20260804T185605Z-API-010/PLAN.md`

Objective remediation: `attempt-001` produced no device effects and failed
only because its detector transcript was mode `0644`. This retry changes only
the scratch transcript setup: it creates the fresh file and applies mode
`0600` before `tee`, which truncates the file without changing its mode.

Authorization: standing task authorization permits exactly these effectful
commands after this retry contract is clean and pushed:

1. `just package`
2. `mkdir -p scratch/api010-theme-durability/detector-002 && touch scratch/api010-theme-durability/detector-002/detector.stdout && chmod 600 scratch/api010-theme-durability/detector-002/detector.stdout && just detect-ultra205 2>&1 | tee scratch/api010-theme-durability/detector-002/detector.stdout`
3. `just verify-theme-durability --private-root scratch/api010-theme-durability/attempt-002 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/api010-theme-durability/detector-002/detector.stdout --projection docs/parity/evidence/api010-theme-durability/theme-durability-projection.json --capture-timeout-seconds 360`

The capture may POST one generated non-secret theme value, perform one normal
software restart through the canonical device-session transaction, restore the
exact original theme, and use an exact-package recovery flash only if normal
restoration cannot be confirmed. It may not change Wi-Fi or pool credentials,
mining, voltage, frequency, fan, thermal, power, ASIC, display-input, OTA, raw
partitions, direct UART, pins, or other electrical state.

Evidence and privacy: `scratch/api010-theme-durability/attempt-002` is an
absent-before-use mode-`0700` private root whose files are mode `0600`; detector
output remains private and must be mode `0600` before capture. The committed
projection may contain only schema and cryptographic identities, bounded
counts/categories, the closed device-session projection, and safe booleans.
It must never contain origins, theme values, hostnames, ports, USB identities,
network identifiers, credentials, raw HTTP, serial, or process traces.

Recovery, retry, and stop: preserve the earliest typed failure; attempt normal
POST restoration and exact readback first, then use the exact admitted package
for one recovery flash if required. Recovery is secondary and public only as
safe booleans. `attempt-002` is the sole authorized retry. Stop on any detector
failure, launch failure, timeout, malformed evidence, non-ready device session,
restoration uncertainty, cleanup failure, privacy failure, or safety invariant
violation. No further retry is authorized.

Verification: The complete ordered Rust sequence, Bright Builds, all 34 Bazel
tests, parity/progress, redaction, reference cleanliness, immutable-plan, and
diff checks passed before the retry contract was pushed as `8e95e5a6`. The
fresh detector passed for exactly one Ultra 205 and its transcript was mode
`0600`. A read-only classifier recheck reproduced the closed
`baseline_multiple_sessions` category from the private initial trace. All
private attempt directories are mode `0700`, all files are mode `0600`, and no
public projection exists.

Completion review: `attempt-002` used the exact pushed package and stopped with
earliest category `evidence_invalid` because the initial production-shaped
flash-monitor trace classified as `baseline_multiple_sessions`. The failure
occurred before hostname/theme GET or POST, software restart, or settings
mutation. The authorized exact-package flash was the sole device effect, and
no recovery was required. No public evidence was emitted, `API-010` remains
`implemented`, and no further hardware retry is authorized. The remaining gap
was a host-side baseline-epoch admission defect; its completed software fix is
recorded under `task-parity-api010-baseline-epoch-admission` in
`TASKS.archive.md`. A new task-gated hardware contract is still required before
any live retry.

### task-parity-api010-live-theme-durability-attempt-003 | 2026-08-04 | Run post-fix live theme durability

- [x] Preserve the pushed terminal-epoch and selector fixes, freeze the exact
      planning-commit package, and privately admit exactly one Ultra 205.
- [x] Run exactly one bounded `attempt-003`; the transaction stopped at the
      initial exact-package flash-monitor child before baseline classification,
      theme mutation, restart, persistence, or restoration.
- [x] Promote only `API-010` on complete typed evidence; otherwise record the
      earliest terminal category, withhold evidence, and stop without retry.

Plan: `docs/parity/work-plans/20260804T200849Z-API-010/PLAN.md`. This immutable
plan continues from
`docs/parity/work-plans/20260804T192918Z-API-010/PLAN.md` after the verified
terminal-baseline fix.

Objective and progress basis: `attempt-002` stopped at
`baseline_multiple_sessions` before theme mutation or restart. Commit
`67974ccc` fixes that exact production-shaped classifier boundary through a
real child process, and commit `053410ff` makes the explicit plan lineage
deterministically selectable. This is a progress-backed fresh ordinal, not an
unchanged retry.

Authorized commands after this task and plan are clean, verified, committed,
and pushed:

1. `just package`
2. `test ! -e scratch/api010-theme-durability/wrapper-003 && (umask 077; mkdir -m 700 scratch/api010-theme-durability/wrapper-003 && just detect-ultra205 > scratch/api010-theme-durability/wrapper-003/detector.stdout 2>&1)`
3. `test ! -e scratch/api010-theme-durability/attempt-003 && test ! -e docs/parity/evidence/api010-theme-durability/theme-durability-projection.json && (umask 077; just verify-theme-durability --private-root scratch/api010-theme-durability/attempt-003 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/api010-theme-durability/wrapper-003/detector.stdout --projection docs/parity/evidence/api010-theme-durability/theme-durability-projection.json --capture-timeout-seconds 360 > scratch/api010-theme-durability/wrapper-003/verify.stdout 2> scratch/api010-theme-durability/wrapper-003/verify.stderr)`

Hardware and effect boundary: detector admission must find exactly one likely
ESP USB serial port and successful ESP32-S3 board info. The capture may use the
admitted port for one exact-package flash-monitor transaction, one generated
non-secret theme mutation, one normal software restart, exact readbacks and
restoration, and at most one built-in exact-package recovery flash if normal
restoration cannot be confirmed. It may not change Wi-Fi or pool configuration,
mine, enable ASIC work, change voltage/frequency/fan/thermal/power controls,
exercise display input, perform OTA or raw partition writes, discover network
targets, terminate foreign processes, use direct UART or pins, or perform any
other electrical action.

Evidence and privacy: the supervisor must exclusively create the absent
`scratch/api010-theme-durability/attempt-003` child as a mode-`0700`
`ProtectedOperational` root with mode-`0600` files. The caller-owned
`scratch/api010-theme-durability/wrapper-003` sibling is mode `0700`; its
detector and stdout/stderr files are mode `0600`. Credential contents are
`NeverPersistRaw` and may not reach disk or terminal. The committed projection
may contain only its closed schema, public provenance, bounded categories and
counts, the closed device-session projection, and safe booleans—never origins,
theme values, hostnames, ports, USB/network identifiers, credentials, raw
HTTP/serial/process material, or private paths.

Recovery, retry, and stop: preserve the earliest typed failure. Normal exact
theme restoration and readback precede the workflow's single exact-package
recovery fallback; recovery remains secondary and public only as safe booleans.
`attempt-003` is the sole authorized capture. If the fixed
`baseline_multiple_sessions` signature recurs, select
`stop_repeated_boundary`. Any detector failure, launch failure, timeout,
malformed/missing projection, non-ready device session, persistence mismatch,
restoration uncertainty, cleanup failure, privacy failure, or safety invariant
violation ends the attempt without retry.

Accepted outcomes: `complete` only when exact package identity, one admitted
physical board 205, one restart request, same-device recovery, exact build,
changed boot session, ordinal `N+1`, software reset, immediate and post-restart
theme equality, exact restoration, disabled mining/hardware control, cleanup,
and redaction all pass. Otherwise record the closed public automation category
and recovery booleans, withhold `RESULT.md` and evidence, keep `API-010` at
`implemented`, and stop.

Verification: The exact package binds source commit
`f80b9b9656b9da20f36ee600f767ddd449a1684d` and reference commit
`c1915b0a63bfabebdb95a515cedfee05146c1d50`. The task-recorded detector and
sole capture commands ran once. The capture returned public category
`process_failed` with safe summary `exact-package flash-monitor failed`; the
public projection is absent. The attempt and initial roots are mode `0700`,
the wrapper streams are mode `0600`, and the empty attempt root contains no
private captured files. Pre-attempt formatting, strict Clippy, build, Cargo
tests, Bright Builds, `just test`, parity, progress, redaction, reference, and
diff gates passed.

Completion review: `attempt-003` is exhausted without promotion. The earliest
failure is the distinct initial `process_failed` boundary, not the fixed
`baseline_multiple_sessions` signature, so `stop_repeated_boundary` does not
apply. No theme mutation, software restart, durability observation, restoration
transaction, or public evidence occurred; any exact-package flash effect is
unconfirmed because the child emitted no accepted capture. Keep `API-010` at
`implemented` and diagnose the flash-monitor child boundary before proposing a
fresh progress-backed ordinal. This task does not claim installed AxeOS browser
behavior or any networking, mining, ASIC, hardware-control, OTA, recovery,
other-board, or release parity.

### task-parity-api010-live-theme-durability-attempt-004 | 2026-08-04 | Run typed post-fix theme durability

- [x] Preserve the pushed baseline, selector, and initial-child diagnostic
      fixes and freeze the exact planning-commit package; the one private
      Ultra 205 detector admission terminated before capture.
- [x] Withhold the bounded `attempt-004` capture after the detector failure;
      no theme GET/POST, flash-monitor, software restart, or recovery fallback
      was launched.
- [x] Record the earliest closed detector signature, withhold evidence, keep
      only `API-010` at `implemented`, and stop without retry.

Plan: `docs/parity/work-plans/20260804T204310Z-API-010/PLAN.md`. This immutable
plan directly continues
`docs/parity/work-plans/20260804T200849Z-API-010/PLAN.md` after verified source
commit `8c93b1b73a0e62ba4fecb1ae46604d30ac29916a` added the missing real-process
diagnostic boundary.

Authorized commands after the plan/task checkpoint is clean, verified,
committed, and pushed:

1. `just package`
2. `test ! -e scratch/api010-theme-durability/wrapper-004 && (umask 077; mkdir -m 700 scratch/api010-theme-durability/wrapper-004 && just detect-ultra205 > scratch/api010-theme-durability/wrapper-004/detector.stdout 2>&1)`
3. `test ! -e scratch/api010-theme-durability/attempt-004 && test ! -e docs/parity/evidence/api010-theme-durability/theme-durability-projection.json && (umask 077; just verify-theme-durability --private-root scratch/api010-theme-durability/attempt-004 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/api010-theme-durability/wrapper-004/detector.stdout --projection docs/parity/evidence/api010-theme-durability/theme-durability-projection.json --capture-timeout-seconds 360 > scratch/api010-theme-durability/wrapper-004/verify.stdout 2> scratch/api010-theme-durability/wrapper-004/verify.stderr)`

Hardware and effect boundary: detector admission must find exactly one likely
ESP USB serial port and successful ESP32-S3 board info. The capture may use the
admitted port for one exact-package flash-monitor transaction, one generated
non-secret theme mutation, one normal software restart, exact readbacks and
restoration, and at most one built-in exact-package recovery flash if normal
restoration cannot be confirmed. It may not change Wi-Fi or pool configuration,
mine, enable ASIC work, change voltage/frequency/fan/thermal/power controls,
exercise display input, perform OTA or raw partition writes, discover network
targets, terminate foreign processes, use direct UART or pins, or perform any
other electrical action.

Evidence and privacy: the supervisor exclusively creates the absent
`scratch/api010-theme-durability/attempt-004` child as mode `0700` with only
mode-`0600` private artifacts. The caller-owned
`scratch/api010-theme-durability/wrapper-004` sibling is mode `0700` with
mode-`0600` streams. Credential contents remain `NeverPersistRaw`. The public
projection or failure envelope may contain only closed schema fields, public
provenance, categories, booleans, bounded counts/durations, the closed
device-session projection, and the declared initial-child discriminator—never
origins, theme/hostname values, ports, USB/network identifiers, credentials,
raw child/HTTP/serial/process material, or private paths.

Recovery, retry, and stop: preserve the earliest typed failure. Normal exact
theme restoration/readback precedes the workflow's one exact-package recovery
fallback; recovery remains secondary and public only as safe booleans.
`attempt-004` is the sole authorized capture. If an initial-child failure omits
or malforms the now-required closed discriminator, select
`stop_repeated_boundary`. Any detector failure, launch failure, timeout,
non-ready session, persistence mismatch, restoration uncertainty, cleanup or
privacy failure, or safety invariant violation ends without retry.

Accepted outcomes: `complete` only when exact package identity, one admitted
physical board 205, one restart request, same-device recovery, exact build,
changed boot session, ordinal `N+1`, software reset, immediate and post-restart
theme equality, exact restoration, disabled mining/hardware control, cleanup,
and redaction all pass. Otherwise record the closed automation category and
declared signature/recovery booleans, withhold `RESULT.md` and evidence, keep
`API-010` at `implemented`, and stop.

Verification: The exact package was built from planning commit
`3fbd7db361fb5dac02ee0412056e58bbdc760b7e`. The sole authorized detector
command exited `1` with the closed durable-USB category
`bootloader_connect_failed`. The protected wrapper and detector transcript
retained modes `0700` and `0600`; the attempt root, verifier streams, and
public projection remain absent. No second detector or capture was run.

Completion review: `attempt-004` is exhausted at detector admission. It
produced no promotable evidence, so `API-010` remains `implemented`. No
firmware flash, theme mutation, software restart, recovery fallback, mining,
ASIC work, hardware-control action, OTA, direct UART, or pin work occurred.
The closed detector category is the only new blocker fact; private device,
port, USB, network, credential, and trace values remain unreported. This task
claims no installed AxeOS browser, networking, mining, ASIC, hardware-control,
OTA, recovery, other-board, or release parity.

### task-parity-api010-live-theme-durability-attempt-005 | 2026-08-04 | Continue after normal-power remediation

- [x] Preserve the pushed implementation and attempt-004 record, then commit
      and push the linked immutable attempt-005 plan before any hardware action.
- [x] After the user reports one full normal barrel/USB power cycle, freeze the
      exact planning-commit package and run one fresh protected detector.
- [x] Run the single bounded capture only if board-info admission objectively
      proves the detector boundary changed; otherwise record the terminal
      detector outcome and stop.
- [x] Promote only `API-010` on complete typed evidence; otherwise preserve the
      earliest category, withhold evidence, keep `implemented`, and stop.
- [x] Create a linked audited software-remediation plan for the protected
      panic/stack-overflow reboot-loop classification before any attempt-006
      hardware action.

Plan: `docs/parity/work-plans/20260804T205704Z-API-010/PLAN.md`. This immutable
plan directly continues
`docs/parity/work-plans/20260804T204310Z-API-010/PLAN.md` after attempt-004
stopped at detector admission.

Progress basis: protected classification narrows the attempt-004 signature to
`terminal_category=bootloader_connect_failed`,
`espflash_detail=connection_failed`, `enumeration_changed=false`,
`same_physical_device=true`, and `cleanup_complete=true`. The durable USB
policy maps exactly those facts to a full normal barrel/DC and USB power cycle.
The cycle is an authorized non-invasive remediation through normal connectors,
but only a successful fresh detector is objective proof that the boundary
changed.

Manual occurrence checkpoint: the user must disconnect both normal barrel/DC
power and USB for at least ten seconds, then reconnect normal barrel power
followed by USB. Do not infer or automate this occurrence, and do not run the
detector until the user reports it completed. This is not a repeated authority
request; standing task authorization already covers the task-gated commands.

Authorized commands after this plan/task checkpoint is clean, verified,
committed, and pushed and after the manual occurrence is reported:

1. `just package`
2. `test ! -e scratch/api010-theme-durability/wrapper-005 && (umask 077; mkdir -m 700 scratch/api010-theme-durability/wrapper-005 && just detect-ultra205 > scratch/api010-theme-durability/wrapper-005/detector.stdout 2>&1)`
3. Only after command 2 succeeds:
   `test ! -e scratch/api010-theme-durability/attempt-005 && test ! -e docs/parity/evidence/api010-theme-durability/theme-durability-projection.json && (umask 077; just verify-theme-durability --private-root scratch/api010-theme-durability/attempt-005 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/api010-theme-durability/wrapper-005/detector.stdout --projection docs/parity/evidence/api010-theme-durability/theme-durability-projection.json --capture-timeout-seconds 360 > scratch/api010-theme-durability/wrapper-005/verify.stdout 2> scratch/api010-theme-durability/wrapper-005/verify.stderr)`

Hardware and effect boundary: the manual step may use only the device's normal
barrel/DC and USB connectors. Detector admission must find exactly one likely
ESP USB serial port and successful ESP32-S3 board info. The capture may use the
admitted port for one exact-package flash-monitor transaction, one generated
non-secret theme mutation, one normal software restart, exact readbacks and
restoration, and at most one built-in exact-package recovery flash if normal
restoration cannot be confirmed. It may not change Wi-Fi or pool configuration,
mine, enable ASIC work, change voltage/frequency/fan/thermal/power controls,
exercise display input, perform OTA/erase/fault-injection/raw-partition writes,
discover network targets, terminate foreign processes, use direct UART or pins,
or perform any other electrical action.

Evidence and privacy: the supervisor exclusively creates the absent
`scratch/api010-theme-durability/attempt-005` child as mode `0700` with only
mode-`0600` private artifacts. The caller-owned
`scratch/api010-theme-durability/wrapper-005` sibling is mode `0700` with
mode-`0600` streams. Credential contents remain `NeverPersistRaw`. The public
projection or failure envelope may contain only closed schema fields, public
provenance, categories, booleans, bounded counts/durations, the closed
device-session projection, and declared child discriminators—never origins,
theme/hostname values, ports, USB/network/process identifiers, credentials,
raw child/HTTP/serial material, or private paths.

Recovery, retry, and stop: preserve the earliest typed failure. Normal exact
theme restoration/readback precedes the workflow's one exact-package recovery
fallback; recovery remains secondary and public only as safe booleans.
`attempt-005` authorizes exactly one detector and, only after its success, one
capture. Recurrence of the full attempt-004 signature after the manual cycle
selects `stop_repeated_boundary`; another detector failure selects
`stop_hardware_blocker`. Any launch failure, timeout, non-ready session,
persistence mismatch, restoration uncertainty, cleanup/privacy failure, or
safety invariant violation ends without retry.

Accepted outcomes: `complete` only when exact package identity, one admitted
physical board 205, one restart request, same-device recovery, exact build,
changed boot session, ordinal `N+1`, software reset, immediate and post-restart
theme equality, exact restoration, disabled mining/hardware control, cleanup,
and redaction all pass. Otherwise record the closed category and safe recovery
booleans, withhold `RESULT.md` and evidence, keep `API-010` at `implemented`,
and stop.

Attempt-005 outcome: the user reported the required normal-connector cycle and
the one detector passed, objectively changing the attempt-004 boundary. The
single capture completed the exact-package flash effect but stopped as
`evidence_invalid` with no public projection, recovery flash, or secondary
recovery failure. Protected offline classification reports
`runtime_origin_missing`, 27 distinct sequential boot sessions and ordinals,
27 panic resets, no runtime-origin or connected Wi-Fi marker, and only the
allowlisted stack-overflow panic category. The workflow did not read or mutate
the theme or request a software restart. Attempt-005 is consumed; evidence and
`RESULT.md` remain withheld and `API-010` remains `implemented`.

Next safe action: no hardware retry. Create and push a new immutable
software-remediation plan that confirms and fixes the boot-evidence replay
stack overflow before assigning attempt-006. The 10-second replay cadence, one
identity per boot, and the source-owned 8 KiB background observer stack are the
leading source-level hypothesis, not yet a verified root cause.

Verification: The exact package and sole detector passed before the bounded
capture produced the closed failure above. The ordered format, strict Clippy,
all-target build, all-feature Cargo tests, Bright Builds, all Bazel tests,
parity/progress, semantic redaction, pinned-reference cleanliness,
immutable-plan, protected-mode, sensitive-output, and diff checks pass. The
public projection remains absent and no scratch artifact is tracked.

Completion review: Attempt-005 is truthfully closed without promotion or a
hardware retry. The residual risk is the unverified 8 KiB boot-evidence
observer stack hypothesis and the exact installed package remains in the panic
loop until a separately planned software remediation is built and flashed.
This task claims no installed AxeOS browser, network discovery, mining, ASIC,
hardware-control, OTA, recovery, other-board, or release parity beyond the
exact admitted transaction.

### task-parity-api010-live-theme-durability-attempt-006 | 2026-08-04 | Fix boot-evidence replay stack overflow

- [x] Raise only the boot-evidence observer stack to 16 KiB and preserve its
      single-owner, cadence, identity, attestation, and safe-state behavior.
- [x] Add a focused source-ownership regression and run the canonical firmware
      build plus the complete ordered repository gate.
- [x] Commit and push the clean software fix before one fresh protected
      detector and conditional attempt-006 capture.
- [x] Promote only `API-010` on complete typed evidence; otherwise preserve the
      earliest category, withhold evidence, keep `implemented`, and stop.
- [x] Create a linked attempt-007 remediation contract requiring one full
      normal barrel/USB power cycle before any new detector or hardware action.

Plan: `docs/parity/work-plans/20260804T222559Z-API-010/PLAN.md`. This immutable
plan continues the pushed attempt-005 outcome at `dc2ea737` after protected
classification proved a panic/stack-overflow reboot loop.

Progress basis: the exact-package flash effect completed, 27 distinct
sequential boot sessions and ordinals were observed, all 27 reset reasons were
`panic`, no runtime-origin or connected Wi-Fi marker appeared, and the only
allowlisted panic category was stack overflow. Each boot emitted one identity
and safe-state marker before failing around the first 10-second identity
replay. Source inspection shows that replay belongs to the sole 8 KiB
boot-evidence observer, while the same emission succeeds during startup. The
16 KiB budget already used by complex runtime owners is the minimal bounded
remediation; hardware evidence must still verify the hypothesis.

Authorization, commands, private paths, recovery, retry bounds, stop
conditions, and promotion criteria are exact in the linked plan. Standing task
authorization covers its single detector-gated attempt after the fix is clean,
verified, committed, and pushed. No new manual electrical action is required.

Verification: The focused ownership target failed red against the 8 KiB
budget and passed green after the sole 16 KiB production change. It proves one
budget declaration, one stack-size use, one observer spawn, and unchanged
10-second identity replay. The canonical ESP32-S3 firmware builds; formatting,
strict Clippy, all-target/all-feature build, all Cargo tests, Bright Builds,
all 35 Bazel tests, parity validation/progress, semantic redaction, and
pinned-reference cleanliness pass. The exact attempt-006 package built, but
the sole detector stopped as `bootloader_connect_failed`; same-device retry
admission and final cleanup each had three stable accessible holder-free
samples with unchanged enumeration. No flash or capture occurred.

Completion review: Attempt-006 is truthfully closed without retry, evidence,
or promotion. The software fix remains pushed but untested on-device because
the detector failed before flashing. A new attempt requires its own immutable
contract and one reported normal-connector power cycle. This task claims no
network discovery, mining, ASIC, hardware-control, display-input, OTA,
partition, recovery, other-board, or release parity beyond the exact admitted
transaction.

### task-parity-api010-live-theme-durability-attempt-008 | 2026-08-04 | Remove oversized screen snapshot path

- [x] Commit and push the linked immutable attempt-008 plan before source
      changes.
- [x] Replace the full API-snapshot screen dependency with a narrow read-only
      projection and remove the disproven observer-stack workaround.
- [x] Add a red/green source regression and canonical ELF entry-frame audit,
      then run the complete software gate.
- [x] Commit and push the clean exact-package fix before hardware.
- [x] Run exactly one protected detector and conditional attempt-008 capture;
      promote only `API-010` on complete typed evidence, otherwise withhold
      evidence, keep `implemented`, and stop.

Plan: `docs/parity/work-plans/20260804T230534Z-API-010/PLAN.md`. This immutable
plan directly continues the pushed attempt-007 outcome at `bfcd4234`.

Diagnosis: attempt-007 passed detector admission and exact-package flashing,
then closed `runtime_origin_missing`. Its private trace reduces to 51 distinct
panic-reset boot sessions and 52 stack overflows with no runtime-origin or
Wi-Fi-state marker. Startup reaches the rendered operator display immediately
before failure. ELF disassembly proves the 8 KiB operator-sensor task's 2 KiB
frame reaches a 7,872-byte full API-snapshot frame through the physical-screen
collector. The earlier boot-observer stack change targeted the wrong thread.

Authorization, exact commands, private paths, recovery, retry bounds, stop
conditions, and promotion criteria are defined in the linked plan. Standing
task authorization covers one attempt-008 only after the software fix is
clean, verified, committed, and pushed. No manual electrical action is
required.

Verification: The focused source ownership test failed before the projection
change and passed afterward. The canonical build now audits the real demangled
ELF and measures 2,048 bytes for the operator-sensor entry frame plus 960 bytes
for the screen collector, 3,008 bytes combined. Formatting, strict Clippy,
all-target/all-feature build and tests, Bright Builds, all 35 Bazel tests,
canonical firmware build/package, parity validation/progress, redaction,
reference cleanliness, selector stability, immutable-plan, and diff checks
passed. Exact commit `50287f62` packaged successfully. The one protected
attempt-008 detector then stopped as `bootloader_connect_failed`; no capture,
theme mutation, restart, public projection, or promotion occurred.

Completion review: The physical-screen root cause is fixed and guarded in
source and the real ELF, but the single authorized attempt did not pass device
admission. `API-010` therefore remains `implemented`, the evidence and result
remain withheld, and this task stays unresolved under the no-retry contract.
This task claims no network discovery, mining, ASIC, hardware-control,
display-input, OTA, partition, recovery, other-board, or release parity beyond
the exact admitted transaction.

### task-parity-api010-bootloader-diagnostic-attempt-009 | 2026-08-04 | Diagnose USB bootloader synchronization

- [x] Commit and push the linked immutable diagnostic plan before source
      changes.
- [x] Turn a protected espflash debug transcript fixture red at the real USB
      classification seam, then add a closed bootloader failure signature.
- [x] Enable private diagnostic logging only for detector-owned board-info
      children and prove the real child environment boundary.
- [x] Preserve raw logs exclusively below the mode-0700 device-session root;
      public errors may contain only the closed category and signature.
- [x] Run focused and complete software gates, commit, and push the diagnostic
      implementation before hardware.
- [x] Run one protected diagnostic detector and, only if it succeeds, one
      attempt-009 theme-durability capture against the exact fixed package.
- [x] Promote only `API-010` on complete typed evidence; otherwise withhold
      evidence, preserve the earliest diagnostic signature, and stop.

Plan: `docs/parity/work-plans/20260805T005320Z-API-010/PLAN.md`. This immutable
plan directly continues attempt-008 at `db0bef8f` without editing its plan or
repeating its uninstrumented detector boundary.

Authorization: Standing task authorization covers the exact repo-owned
board-info detector and conditional capture defined in the linked plan. The
detector may use the existing USB reset/hard-reset sequence but performs no
flash write. The conditional capture retains its one exact-package flash,
generated non-secret theme mutation, one software restart, exact restoration,
and bounded recovery contract. Factory reset, erase, OTA, raw writes, mining,
hardware controls, direct UART, pins, pads, headers, probes, jumpers, and
injected signals remain prohibited.

Verification: The pre-plan and implementation gates passed Cargo format,
clippy, build, and tests; Bright Builds; all Bazel tests; canonical package;
parity and progress; redaction; pinned-reference; selector; immutable-output;
and diff checks. The implementation gate first caught and resolved a
file-length finding by splitting process tests from runtime code. A repeated
macOS `os error 35` at parity triggered host-capacity diagnosis; the host was
healthy and the re-planned fresh-process parity and remaining checks passed.

Hardware result: The sole detector failed as `bootloader_connect_failed` with
`connection_signature=generic_connection_failure`. Its seven protected reset
attempts contained no boot-mode/download-mode observation and no specific
serial/reset error. The exact package remained unflashed, attempt-009 capture
was not created, final evidence was withheld, cleanup completed, and
`API-010` remains `implemented`.

Completion review: Terminal blocker. Automatic USB-JTAG/Serial control toggles
complete but do not place the connected ESP32-S3 into an observable ROM
download session. Pinned espflash and managed esptool use the same reset
sequence, while factory reset cannot repair or bypass this pre-flash boundary.
Further recovery requires an external normal-connector state change or manual
boot-mode hardware intervention under a new authorization/attempt contract.
This task claims no parity beyond the exact diagnostic transaction.

Plan closure:
`docs/parity/work-plans/20260805T005320Z-API-010/CLOSURE.md` records the
terminal non-verified disposition. `API-010` remains `implemented`, no parity
transition or progress synchronization is warranted, and physical Bitaxe
access is currently unavailable. The next safe action is a fresh task-gated
plan and attempt after physical access returns; this exhausted task remains
active and unarchived.

### task-ultra205-boot-recovery-attempt-010 | 2026-08-11 | Recover the blinking Ultra 205 with one safe observation flash

- [x] Commit and push the linked immutable recovery plan before hardware use.
- [x] Verify and freeze one clean current-head Ultra 205 package.
- [x] Run one protected detector after the user's fresh USB connection.
- [x] Only if detector admission succeeds, run one exact-package observation
      campaign that persists `mineonboot=false` and performs no mining or
      hardware-control actuation.
- [x] Preserve the earliest typed outcome, record whether the flash effect
      completed separately from runtime proof, and stop without a retry.

Plan: `docs/parity/work-plans/20260811T150224Z-API-010/PLAN.md`. This immutable
plan follows the closed attempt-009 lineage after physical access returned and
the user reported a fresh USB connection with the same blinking symptom. It
does not reopen or edit any prior plan.

Hardware contract:

1. `just package`
2. `test ! -e scratch/ultra205-boot-recovery/wrapper-010 && (umask 077; mkdir -m 700 -p scratch/ultra205-boot-recovery/wrapper-010 && just detect-ultra205 > scratch/ultra205-boot-recovery/wrapper-010/detector.stdout 2>&1)`
3. Only after command 2 succeeds:
   `just mining-campaign stage=observation board=205 port=<detector-port> manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json evidence-dir=scratch/ultra205-boot-recovery/attempt-010 duration-seconds=360 redact-evidence=true`

Objective and preconditions: admit exactly one Ultra 205, install the clean
current package containing the screen-stack fix, and determine whether it
reaches an exact-package, stable, non-mining runtime. The worktree and source
commit must be clean and pushed, the package manifest must bind current source
and the pinned reference, `wifi-credentials.json` must exist without exposing
its contents, and both private roots must be absent before use.

Allowed effects: repo-owned USB reset and re-enumeration; one factory-package
flash; replacement of the NVS partition with only the owner-supplied Wi-Fi
credentials, `mineonboot=false`, and the observation-stage marker; and bounded
receive-only USB plus same-origin runtime observation. The NVS replacement may
remove prior hostname, pool, and other settings. No pool credentials are read.

Prohibited effects: mining, ASIC work or initialization, voltage, frequency,
fan, thermal, or power control; pool traffic; OTA; erase-flash; ad hoc or raw
writes; network discovery; foreign-process termination; parity promotion;
direct UART; pins, pads, headers, GPIO, probes, jumpers, soldering, or injected
signals.

Evidence and privacy: the wrapper and attempt roots are ignored mode-0700
directories containing mode-0600 artifacts. Credential contents, ports, USB
identities, network values, origins, commands, and raw serial/process traces
remain private. Public reporting is limited to clean provenance, closed
categories, bounded counts, safe booleans, and whether detector, flash,
runtime identity, safe state, cleanup, and redaction passed.

Recovery, retry, and stop: detector failure stops before write. If the flash
effect completes but runtime proof fails, record those outcomes separately and
do not reflash. Preserve the earliest typed failure through cleanup. Release
owned USB and process resources in every terminal path. This task authorizes
exactly one detector and, conditionally, one observation campaign; any failure
or recurrence stops without another attempt. Manual boot-mode intervention
remains outside this task.

Acceptance: `complete` requires detector admission, exact current-package
flash completion, trusted same-package runtime identity, stable observation,
`mineonboot=false`, no mining or hardware-control effects, USB cleanup, private
artifact modes, and redaction. Otherwise record the closed terminal category,
withhold parity evidence and `RESULT.md`, and keep `API-010` at `implemented`.

Verification: The clean exact package bound source commit `d6770696`, the
pinned reference, six required artifacts, and `source_dirty=false`. The sole
detector passed with a private mode-0700 wrapper and mode-0600 output. The
conditional command then stopped before the campaign binary, evidence-root
creation, USB ownership, flash, NVS write, credential read, or runtime
observation because the task supplied `stage=observation` while the canonical
Clap interface requires `--stage observation`.

Completion review: Closed as `process_failed` with
`cli_argument_rejected`. No campaign hardware effect occurred, attempt-010 is
consumed, and the detector's changed ROM-sync boundary remains established.
The next eligible attempt requires a regression proving the exact observation
flag shape and a new immutable command contract. This task cannot verify theme
durability or promote `API-010`.

Plan closure:
`docs/parity/work-plans/20260811T150224Z-API-010/CLOSURE.md` records the
non-verified outcome. `API-010` remains `implemented` and no evidence or
`RESULT.md` was created.

### task-parity-rel002-rollback-interruption-attempt-001 | 2026-08-11 | Prove ESP-IDF interrupted-update abort and rollback

- [x] Commit and push the immutable `REL-002` plan before implementation.
- [x] Build an isolated pending-validation rollback probe without changing the
      normal release image behavior.
- [x] Add typed interrupted-upload, probe-boot, rollback, recovery, evidence,
      and redaction workflows with behavior-focused and real-process tests.
- [x] Run focused and mandatory software gates; commit and push the exact
      implementation before hardware use.
- [x] Run one protected detector and, only after admission, one protected
      attempt-001 capture on board 205.
- [ ] Validate the closed public projection and promote only `REL-002` when
      every interruption, same-device, rollback, restoration, and cleanup fact
      passes.

Plan: `docs/parity/work-plans/20260811T210329Z-REL-002/PLAN.md`.

Objective and preconditions: close only the `REL-002` rollback-enabled SDK
behavior on one connected Ultra 205. The worktree and pinned reference must be
clean, source must be pushed, the normal package and isolated rollback probe
must bind the same source/reference, `wifi-credentials.json` is an opaque
ignored input, and all private/output roots must be absent before first use.

Authorized hardware command: after the linked plan's software gates and clean
push, run its exact package/probe build, protected `just detect-ultra205`, and
conditional `just capture-sdkconfig-rollback-evidence ...` attempt-001 command
once and in order.

Allowed effects: repo-owned USB reset/re-enumeration; one exact normal factory
flash; replacement NVS containing only owner-supplied Wi-Fi credentials and
`mineonboot=false`; bounded receive-only USB and same-origin HTTP; one bounded
truncated application OTA write followed by protocol abort; one complete
rollback-probe application OTA upload; its scheduled software restart; and one
normal probe restart that permits ESP-IDF bootloader rollback. If normal build
restoration cannot be confirmed, one exact normal factory recovery flash is
allowed only for recovery and cannot produce success evidence.

Prohibited effects: OTAWWW or SPIFFS update, erase-flash, arbitrary raw writes,
bootloader or partition-table corruption, power interruption, mining, ASIC
work or initialization, pool access, voltage, frequency, fan, thermal, or
power control, network discovery, foreign-process termination, direct UART,
pins, pads, headers, GPIO, probes, jumpers, soldering, or injected signals.

Evidence and privacy: the wrapper and capture roots are ignored mode-`0700`
directories with mode-`0600` files. Credentials, origins, hostnames, ports,
USB/network/process identities, HTTP bodies, image bytes, commands, and raw
serial/child traces remain private. Only the closed redacted v1 projection may
be committed. Public facts are limited to provenance/digests, bounded counts,
safe booleans, typed terminal category, cleanup, modes, and redaction.

Recovery, retry, and stop: detector failure stops before writes. Preserve the
earliest typed failure through cleanup and optional exact-package recovery;
recovery status is secondary. Release every owned resource. Attempt-001 is the
only authorized ordinal and is consumed by any conditional capture start.
Stop without retry on success or on `package_invalid`, `process_failed`,
`timeout`, `hardware_blocked`, `evidence_invalid`,
`interruption_not_observed`, `probe_boot_failed`, `rollback_not_observed`, or
`recovery_failed`.

Acceptance: complete only when the exact normal baseline remains unchanged
after one interrupted upload, the same physical device boots the admitted
pending-validation probe in `ota_0` at `N+1`, one normal restart causes native
rollback to the exact normal factory build at the following ordinal, mining
and hardware control remain disabled, cleanup/modes/redaction pass, and the
typed projection validates. Otherwise withhold evidence, create a truthful
closure, and leave `REL-002` at `implemented`.

Verification: The complete focused and mandatory software gate passed and the
implementation was pushed at `89fd198cf262f4755a5846206da0e122985f92c6`.
The detector admitted one Ultra 205. Attempt-001 ended as
`interruption_not_observed`: ten same-origin checks confirmed the unchanged
exact normal factory build but found no retained protocol-abort marker. The
probe and rollback sessions did not start, no recovery flash was needed, all
private paths remained mode protected, cleanup completed, and the public
projection was withheld.

Completion review: Non-verifying terminal closure. The host interruption helper
returns after a write-side `socket.end` flush without forcing or awaiting full
socket closure, while its synthetic child closes the peer side itself. A new
attempt requires a force-close regression, a pushed fix, a fresh immutable
task/plan, new paths, and a new detector. Attempt-001 is consumed; `REL-002`
remains `implemented`.

### task-parity-rel002-reset-before-fin-attempt-002 | 2026-08-11 | Prove interrupted OTA abort and native rollback with TCP reset

- [ ] Commit and push the corrected immutable `REL-002` attempt-002 plan
      before implementation.
- [x] Flush the strict partial request without FIN, force one TCP reset, await
      local close, and prove it with a non-cooperative real child.
- [x] Run focused and mandatory software gates; commit and push the exact
      implementation before hardware use.
- [x] Run one protected detector and, only after admission, one protected
      attempt-002 capture on board 205.
- [ ] Validate the closed public projection and promote only `REL-002` when
      every abort, same-device, rollback, restoration, cleanup, and privacy
      fact passes.

Plan: `docs/parity/work-plans/20260811T215904Z-REL-002/PLAN.md`.

Objective and preconditions: close only `REL-002` on one connected Ultra 205.
The worktree, pushed source, and pinned reference must be clean; normal and
isolated rollback-probe packages must share provenance; the ignored credential
input remains opaque; and fresh private/public targets must be absent.

Authorized hardware command: after the linked plan's software gates and clean
implementation push, build its exact normal package and probe, run its one
protected `just detect-ultra205`, then conditionally run its exact protected
`just capture-sdkconfig-rollback-evidence ...` attempt-002 command once.

Allowed effects: repo-owned USB reset/re-enumeration; one exact normal factory
flash; replacement NVS with owner-supplied Wi-Fi and `mineonboot=false`;
bounded receive-only USB and same-origin HTTP; one bounded truncated
application OTA request terminated by TCP reset before FIN; one complete
rollback-probe OTA; its scheduled software restart; and one normal probe
restart permitting native rollback. One exact normal recovery flash is allowed
only if restoration cannot otherwise be confirmed and cannot produce success.

Prohibited effects: OTAWWW or SPIFFS update, erase-flash, arbitrary raw writes,
bootloader/partition-table corruption, power interruption, mining, ASIC work,
pool access, voltage, frequency, fan, thermal or power control, network
discovery, foreign-process termination, direct UART, pins, pads, headers,
GPIO, probes, jumpers, soldering, or injected signals.

Evidence and privacy: wrapper-002 and attempt-002 are ignored mode-`0700`
directories with mode-`0600` files. Credentials, origins, hostnames, ports,
USB/network/process identities, HTTP bodies, firmware bytes, commands, and raw
traces remain private. Only the closed redacted v1 projection may be committed.

Recovery, retry, and stop: detector failure stops before writes. Preserve the
earliest typed failure through cleanup and optional exact-package recovery;
recovery is secondary. Any conditional capture start consumes attempt-002.
Release every resource and stop without retry on success or any admitted typed
failure category in the linked plan.

Acceptance: complete only when the unchanged normal baseline retains the
protocol abort, the same device boots the pending-validation probe in `ota_0`
at `N+1`, one normal restart rolls back to the exact normal factory build at
the next ordinal, mining/hardware control remain disabled, cleanup, modes, and
redaction pass, and the typed projection validates. Otherwise withhold public
evidence, create a truthful closure, and keep `REL-002` implemented.

Completion review: Attempt-002 closed as `evidence_invalid` before the
interrupted-upload stage. The initial exact-package flash/monitor retained
trusted runtime attestation but consumed the full capture window, and the
baseline HTTP artifact was never created. Exact-package recovery completed,
private modes and cleanup pass, and no public projection exists. Attempt-002
is consumed; continue only with bounded initial monitor admission, typed HTTP
readiness, a full clean gate, and fresh wrapper/attempt-003 paths. See the
linked plan's `CLOSURE.md`.

### task-parity-rel002-baseline-readiness-attempt-003 | 2026-08-11 | Bound initial monitor and baseline readiness

- [x] Commit and push the immutable `REL-002` attempt-003 plan before
      implementation.
- [x] Cap initial monitor capture at 90 seconds and add six typed baseline HTTP
      readiness attempts without changing device-session timeouts.
- [x] Run focused and mandatory software gates; commit and push the exact
      implementation before hardware use.
- [x] Run one protected detector and, only after admission, one protected
      attempt-003 capture on board 205.
- [ ] Validate the closed public projection and promote only `REL-002` when all
      interruption, same-device, rollback, restoration, cleanup, and privacy
      facts pass.

Plan: `docs/parity/work-plans/20260811T223031Z-REL-002/PLAN.md`.

Objective and preconditions: close only `REL-002` on one Ultra 205. Source,
upstream, reference, normal/probe provenance, credential opacity, and fresh
private/public paths must pass before the exact linked commands are eligible.

Authorized effects: the linked plan's one normal factory flash, replacement
NVS with owner Wi-Fi and `mineonboot=false`, bounded receive-only USB and HTTP,
one reset-aborted partial OTA, one complete rollback probe, two planned
software restarts, and conditional exact normal recovery flash. Run one fresh
detector and at most one conditional attempt-003 only after a clean push.

Prohibited effects: OTAWWW/SPIFFS update, erase, raw writes, bootloader/table
corruption, power interruption, mining, ASIC/pool activity, voltage, frequency,
fan, thermal/power control, discovery, foreign-process termination, direct
UART, pins, pads, headers, GPIO, probes, jumpers, soldering, or signals.

Evidence and privacy: wrapper-003/attempt-003 are ignored mode-`0700` roots
with mode-`0600` files. Operational device, network, credential, command,
image, and trace values remain private. Only the closed redacted v1 projection
may be committed.

Recovery, retry, and stop: detector failure stops before writes. Preserve the
earliest typed category through cleanup and optional exact-package recovery.
Any conditional capture start consumes attempt-003; release every resource and
stop without retry on success or any admitted terminal category.

Acceptance: require the exact safe normal baseline, retained partial-upload
abort, same-device pending probe boot in `ota_0` at `N+1`, native rollback to
the exact factory build at the next ordinal, disabled mining/control, cleanup,
modes, redaction, and valid projection. Otherwise withhold evidence, close
truthfully, and keep `REL-002` implemented.

Completion review: Attempt-003 is consumed. The bounded baseline and strict
reset-aborted upload both succeeded, and all ten retained snapshots prove the
canonical API-visible protocol-error status with the normal factory runtime
unchanged. The host searched for a UART-only spelling, closed before probe or
rollback, and emitted no public evidence. Continue only through the linked
closure's fresh predicate-fix plan and attempt-004 paths.

### task-parity-rel002-retained-marker-attempt-004 | 2026-08-11 | Admit the canonical retained OTA abort marker

- [x] Commit and push the immutable `REL-002` attempt-004 plan before
      implementation.
- [x] Replace the UART-only log predicate with the canonical API-visible
      retained OTA status and add fail-closed production-shaped regressions.
- [x] Run focused and mandatory software gates; commit and push the exact
      implementation before hardware use.
- [x] Run one protected detector and, only after admission, one protected
      attempt-004 capture on board 205.
- [ ] Validate the closed public projection and promote only `REL-002` when all
      interruption, same-device, rollback, restoration, cleanup, and privacy
      facts pass.

Plan: `docs/parity/work-plans/20260811T225226Z-REL-002/PLAN.md`.

Objective and preconditions: close only `REL-002` on one Ultra 205. Source,
upstream, reference, normal/probe provenance, credential opacity, and fresh
private/public paths must pass before the exact linked commands are eligible.

Authorized effects: the linked plan's one normal factory flash, replacement
NVS with owner Wi-Fi and `mineonboot=false`, bounded receive-only USB and HTTP,
one reset-aborted partial OTA, one complete rollback probe, two planned
software restarts, and conditional exact normal recovery flash. Run one fresh
detector and at most one conditional attempt-004 only after a clean push.

Prohibited effects: OTAWWW/SPIFFS update, erase, raw writes, bootloader/table
corruption, power interruption, mining, ASIC/pool activity, voltage, frequency,
fan, thermal/power control, discovery, foreign-process termination, direct
UART, pins, pads, headers, GPIO, probes, jumpers, soldering, or signals.

Evidence and privacy: wrapper-004/attempt-004 are ignored mode-`0700` roots
with mode-`0600` files. Operational device, network, credential, command,
image, and trace values remain private. Only the closed redacted v1 projection
may be committed.

Recovery, retry, and stop: detector failure stops before writes. Preserve the
earliest typed category through cleanup and optional exact-package recovery.
Any conditional capture start consumes attempt-004; release every resource and
stop without retry on success or any admitted terminal category.

Acceptance: require the exact safe normal baseline, canonical retained partial-
upload abort, same-device pending probe boot in `ota_0` at `N+1`, native
rollback to the exact factory build at the next ordinal, disabled mining and
control, cleanup, modes, redaction, and valid projection. Otherwise withhold
evidence, close truthfully, and keep `REL-002` implemented.

Completion review: Attempt-004 is consumed. It admitted the canonical partial-
upload abort and completed a `ready` same-device probe OTA into the exact
`ota_0` build at `N+1`, but the late serial attachment missed two early boot
lines. Both lines are intentionally API-retained. Exact-package recovery and
private cleanup passed, no public evidence was emitted, and the linked closure
defines the retained-log continuation using fresh attempt-005 paths.

### task-parity-net002-provisioning-network-attempt-001 | 2026-08-12 | Verify the live configuration network

- [x] Select `NET-002`, inspect its exact reference/implementation/evidence gap,
      and persist the linked immutable plan before implementation.
- [x] Add the typed macOS provisioning-client capture, closed projection,
      independent validator, recovery behavior, and production-shaped tests.
- [x] Run focused and mandatory software gates; commit and push the exact
      implementation before hardware use.
- [x] Run one protected detector and, only after admission, one protected
      attempt-001 capture on board 205.
- [ ] Promote only `NET-002` when exact-package AP visibility, association,
      DHCP, wildcard DNS, captive redirect, settings access, safe recovery,
      cleanup, private modes, and redaction all pass.

Plan: `docs/parity/work-plans/20260812T040437Z-NET-002/PLAN.md`.

Objective and preconditions: close only `NET-002` on one Ultra 205. Source,
upstream, reference, package provenance, owner-credential opacity, fresh paths,
one powered-on/unassociated macOS Wi-Fi interface, and zero matching baseline
candidates must pass before the linked effect commands are eligible.

Authorized effects: one exact normal package flash without Wi-Fi credentials;
replacement NVS with exact Ultra 205 defaults and `mineonboot=false`; bounded
receive-only USB; local candidate enumeration; association to the unique open
configuration AP; one DHCP lease; one synthetic wildcard DNS query; one captive
redirect request; one same-origin system-info read; host Wi-Fi off/on cleanup;
and one ordinary exact-package recovery flash with the owner Wi-Fi file. The
final state retains the exact package, owner Wi-Fi, exact defaults, and disabled
mining.

Prohibited effects: router/RF configuration changes, non-Bitaxe association,
host credential access/mutation, provisioning credential submission, software
restart, station handoff, external discovery, internet evidence requests,
erase, ad hoc/raw writes, OTA, power interruption, mining, ASIC/pool activity,
voltage, frequency, fan, thermal/power control, self-test, foreign-process
termination, direct UART, pins, pads, headers, GPIO, probes, jumpers, soldering,
or signals.

Evidence and privacy: wrapper-001/attempt-001 are ignored mode-0700 roots with
mode-0600 files. Credential, SSID, interface, USB, device, network, route,
origin, command, process, DNS, HTTP, and raw serial values remain private. Only
the closed redacted `bitaxe-provisioning-network-evidence-v1` projection may be
committed.

Recovery, retry, and stop: host admission failure stops before writes. Detector
failure stops before writes. Preserve the earliest typed category through host
cleanup and optional ordinary exact-package recovery. Any conditional capture
start consumes attempt-001; release every resource and stop without retry on
success or any admitted terminal category.

Acceptance: require exact package and board identity, safe AP-only boot, unique
SSID, association, DHCP, wildcard IN/A response to the AP gateway with TTL 300,
the captive redirect contract, same-origin exact-build/settings quorum,
disabled mining/control, host restoration, exact recovery, cleanup, modes,
redaction, and a valid projection. Otherwise withhold evidence, close
truthfully, and keep `NET-002` implemented.

Verification: Immutable-plan gate passed: format, strict Clippy, all-target
build, all-feature tests, Bright Builds, all 37 Bazel tests, parity, progress,
redaction, reference, generated contracts, selector, plan hash, task
uniqueness, reference cleanliness, and diff checks are green. One cold full-
suite run exposed a known timing-sensitive loopback test race; five focused
runs and the complete warm rerun passed. A later monolithic launch reached the
macOS process limit after the successful Bazel build; the same remaining gates
passed after Bazel quiesced. No device or host-network effect occurred.

Completion review: Attempt-001 closed as `evidence_invalid` before host
association because the late-attached serial capture contained recurring exact-
build runtime records but missed the one-shot safe/AP startup lines. Host and
device recovery completed without a secondary failure, private modes passed,
and no public projection exists. The linked closure defines the fresh
continuation; this task remains active and unarchived because `NET-002` is not
verified.

### task-parity-net002-provisioning-network-attempt-002 | 2026-08-12 | Admit late-attached AP runtime evidence

- [x] Select the first canonical row and bind the fresh continuation to the
      explicit attempt-001 closure and exact aggregate diagnosis.
- [x] Replace only the one-shot AP startup-line prerequisite with trusted
      recurring passive-safe runtime admission.
- [x] Add production-shaped late-attach success and missing-safety failure
      regressions; run focused and mandatory software gates.
- [x] Commit and push the exact implementation, then run one protected detector
      and at most one conditional attempt-002 on board 205.
- [ ] Promote only `NET-002` after the complete client/API, safety, recovery,
      cleanup, privacy, and independent-evidence quorum passes.

Plan: `docs/parity/work-plans/20260812T051446Z-NET-002/PLAN.md`.

Objective and preconditions: close only `NET-002` from a clean pushed exact
package after one powered-on/unassociated macOS Wi-Fi interface, zero baseline
configuration candidates, opaque owner-Wi-Fi availability, fresh paths, and
one detector-admitted Ultra 205 pass.

Authorized effects: one exact package flash without credentials; default safe
NVS; bounded USB receive; unique local configuration-AP association; DHCP; one
wildcard DNS query; captive and same-origin system-info reads; host Wi-Fi
cleanup; and one ordinary exact-package owner-Wi-Fi recovery flash. Prohibited
effects remain router/RF mutation, provisioning submission, external discovery,
erase/raw writes, OTA, power interruption, mining, ASIC/pool work, controls,
self-test, direct UART, and pins.

Evidence and privacy: use only fresh ignored mode-0700 wrapper-002 and
attempt-002 roots with mode-0600 files. Keep SSIDs, interfaces, ports, USB and
network identities, addresses, routes, origins, credentials, DNS/HTTP bytes,
commands, processes, and serial content private. Only a valid closed aggregate
projection may be committed.

Recovery, retry, and stop: admission or detector failure stops before the
effect. Preserve the earliest typed category through host cleanup and at most
one ordinary exact-package recovery. Any capture start consumes attempt-002;
release resources and stop without retry on success or any terminal category.

Acceptance: require every immutable-plan exact-package, recurring-runtime,
AP/client/DHCP/DNS/HTTP, safety, host/device recovery, cleanup, mode,
redaction, and validator criterion. Otherwise withhold evidence, close
truthfully, and keep `NET-002` implemented.

Verification: Plan-only gate passed: ordered Cargo, Bright Builds, all 37 Bazel
tests, parity/progress, redaction, reference, generated contracts, selector,
task uniqueness, fresh paths, and diff checks are green. Immutable plan SHA-256
is `657f37b864e8dee5accb4d0bae683f39820a69483d49563dd93f2c951bccd44c`.

Completion review: Attempt-002 passed recurring passive-safe runtime admission
but closed as `hardware_blocked` inside the undifferentiated client observation
stage. Host and exact-package device recovery passed without secondary failure,
private modes and cleanup passed, and no public projection exists. The linked
closure defines the typed-boundary continuation; this task remains active and
unarchived because `NET-002` is not verified.

### task-parity-net002-provisioning-network-attempt-003 | 2026-08-12 | Type the failed client boundary

- [x] Select `NET-002` and bind the continuation to the attempt-002 closure.
- [x] Add the six-value redaction-safe client-boundary error contract.
- [x] Test every boundary, sensitive-output absence, recovery precedence, and
      real-child behavior; run all mandatory software gates.
- [x] Commit and push, then run one detector and at most one conditional
      attempt-003 with fresh protected paths.
- [ ] Promote only after the complete immutable-plan success quorum passes.

Plan: `docs/parity/work-plans/20260812T061233Z-NET-002/PLAN.md`.

Authorization: the linked plan's bounded USB, local Wi-Fi client, DNS/HTTP, host
cleanup, and exact-package recovery effects only. All operational and
identifying values remain private; public failure output permits only one
closed boundary token and safe recovery booleans. Capture start consumes the
ordinal and never retries it.

Verification: Plan-only gate passed: ordered Cargo, Bright Builds, 37 Bazel
tests, parity/progress, redaction, reference, selector, task, fresh-path, and
diff checks are green. Immutable plan SHA-256 is
`a83af65b730179383356a0b349b116a815ef1ee545cc802a631f1e35f4216131`.
Implementation gate also passes the focused production-boundary regression,
the ordered Cargo sequence, Bright Builds, all 37 Bazel tests,
parity/progress, redaction, reference, selector, immutable-plan, task,
fresh-path, reference-cleanliness, sensitive-output, and diff checks.

Completion review: Attempt-003 closed as `hardware_blocked` at the exact
`configuration_candidate` boundary. No association or later client effect
started. Host restoration and exact-package device recovery passed without a
secondary failure; private modes and cleanup passed; no public projection
exists. The linked closure requires a private exact-device candidate plus
recurring AP readiness before another ordinal. This task remains active and
unarchived because `NET-002` is not verified.

### task-parity-net002-provisioning-network-attempt-004 | 2026-08-12 | Bind candidate identity and recurring AP readiness

- [x] Select `NET-002` and bind the continuation to the attempt-003 closure.
- [x] Derive the private expected candidate from the detector-owned device
      identity and add recurring redaction-safe AP/DHCP/DNS readiness.
- [x] Test late attachment, invisible enumeration, ambiguity, malformed device
      identity, all closed boundaries, recovery, real children, and redaction;
      run every mandatory software gate.
- [x] Commit and push, then run one detector and at most one conditional
      attempt-004 with fresh protected paths.
- [ ] Promote only after the complete immutable-plan success quorum passes.

Plan: `docs/parity/work-plans/20260812T063811Z-NET-002/PLAN.md`.

Authorization: the linked plan's bounded USB, local Wi-Fi client, DNS/HTTP,
host cleanup, and exact-package recovery effects only. The exact candidate is
private detector handoff material. Public runtime readiness and failure output
use only closed categories and safe booleans. Capture start consumes the
ordinal and never retries it.

Verification: Plan-only gate passed: ordered Cargo, Bright Builds, all 37
Bazel tests, parity progress, redaction, reference, selector, task uniqueness,
immutable-plan digest, reference cleanliness, fresh paths, and diff checks are
green. Immutable plan SHA-256 is
`48796a1c9bdbbce5fbe3b8f07ae7c34ac6f2a6069396d081321b135e6e569877`.
Implementation verification also passes the focused API, firmware ownership,
flash, automation, real-child, and real firmware-package targets; the ordered
Cargo sequence; Bright Builds; all 37 Bazel tests; parity/progress; redaction;
reference; generated contracts; selector; immutable-plan; task; fresh-path;
reference-cleanliness; sensitive-output; and diff checks. One post-test parity
launch hit transient macOS resource exhaustion, then the isolated complete tail
passed without code changes.

Completion review: Attempt-004 closed as `hardware_blocked` at the typed
`association` boundary after exact-device candidate admission, exact-package
passive safety, and eleven recurring AP/DHCP/DNS readiness samples passed.
Host restoration and exact-package device recovery passed without a secondary
failure; private modes and evidence withholding passed. The linked closure
requires a private association sub-boundary and a supported exact-SSID macOS
association transaction before a fresh ordinal. This task remains active and
unarchived because `NET-002` is not verified.

### task-parity-net002-provisioning-network-attempt-005 | 2026-08-12 | Replace inventory-bound association with CoreWLAN

- [x] Select `NET-002`, bind the continuation to attempt-004, and prove the
      installed CoreWLAN directed-scan and association API shape without an
      effect.
- [x] Add the protected exact-SSID CoreWLAN helper and private association
      sub-boundaries without changing the closed public evidence contract.
- [x] Test real-child fixture execution, timeouts, malformed results, every
      association subtype, recovery precedence, modes, and redaction; run all
      mandatory software gates.
- [x] Commit and push, then run one detector and at most one conditional
      attempt-005 with fresh protected paths.
- [ ] Promote only after the complete immutable-plan success quorum passes.

Plan: `docs/parity/work-plans/20260812T071223Z-NET-002/PLAN.md`.

Authorization: the linked plan's bounded USB, one exact directed CoreWLAN scan
and association, local DHCP/DNS/HTTP, host cleanup, and exact-package recovery
effects only. Candidate and raw CoreWLAN outcomes remain private. Capture start
consumes the ordinal and never retries it.

Verification: Plan-only gate passed: no-effect Swift CoreWLAN typecheck,
ordered Cargo, Bright Builds, all 37 Bazel tests, parity/progress, redaction,
reference, generated contracts, selector, task uniqueness, immutable-plan
digest, reference cleanliness, fresh paths, and diff checks are green.
Immutable plan SHA-256 is
`2d705f1a10befc1e235d383b9b33e5fe620e6522fcba4f6b0ba814d92bc99028`.
Implementation verification passes Swift typecheck and the no-effect
real-child fixture; all automation tests; focused production and exact
firmware-package builds; ordered Cargo; Bright Builds; all 37 Bazel tests;
parity/progress; redaction; reference; generated contracts; selector;
immutable-plan; task; fresh-path; reference-cleanliness; sensitive-output; and
diff checks.

Completion review: Attempt-005 proved directed CoreWLAN association, DHCP,
wildcard DNS, captive redirect, same-origin system-info, cleanup, and recovery.
It closed as `service_recovery_failed` only because the checker incorrectly
treated `startMiningOnBoot === false` as runtime safety. The runtime attestation
proved mining and controls disabled. No projection exists. The linked closure
requires removal of that preference-value assumption in a fresh ordinal; this
task remains active because `NET-002` is not verified.

### task-parity-api009-command-effect-evidence-audit | 2026-08-12 | Audit the complete operator-command effect quorum

- [x] Select `API-009` from a clean synchronized selector without skipping a
      row.
- [x] Audit committed public evidence for all five command-correlated effects.
- [x] Promote only on a complete genuine quorum; otherwise seal the exact
      terminal evidence blocker without partial hardware execution.

Plan: `docs/parity/work-plans/20260812T135813Z-API-009/PLAN.md`.

Authorization: Read-only committed public evidence and source plus mandatory
software verification only. No protected evidence, credentials, detector,
package, flash, reset, USB/network or HTTP session, mining, identify command,
block-state injection, controls, recovery, direct UART, pins, or physical
effect is permitted.

Verification and stop rule: Require all five genuine command effects for
promotion. Missing trusted physical identify rendering or an actually active
production block-found dismissal is a terminal evidence blocker. Do not use
synthetic state, partial hardware effects, weakened claims, or row skipping.

Verification: The plan-only gate passed. The public evidence audit found route
registration but no command-correlated physical identify, active block-found
dismissal, or active-mining pause/resume hardware proof. Current production
state has no writer that raises the block notification; it initializes false
and the dismiss path only preserves the count while writing false. Upstream
requires a nonce at network difficulty, which is not a bounded precondition.
See `docs/parity/work-plans/20260812T135813Z-API-009/CLOSURE.md`.

Completion review: API-009 remains implemented. No device action was run
because partial command effects could not close the conjunctive row. Resume
only after the production block-found producer and bounded genuine active-state,
physical identify, and active-mining command evidence contracts exist.

Software continuation plan:
`docs/parity/work-plans/20260812T141252Z-API-009/PLAN.md`.

- [x] Derive block qualification from one valid current-generation nonce and
      its admitted compact network target in the pure production core.
- [x] Emit one redacted effect and mutate block count/visibility only through
      the retained firmware runtime-snapshot owner.
- [x] Add focused state, correlation, ordering, redaction, and ownership
      regressions; run every mandatory gate and push the source.
- [x] Keep API-009 `implemented`, update only its production ownership target,
      and record the still-missing physical command-effect evidence.

Verification: Source commit `4ab1968982ec614012860230831e3abbdd9a965e`
passes focused API, Stratum, and firmware ownership tests plus the complete
ordered Cargo, Bright Builds, Bazel, parity, parity-progress, redaction, and
reference gate. The immutable plan digest, unique task binding, selector,
reference cleanliness, diff, and sensitive-output reviews also pass.

Completion review: The missing production notification writer is resolved with
one shared target calculation, one redaction-safe effect, and one retained
state owner. API-009 remains `implemented`: accepted physical identify,
active-mining pause/resume, restart, and genuine active-notification dismissal
must still be proven together. See
`docs/parity/work-plans/20260812T141252Z-API-009/CLOSURE.md` for the bounded next
safe action and non-claims; this unresolved task remains active.

Continuation authorization: software, fixtures, builds, and repository
verification only. No hardware, USB, flash, credentials, network/pool session,
mining, ASIC traffic, HTTP command, identify or block-state injection,
hardware controls, OTA, recovery, direct UART, pins, or physical effect.

Hardware evidence continuation plan:
`docs/parity/work-plans/20260812T144217Z-API-009/PLAN.md`.

- [x] Add the typed `command-effects` campaign stage, easy-target local Stratum
      fixture, one-time physical identify checkpoint, canonical restart join,
      closed public evidence projection, and focused regressions.
- [x] Run all focused and mandatory software gates; commit and push the exact
      implementation before touching hardware.
- [x] Run exactly one fresh detector-gated `attempt-001` against board 205 and
      stop after its complete result without retry.
- [ ] Promote API-009 only on the complete five-command device-user quorum;
      otherwise preserve `implemented` and record the first typed blocker.

Hardware continuation authorization: standing task authorization permits the
single bounded attempt only after the immutable plan/task checkpoint and clean
pushed implementation pass. The repo-owned command must flash the exact
package, seed only private generated local-pool credentials plus the opaque
Wi-Fi input, mine under the conservative profile for at most 600 seconds,
issue pause/resume/identify/dismiss once in the admitted order, obtain one-time
operator observations for IDENTIFY rendering and clearing, safe-stop, and make
exactly one canonical software-restart request. Allowed physical effects are
USB flash/reset, conservative BM1366 initialization/mining against the local
fixture, display identify on/off, HTTP command effects, and software restart.
No external pool, owner pool credentials, diagnostic setter, erase, OTA,
rollback, power cycle, direct UART, pins, fault injection, or control override
is permitted. Private evidence remains mode 0700/0600 beneath ignored scratch;
public output contains no origin, hostname, port, USB/network identity,
credential, worker, address, password, token, checkpoint secret, or raw trace.

Verification: Source commit `2d6337e090e1ba747fdc1507830c732782eaf139`
passes the complete ordered Cargo, Bright Builds, all 39 Bazel tests, parity,
progress, redaction, reference, generated-contract, selector, task,
immutable-plan, reference-cleanliness, sensitive-output, and diff gates.
Detector-gated `attempt-001` stopped before device admission as `timeout`
because the local fixture readiness document did not appear. No campaign root
or public projection exists; no device effect occurred. Host-only exact-runtime
diagnostics passed and localize the defect to the automation child-lifecycle
handoff. See the linked `CLOSURE.md`.

Completion review: API-009 remains `implemented`. The immutable attempt is
consumed without a hardware retry. The next safe action is a new immutable
continuation that races fixture child completion against readiness, preserves a
protected launch diagnostic, tests the real local process port, and defines a
fresh detector-gated ordinal.

Host-process continuation plan:
`docs/parity/work-plans/20260812T154751Z-API-009/PLAN.md`.

- [x] Replace the implicit `process.execPath` fixture launch with a repo-owned
      Bazel/runfiles executable and race early child completion with readiness.
- [x] Persist only protected closed child outcome facts and add the real
      deployed-layout `createLocalProcessPort` regression.
- [x] Run every focused and mandatory software gate; commit and push the exact
      fixed source before touching hardware.
- [x] Run exactly one fresh detector-gated `attempt-002` and stop after its
      complete terminal result without retry.
- [x] Promote API-009 only on the complete five-command device-user quorum;
      otherwise preserve `implemented` and the first typed blocker.

Attempt-002 authorization: standing task authorization permits the single
fresh retry only after the real deployed-launcher regression proves the exact
attempt-001 boundary fixed and the immutable plan, task, software, privacy,
recovery, and evidence gates pass. Allowed effects and prohibited effects are
identical to attempt-001. The command must use a fresh mode-`0700` private root,
mode-`0600` artifacts, exact clean package, detector-admitted board 205,
conservative 600-second local-fixture mining lease, one-time physical identify
checkpoints, request-once commands, safe stop, cleanup, and canonical software
restart. No external pool, owner credentials, diagnostic setter, erase, OTA,
rollback, power cycle, direct UART, pins, fault injection, control override, or
second retry is permitted. Public evidence excludes all origins, hostnames,
ports, USB/network identity, credentials, workers, addresses, passwords,
tokens, checkpoint secrets, paths, and raw traces.
Plan verification: Ordered Cargo, Bright Builds, all 39 Bazel tests, parity,
progress, redaction, reference, generated contracts, selector, unique task,
immutable plan, reference cleanliness, and diff checks pass. Immutable PLAN.md
SHA-256 is `b9d055764d046233159226a12e9e44444f52a66d44ce2c83375ce692fd04e52b`.
Recovery is pause/safe-stop first and exact-package restore only if required;
the earliest failure remains primary. Accepted stop categories are
`hardware_blocked`, `evidence_invalid`, `timeout`, and `process_failed`.

Attempt-002 completion review: Exact pushed source and package
`2feff204fb7c4d2a2d3196e69a1dd8acf91bbfdb` passed every software and
detector gate. The independent fixture process reached readiness and exited
cleanly, resolving attempt-001's host blocker. The single hardware attempt then
stopped before serial observation as `hardware_blocked` / `flash_failed` with
the package admitted, runtime identity unobserved, zero captured serial bytes,
no network phase or IDENTIFY checkpoint, safe stop unobserved, USB cleanup
ready, private modes valid, and the public projection withheld. API-009 remains
`implemented`. See the linked `CLOSURE.md`. Resume only through a fresh
immutable plan after a protected flash-child diagnostic distinguishes the
failed boundary; attempt-002 cannot be retried.

Flash-boundary continuation plan:
`docs/parity/work-plans/20260812T161941Z-API-009/PLAN.md`.

- [x] Preserve the durable USB supervisor's typed factory/NVS child diagnostic
      through the campaign and bind only closed, redaction-safe facts into the
      protected result.
- [x] Bypass the confirmed RAM-stub `FlashDeflData` timeout with the supported
      `write-bin --no-stub` path for both factory and NVS writes.
- [x] Add real-child, command-shape, failure-precedence, private-mode, schema,
      and sensitive-output regressions; run and push every software gate before
      hardware.
- [x] Run at most one fresh detector-gated `attempt-003` and promote only on
      the complete five-command device-user quorum.

Attempt-003 authorization: standing task authorization permits this single
fresh attempt only after the immutable plan and exact clean pushed source prove
the material no-stub remediation and typed diagnostic path. Allowed effects,
prohibited effects, evidence privacy, exact-package admission, board-205
detector gate, 600-second local-fixture lease, one-time physical identify
checkpoints, safe stop, cleanup, canonical software restart, recovery, public
withholding, and accepted stop categories are exactly those recorded in the
linked plan. No second retry is permitted.

Attempt-003 completion review: Exact pushed source and package
`f8c279d25f0c4a3704bf1837a0eabef58df26410` passed every software and detector
gate. Both factory and NVS ROM-loader writes completed on their first attempt,
proving the no-stub remediation; trusted runtime identity, safe stop, USB
cleanup, private modes, and closed flash diagnostics also passed. The sole
attempt then stopped as `hardware_blocked` / `network_correlation_failed`
because the production runtime reported `stratum_v1_unsupported` before any
active network window or command-effect request. No IDENTIFY checkpoint or
restart occurred, the public projection was withheld, and API-009 remains
`implemented`. The linked closure requires a fresh immutable continuation that
types the production protocol-gate decision and reproduces it from the exact
campaign NVS image before any material fix or new ordinal.

Protocol-gate continuation plan:
`docs/parity/work-plans/20260812T170039Z-API-009/PLAN.md`.

- [x] Replace repeated exclusive default-NVS partition acquisition with one
      boot-lifetime owner whose clones are shared by settings, production
      campaign, protocol-gate, and scoreboard adapters.
- [x] Carry a closed protocol-gate decision through the production snapshot and
      campaign marker without selector values, pool data, network data, or raw
      logs.
- [x] Add exclusive-owner, generated-campaign-NVS, startup-race, recovery,
      schema, and sensitive-output regressions; run and push every software
      gate before hardware.
- [x] Run at most one fresh detector-gated `attempt-004` and promote only on
      the complete five-command device-user quorum.

Attempt-004 authorization: standing task authorization permits this single
fresh attempt only after the immutable plan and exact clean pushed source prove
the material boot-lifetime NVS ownership fix and typed protocol decision. The
allowed effects, prohibited effects, evidence privacy, exact-package and
board-205 detector gates, 600-second local-fixture lease, one-time physical
IDENTIFY checkpoints, safe stop, cleanup, canonical software restart,
recovery, public withholding, and accepted stop categories are exactly those
in the linked plan. Instrumentation alone is not retry eligibility, and no
second attempt is permitted.

Attempt-004 completion review: Exact pushed source and package
`2c603f34b391a0c14c8539724fb28444961798d7` passed every software, package,
privacy, and detector gate. Both no-stub writes completed on attempt one; the
runtime identity was trusted and the typed protocol gate was `ready`, resolving
attempt-003's blocker. The sole attempt observed a genuine positive block,
eight qualified candidates, and a confirmed pause, then stopped as
`hardware_blocked` / `network_correlation_failed` after the resume request
reported `safety_prerequisites_stale`. Resume, IDENTIFY, dismiss, and restart
did not complete, safe stop and cleanup passed, private modes remained valid,
and the public projection was withheld. API-009 remains `implemented`. The
linked closure requires a fresh immutable continuation to diagnose and test the
pause/resume safety-readiness transition before any new ordinal.

Safety-readiness continuation plan:
`docs/parity/work-plans/20260812T173427Z-API-009/PLAN.md`.

- [x] Bind one closed sampled-observation epoch and readiness transition to the
      production session and versioned campaign evidence.
- [x] Reproduce stale-resume followed by fresh producer notification at the
      production shell seam and fix only the confirmed missed transition.
- [x] Add ordering, coalescing, recovery, schema, and sensitive-output tests;
      run and push every software gate before hardware.
- [x] Run at most one fresh detector-gated `attempt-005` only after a red/green
      material fix, and promote only on the complete five-command quorum.

Attempt-005 authorization: standing task authorization permits this single
fresh attempt only after the immutable plan and exact clean pushed source prove
the production-shaped stale-then-fresh transition regression fails before and
passes after the material fix. The allowed effects, prohibited effects,
evidence privacy, exact-package and board-205 detector gates, 600-second
local-fixture lease, one-time physical IDENTIFY checkpoints, safe stop,
cleanup, canonical software restart, recovery, public withholding, and
accepted stop categories are exactly those in the linked plan. Closed
instrumentation alone is not retry eligibility, and no second attempt is
permitted.

Attempt-005 completion review: Exact pushed source and package
`afc4839967f820d144167cbb5c981ca66b2b5942` passed every software, package,
privacy, and detector gate. Both no-stub writes completed on attempt one,
runtime identity and protocol admission were trusted, and the sole campaign
observed a genuine positive block plus confirmed pause and resume under the
same boot and package. This resolves attempt-004's lost fresh-observation wake:
the session returned active after resume. The campaign then required the
one-time physical IDENTIFY-rendered observation, but rendering and clearing
were not confirmed before the bounded run closed. Its protected result
preserved `network_correlation_failed` with terminal
`safety_prerequisites_stale`; the public wrapper reported
`hardware_blocked`. Safe stop, USB cleanup, private modes, and evidence
withholding passed. No public projection exists, API-009 remains
`implemented`, and attempt-005 cannot be retried. See the linked closure for
the exact non-claims and next safe action.

Operator-checkpoint continuation plan:
`docs/parity/work-plans/20260812T182405Z-API-009/PLAN.md`.

- [x] Supervise the two private typed IDENTIFY requirements concurrently with
      the campaign child and emit only ordered, exactly-once closed stderr
      signals while retaining one final stdout envelope.
- [x] Add fake and real-child filesystem regressions for acknowledgement,
      settlement cancellation, malformed/missing inputs, failure precedence,
      private modes, and sensitive-output absence.
- [x] Run every focused and mandatory software gate; commit and push the exact
      implementation before package, detector, or hardware use.
- [x] Run at most one fresh detector-gated `attempt-006`, confirm each
      checkpoint only after the matching user-observed physical state, and
      promote only on the complete five-command quorum.

Attempt-006 authorization: standing task authorization permits this single
fresh attempt only after the immutable plan and exact clean pushed source prove
the production-shaped live checkpoint handoff. The allowed effects, prohibited
effects, evidence privacy, exact-package and board-205 detector gates,
600-second local-fixture lease, request-once confirmations, safe stop, cleanup,
canonical software restart, recovery, public withholding, and accepted stop
categories are exactly those in the linked plan. Campaign start consumes the
ordinal; no automatic, inferred, preemptive, or repeated confirmation and no
second attempt is permitted.

Attempt-006 completion review: Exact pushed source and package
`17835217b5ec1d9e7d33363cbf346a0d4762d332` passed every software, package,
privacy, and detector gate. The sole campaign observed a genuine positive
block, three qualified and accepted shares, confirmed pause and resume, and
active mining under the same boot and package. The new live checkpoint
supervisor then emitted the rendered request immediately, proving the host
handoff fix on hardware. No matching physical-observation reply arrived within
the 30-second IDENTIFY interval, so no confirmation command was issued and no
second IDENTIFY toggle, dismissal, or restart occurred. The protected campaign
later closed `network_correlation_failed` with terminal
`safety_prerequisites_stale`; the public wrapper reported
`hardware_blocked`. Safe stop, USB cleanup, private modes, and evidence
withholding passed. No projection exists, API-009 remains `implemented`, and
attempt-006 cannot be retried. See the linked closure for the next safe action.

Operator-ready continuation plan:
`docs/parity/work-plans/20260812T235141Z-API-009/PLAN.md`.

- [x] Record before effects that the operator is present at the device,
      watching the display, and asking to continue; do not treat this as either
      future IDENTIFY observation or as retroactive attempt-006 evidence.
- [x] Revalidate and push the immutable plan/task checkpoint plus every
      focused, mandatory, privacy, reference, and exact-package gate.
- [x] Run exactly one fresh detector-gated `attempt-007` with the proven live
      prompt, and consume each typed request-once confirmation only after the
      matching live signal and physical operator observation.
- [x] Promote API-009 only on the complete five-command device-user quorum;
      otherwise preserve `implemented`, the earliest typed failure, safe stop,
      cleanup, evidence withholding, and the accepted terminal outcome.

Attempt-007 authorization: the operator's current-thread report that they see
new information on the display and want to continue satisfies the attempt-006
closure's required pre-effect presence/watching/readiness occurrence. Standing
task authorization therefore permits one fresh attempt after the immutable
plan/task and exact clean pushed software/package gates pass. It does not
confirm either new IDENTIFY state. The exact permitted campaign command,
request-once confirmation commands, protected wrapper/detector/attempt roots,
allowed effects, prohibited effects, privacy classes, safety limits, recovery,
cleanup, evidence quorum, retry bound, and stop outcomes are frozen in the
linked plan. Campaign start consumes attempt-007; no inferred confirmation,
expired-checkpoint reuse, unchanged retry, second campaign, external pool,
owner credential, destructive/fault-injection action, power cycle, direct UART,
or pin/pad/GPIO manipulation is authorized.

Attempt-007 verification: `stop_repeated_boundary`; no retry was run. Exact
clean pushed source `ae24565a9376948bb0eeff190938403a1897c7e5`, focused real
boundary tests, all software/privacy/reference gates, and exact package
admission passed before effects. The fresh protected detector admitted exactly
one Ultra 205. The sole campaign used a fresh mode-0700 attempt root with 13
mode-0600 files and retained separate private wrapper/detector streams. Its
sealed v8 result proved trusted package/runtime identity, protocol readiness, a
genuine positive block, two qualified accepted shares, one pause request,
confirmed pause, one resume request, confirmed safe stop, ready USB cleanup,
private modes, and redaction. It emitted neither IDENTIFY requirement, no
confirmation command ran, and the public projection remained absent.

Completion review: Attempt-007 recurred at the exact attempt-004 command-effect
boundary after the targeted producer-wakeup/readiness fix had been verified and
had crossed it successfully in attempts 005/006: `network_correlation_failed` /
`safety_prerequisites_stale`, with active-before-pause true, pause confirmed,
resume requested but not confirmed, active-after-resume false, a deadline wake,
stale safety sample, unchanged observation epoch, and no recovered pending
observation. The hardware-attempt policy therefore selects
`stop_repeated_boundary`. API-009 remains `implemented`, this unresolved task
stays active, and attempt-008 is prohibited. The linked CLOSURE.md records the
terminal blocker, next safe software-only diagnostic condition, and non-claims.

Safe-stop correlation continuation plan:
`docs/parity/work-plans/20260813T061635Z-API-009/PLAN.md`.

- [x] Add a closed firmware-owned resumable-pause safe-stop state that becomes
      confirmed only after logical pause, stopped hardware, and the same armed
      lease coincide.
- [x] Require the command-effects host observer to join API-visible logical
      pause with that same-session serial confirmation before its one resume.
- [x] Prove ordering, either-arrival order, bounded missing joins, request-once
      behavior, reprepare clearing, malformed input, and sensitive-output
      handling at production-shaped boundaries.
- [x] Run and push every focused and mandatory gate, keep API-009 implemented,
      and close without hardware or checklist transition.

Continuation authorization: software, deterministic fixtures, local child
processes, builds, and repository verification only. Protected attempts,
credentials, detector, package effects, device/USB/network/HTTP sessions,
flash, reset, mining, ASIC initialization or traffic, hardware controls,
command effects, OTA, recovery, direct UART, pins, and attempt-008 remain
prohibited. This continuation supplies no hardware eligibility and cannot
promote API-009.

Verification: Marker v12 now carries a closed firmware-owned
`resumable_pause_safe_stop` state derived only from an active-seen
command-effects campaign, retained authorizing lease, paused operator intent,
armed campaign, and stopped hardware. The same-session serial coordinator
sets and clears that fact, while the host requires its join with logical HTTP
pause before one resume and fails closed at the exact 130-second boundary.
Focused Cargo/Bazel tests and the firmware build pass, followed by the ordered
Cargo, Bright Builds, all-42-target Bazel, parity, progress, redaction,
reference, source-ownership, immutable-plan, task, sensitive-output, and diff
gates.

Completion review: The nondeterministic post-pause boundary is explained and
fixed at its missing orchestration join without changing freshness policy or
hardware safety. API-009 remains `implemented`; no protected evidence,
credential, detector, package effect, device, network/HTTP session, mining,
hardware command, or attempt-008 was used. The linked `CLOSURE.md` records why
software completion is not parity verification and the condition for any
future separately selected work.

Hardware retry-evaluation plan:
`docs/parity/work-plans/20260813T070749Z-API-009/PLAN.md`.

- [x] Establish that the pushed marker-v12 pause/safe-stop join explains and
      materially changes the exact attempt-004/007 failure boundary, satisfying
      the prior closure's explicit retry-progress condition.
- [x] Commit and push the immutable one-attempt contract and pass every
      focused, mandatory, privacy, reference, and exact-package gate.
- [x] Run exactly one fresh detector-gated `attempt-008`, using only live
      user-observed IDENTIFY confirmations, and stop without attempt-009.
- [x] Promote API-009 only on the complete five-command device-user quorum;
      otherwise preserve `implemented`, earliest failure, public withholding,
      cleanup, recovery, and the truthful closed stop outcome.

Attempt-008 authorization: this separately selected plan explicitly
supersedes only the prior attempt-008 prohibition because commit `21f19e7a`
proves and fixes the missing same-session logical-pause/hardware-safe-stop join.
Standing task authorization permits the single campaign only after the linked
immutable plan/task checkpoint and every software, package, privacy, recovery,
and detector gate pass at clean pushed HEAD. The linked plan freezes the exact
repo command, fresh private roots, accepted effects, prohibited effects,
600-second bound, physical-observation checkpoints, evidence quorum, recovery,
cleanup, terminal categories, and stop outcomes. Campaign start consumes the
ordinal. No inferred observation, prior protected-artifact reuse, external or
owner pool, destructive/fault-injection action, direct UART, pin/pad/GPIO
manipulation, attempt-009, or unchanged retry is authorized.

Attempt-008 verification: exact clean pushed source and package
`d84e5e5d62d4dfe002a8edd489871fe96258a8a9` passed every focused, software,
privacy, reference, and detector gate. The sole campaign proved a genuine
positive block, five qualified accepted shares, confirmed pause, the new
same-session safe-stop join, confirmed resume, and active mining after resume
under the same boot/package. This resolves the exact attempt-004/007 boundary.
The supervisor then emitted the rendered IDENTIFY checkpoint, but no matching
physical reply arrived in its live window, so no confirmation command ran.
Safe stop, cleanup, the result seal, private modes, redaction, and public
withholding pass; no cleared request, dismissal, restart, or projection exists.

Completion review: API-009 remains `implemented` because the complete
five-command device-user quorum is absent. The closed outcome is
`stop_authority_boundary`, attempt-008 is consumed, and attempt-009 is not
authorized. The linked CLOSURE.md records the successful hardware validation
of the pause/safe-stop fix, the expired physical-observation boundary, the next
safe selector action, and all non-claims.

Operator-present attempt-009 plan:
`docs/parity/work-plans/20260813T085022Z-API-009/PLAN.md`.

- [x] Preserve the user's current pre-effect presence/watching/readiness report
      without treating it as either IDENTIFY observation or retroactive
      attempt-008 evidence.
- [x] Commit and push the immutable one-attempt contract and pass every
      focused, mandatory, privacy, reference, and exact-package gate.
- [x] Run exactly one fresh detector-gated `attempt-009`; relay each live
      request and consume its request-once confirmation only after the matching
      user-observed physical state.
- [x] Promote API-009 only on the complete five-command device-user quorum;
      otherwise preserve `implemented`, earliest failure, public withholding,
      safe stop, cleanup, and the truthful closed outcome.

Attempt-009 authorization: this separately selected plan supersedes only the
prior attempt-009 prohibition because the user's current report establishes
the missing pre-effect operator presence, display observation, and readiness.
It does not confirm either future IDENTIFY state. Standing task authorization
permits the single campaign only after the linked immutable plan/task
checkpoint and every software, package, privacy, recovery, and detector gate
pass at clean pushed HEAD. The linked plan freezes the exact repo command,
fresh private roots, allowed and prohibited effects, 600-second bound, live
physical-observation checkpoints, evidence quorum, recovery, cleanup, terminal
categories, and stop outcomes. Campaign start consumes attempt-009. No inferred
observation, expired-checkpoint reuse, external or owner pool, destructive or
fault-injection action, direct UART, pin/pad/GPIO manipulation, attempt-010, or
unchanged retry is authorized.

Attempt-009 verification: `stop_hardware_blocker`; no retry was run. Exact
clean pushed source and package `6cfb8b7316e18d8fab94b0e40d454b0b9acd80a1`,
focused and real-process tests, all software/privacy/reference gates, and the
sole protected detector passed. The sealed v8 result proved trusted identity,
protocol readiness, a genuine positive block, five qualified accepted shares,
one pause request, confirmed safe stop, ready USB cleanup, private modes,
matching digests, redaction, and public withholding. It stopped before pause
confirmation because the deadline readiness transition reported stale safety,
an unchanged observation epoch, and no recovered pending observation. No
resume, IDENTIFY request or confirmation, dismissal, restart, or public
projection occurred.

Completion review: API-009 remains `implemented`. Attempt-009 is consumed and
attempt-010 is prohibited. The boundary is earlier than attempt-008's proven
pause/safe-stop/resume path, so it is recorded as a distinct unresolved
runtime-safety blocker rather than an authority failure or unchanged retry.
The linked `CLOSURE.md` requires a production-shaped software diagnosis and
regression-backed fix before any future hardware contract.

Resumable-pause shutdown separation plan:
`docs/parity/work-plans/20260813T100428Z-API-009/PLAN.md`.

- [x] Add a closed resumable-versus-terminal hardware-stop purpose to the pure
      production session and its firmware adapter.
- [x] Keep the full terminal/fault/rollback cooling plan, but make operator
      pause complete after the immediate safe effects with full fan duty.
- [x] Reproduce and guard the attempt-009 boundary across session selection,
      actuation planning, sensor/wakeup ownership, campaign status, and the host
      pause join.
- [x] Pass every focused and mandatory software/privacy/reference gate, keep
      API-009 `implemented`, and close without hardware or checklist transition.

Software-only authorization: this plan may change and test repository source,
run deterministic fixtures and local child processes, build firmware, and run
repository verification. It may not read protected attempt contents, access
credentials, run the detector or package effects, interact with a device,
network, USB, HTTP endpoint, ASIC, mining session, hardware controls, command
effects, OTA, recovery, direct UART, pins/pads/GPIO, or attempt-010. A later
hardware ordinal requires this exact fix and its production-boundary regression
to pass, plus a fresh separately selected immutable hardware contract.

Software-fix completion review: The production session now emits a typed
`ResumablePause` or `Terminal` stop purpose. Resumable pause runs the six
immediate fail-closed effects and retains full fan duty; terminal, fault, and
preparation rollback retain the eight-step 45 C cooling proof and 30-percent
settlement. A production-session-to-actuation-to-same-lease-confirmation test,
focused session/actuation/ownership/campaign/host tests, the firmware build,
and every mandatory software/privacy/reference gate pass. API-009 remains
`implemented`; no hardware was contacted and no evidence was published. A
fresh immutable hardware contract may consider attempt-010, but this completed
software plan does not authorize it.

Resumable-pause attempt-010 plan:
`docs/parity/work-plans/20260813T110706Z-API-009/PLAN.md`.

- [x] Commit and push the immutable one-attempt contract and pass every
      focused, mandatory, privacy, reference, and exact-package gate.
- [x] Run exactly one fresh detector-gated `attempt-010`; relay each live
      IDENTIFY request and consume its request-once confirmation only after the
      matching user-observed physical state.
- [x] Promote API-009 only on the complete five-command device-user quorum;
      otherwise preserve `implemented`, earliest failure, public withholding,
      safe stop, cleanup, recovery, and the truthful closed outcome.

Attempt-010 authorization: this separately selected plan supersedes only the
prior attempt-010 prohibition because commit `ecb19811` fixes the exact
attempt-009 production boundary with a typed resumable stop purpose and a
composed same-lease confirmation regression. Standing task authorization
permits the single campaign only after the linked immutable plan/task
checkpoint and every named software, package, privacy, recovery, and detector
gate pass at clean pushed HEAD. The linked plan freezes the exact repo command,
fresh private roots, allowed and prohibited effects, 600-second bound, live
physical-observation checkpoints, evidence quorum, recovery, cleanup, terminal
categories, and stop outcomes. Campaign start consumes attempt-010. No inferred
observation, protected-artifact reuse, external or owner pool, destructive or
fault-injection action, direct UART, pin/pad/GPIO manipulation, attempt-011, or
unchanged retry is authorized.

Attempt-010 verification (corrected): `stop_authority_boundary`; no retry was
run. Exact clean pushed source/package
`8e89891f445f49493d3909fb2e1b4c30795c5dce`,
focused and real-process tests, every software/privacy/reference gate, and the
sole protected detector passed. The sealed v8 result proves trusted identity,
a genuine block, five accepted shares, confirmed pause and resumable safe stop,
confirmed resume, active-after-resume, private modes, matching seal, safe stop,
USB cleanup, redaction, and public withholding. The campaign emitted a
rendered IDENTIFY checkpoint without first establishing operator readiness or
explaining the expected `BITAXE IDENTIFY` / `Hello!` frame and 30-second
window. The user's later normal-screen description arrived after that window,
so it is neither positive nor negative render evidence and no confirmation
command ran. Cleared IDENTIFY, dismissal, restart, and the public projection
were correctly withheld.

Completion review: API-009 remains `implemented`; attempt-010 is consumed and
attempt-011 is prohibited. The hardware result validates the resumable-pause
fix but does not classify IDENTIFY rendering. The actionable blocker is the
host checkpoint contract: it triggers a time-bounded effect before readiness
and emits an under-specified signal. The linked `CLOSURE.md` preserves that
corrected boundary.

Operator-prearmed IDENTIFY plan:
`docs/parity/work-plans/20260813T144901Z-API-009/PLAN.md`.

- [x] Correct attempt-010's late physical report from a display-failure claim
      to an expired-observation authority boundary without changing its sealed
      aggregate, cleanup, withholding, or attempt-consumption facts.
- [x] Add a closed v2 `ready` / `rendered` / `cleared` checkpoint transaction
      whose public signals explain the exact frame and 30-second window.
- [x] Prove the campaign issues no IDENTIFY request before request-once operator
      readiness, preserves ordered confirmations and exactly two requests, and
      fails closed on malformed, reordered, or reused checkpoints.
- [x] Pass every focused and mandatory software/privacy/reference gate, keep
      API-009 `implemented`, and close without hardware or attempt-011.

Software-only authorization: this plan may change and test repository source,
run deterministic fixtures and local child processes, build firmware, and run
repository verification. It may not access credentials, package/detector/USB
effects, device or network sessions, mining, HTTP command effects, physical
display observations, restart/recovery, OTA, destructive or fault-injection
actions, direct UART, pins/pads/GPIO, attempt-011, public parity evidence, or a
checklist transition.

Pre-armed checkpoint completion review: `software_fix_complete`. Private v2
checkpoints now require request-once operator readiness before the first
IDENTIFY request, retain ordered rendered/cleared confirmations, and add the
readiness fact to the v2 command-evidence quorum. Public v2 signals name the
exact four-line frame, 30-second duration, and closed confirmation condition.
Focused campaign/CLI/checkpoint/orchestration and real-child regressions plus
every mandatory software, privacy, reference, parity, and firmware-build gate
pass. The attempt-010 record is corrected to `stop_authority_boundary` without
changing its sealed hardware or cleanup facts. API-009 remains `implemented`;
no hardware was accessed, no public evidence was produced, and attempt-011
requires a separate future contract.

Pre-armed IDENTIFY attempt-011 plan:
`docs/parity/work-plans/20260813T154249Z-API-009/PLAN.md`.

- [x] Commit and push the immutable single-attempt contract and pass every
      focused, mandatory, privacy, reference, and exact-package gate.
- [x] Run exactly one fresh detector-gated attempt-011, consume readiness
      before IDENTIFY, and consume rendered/cleared only after matching live
      physical reports.
- [x] Promote API-009 only on the complete command/restart/device-user quorum;
      otherwise preserve `implemented`, earliest failure, public withholding,
      safe stop, cleanup, recovery, and stop without attempt-012.

Attempt-011 authorization: this separately selected plan supersedes only the
prior attempt-011 prohibition because pushed commit `9d632439` fixes the exact
attempt-010 authority boundary with a pre-effect request-once readiness gate,
self-describing 30-second public signal, and production/real-process
regressions. Standing task authorization permits this single campaign only
after the linked immutable plan/task checkpoint and every named software,
package, privacy, recovery, and detector gate pass at clean pushed HEAD. The
linked plan freezes the exact repo command, fresh private roots, effects,
physical checkpoints, evidence quorum, recovery, cleanup, retry bound, and
stop conditions. Campaign start consumes attempt-011. No inferred or expired
observation, protected-artifact reuse, external or owner pool, destructive or
fault-injection action, direct UART, pin/pad/GPIO manipulation, attempt-012,
or unchanged retry is authorized.

Attempt-011 closure: exact pushed package and runtime identity passed, a
genuine block and four accepted shares preceded one pause request, and the
sealed result stopped before pause confirmation as `network_correlation_failed`
with readiness reason `safety_prerequisites_stale`. No readiness signal or
IDENTIFY request occurred. Recovery confirmed safe stop and cleanup without a
secondary failure; evidence was withheld and API-009 remains `implemented`.

Software-only stale-safety continuation:

Plan: `docs/parity/work-plans/20260813T160905Z-API-009/PLAN.md`.

- [x] Explain why the command-effects pause join observed stale safety while
      the campaign result still reported fresh safety and a clean active marker
      stream.
- [x] Implement the smallest root-cause fix with focused transition,
      pause-join, orchestration, and real-process regressions.
- [x] Run every mandatory software/privacy/reference gate and keep attempt-012
      prohibited until a separate immutable hardware contract is justified.

Authorization: read-only source, current attempt-011 typed metadata, and
software tests/builds only. Do not read raw traces or sensitive fields, publish
protected artifacts, access credentials, USB, the device or its network,
issue HTTP commands, flash, reset, mine, manipulate hardware controls, use
direct UART or pins/pads/GPIO, or create/run attempt-012.

Completion review: the pause request was lost because command intent shared the
replaceable session-derived mining projection. Requested intent now has a
separate typed boot-lifetime owner that session publication cannot overwrite.
The exact interleaving and ownership regressions plus all mandatory gates and a
real ESP firmware build pass. API-009 remains `implemented`; no hardware or
parity evidence was produced by the software-only continuation.

Operator-intent-fixed attempt-012 plan:
`docs/parity/work-plans/20260813T163921Z-API-009/PLAN.md`.

- [x] Commit and push the immutable single-attempt contract and pass every
      focused, mandatory, privacy, reference, firmware, and exact-package gate.
- [x] Run exactly one fresh detector-gated attempt-012 and consume each
      IDENTIFY checkpoint only after its matching live physical report.
- [x] Promote only on the complete command/restart/device-user quorum;
      otherwise preserve `implemented`, earliest failure, evidence withholding,
      safe stop, cleanup, recovery, and stop without attempt-013.

Attempt-012 authorization: pushed commit `f9add8e2` fixes the exact
attempt-011 lost-pause ownership race with a separate boot-lifetime requested
intent owner and regression-backed production readiness routing. Standing task
authorization permits this one campaign only after the linked immutable
contract and all named gates pass at clean pushed HEAD. Campaign start consumes
attempt-012. No inferred observation, prior protected-artifact reuse, external
or owner pool, destructive or fault-injection action, direct UART,
pin/pad/GPIO manipulation, attempt-013, or unchanged retry is authorized.

Attempt-012 completion review: the immutable contract and every named
software, privacy, reference, firmware, package, fresh-path, and detector gate
passed. The sole campaign reached the two post-flash USB boundaries and monitor
admission, but the 810-second parent deadline expired before the child's own
600-second observation plus 180-second terminal-grace window could close. No
IDENTIFY checkpoint was emitted or confirmed. The typed primary category is
`timeout`; safe stop, final cleanup, and recovery are unconfirmed, the current
port is holder-free, the public projection is absent, and API-009 remains
`implemented`. Attempt-013 is not authorized. The next safe work is a
software-only deadline-contract and timeout-recovery fix with a regression that
binds the parent budget above the complete child transaction.

Deadline-contract software plan:
`docs/parity/work-plans/20260813T174155Z-API-009/PLAN.md`.

- [x] Define one typed command-effects child transaction envelope that covers
      bounded USB preparation, observation, terminal grace, and termination.
- [x] Derive the parent process timeout from that envelope and require it to
      remain strictly larger than the child's maximum normal lifetime.
- [x] Preserve primary timeout precedence while collecting any closed campaign
      recovery facts, and add unit plus real-process regressions.
- [x] Run every focused, mandatory, privacy, reference, and real firmware gate;
      close with API-009 still `implemented` and no hardware authorization.

Authorization: software-only source, tests, task, and public work-plan changes.
Do not access credentials, the detector, protected attempt contents beyond the
already recorded public closure facts, USB, device/network interfaces, HTTP,
mining, display, restart, controls, direct UART, pins/pads/GPIO, or attempt-013.

Deadline-contract completion review: the fixed 810-second parent and
920-second fixture deadlines are removed. One checked budget now covers the
version probe, three USB commands with their one allowed retry and recovery,
the 600-second observation, 180-second terminal grace, final USB cleanup, and
process termination. The parent is strictly larger than the 3,250-second child
maximum and the fixture is larger still. Cross-language source contracts,
scaled real-process cleanup, closed/missing/malformed timeout-recovery cases,
focused tests, every mandatory gate, and the real ESP build pass. Primary
`timeout` precedence is unchanged and recovery facts remain safe false unless
closed private artifacts prove them. API-009 stays `implemented`; no hardware
evidence or attempt-013 authorization was produced.

Deadline-fixed attempt-013 plan:
`docs/parity/work-plans/20260813T180631Z-API-009/PLAN.md`.

- [x] Commit and push the immutable single-attempt contract and pass every
      focused, mandatory, privacy, reference, firmware, and exact-package gate.
- [x] Run exactly one fresh detector-gated attempt-013 and consume each
      IDENTIFY checkpoint only after its matching live physical report.
- [x] Promote only on the complete command/restart/device-user quorum;
      otherwise preserve `implemented`, earliest failure, evidence withholding,
      safe stop, cleanup, recovery, and stop without attempt-014.

Attempt-013 authorization: pushed commit `257e3be9` fixes the exact attempt-012
parent/child deadline mismatch with a checked complete-transaction budget and
timeout-recovery regressions. Standing task authorization permits this one
campaign only after the linked immutable contract and all named gates pass at
clean pushed HEAD. Campaign start consumes attempt-013. No inferred physical
observation, prior protected-artifact reuse, external or owner pool,
destructive/fault-injection action, direct UART, pin/pad/GPIO manipulation,
attempt-014, or unchanged retry is authorized.

Attempt-013 completion review: every pre-effect gate passed at exact clean
pushed source, the detector admitted exactly one holder-free board-205
ESP32-S3, and the one authorized campaign consumed attempt-013. Factory and
NVS flashing both closed `ready`; the runtime identity was trusted, safety was
fresh, and USB cleanup closed `ready`. The command-effects observation then
closed `terminal_state_unconfirmed` before any notification, pause, resume, or
IDENTIFY request/checkpoint. The public wrapper preserved `hardware_blocked`,
safe stop remains unconfirmed, no recovery was attempted, private modes pass,
no process or port holder remains, and the public projection is absent.
API-009 remains `implemented`; attempt-014 is not authorized. See
`docs/parity/work-plans/20260813T180631Z-API-009/CLOSURE.md`.

Resumable-epoch diagnosis and software-fix plan:
`docs/parity/work-plans/20260814T014014Z-API-009/PLAN.md`.

- [x] Reproduce attempt-013's pre-active expiry at the public production-
      session seam and preserve it as a focused red-to-green regression.
- [x] Split activation and resumable observation into separately bounded typed
      clocks, keeping one-shot identity and pause/resume safe-stop semantics.
- [x] Carry the larger child envelope through Rust capture and TypeScript
      supervision, with a distinct typed activation-timeout terminal category.
- [x] Pass every focused, mandatory, privacy, reference, firmware, selector,
      and diff gate; close and push without hardware or a parity transition.

Authorization: software-only source, synthetic tests, task, and public work-
plan changes. Do not access credentials, detector or protected raw traces;
USB, device or network interfaces; HTTP commands; flash, reset, restart,
mining, display, hardware controls, direct UART, pins/pads/GPIO, evidence
promotion, or attempt-014.

Resumable-epoch completion review: the public Production Mining Session
regression first failed with the campaign consumed before its first active
state. Command effects now has a bounded activation phase followed by one
continuous resumable active epoch; the activation deadline is disabled after
activation, pause/resume does not reset the epoch, and exact activation timeout
has its own closed blocker and host category. Rust capture and the checked
TypeScript parent/fixture budget cover activation, observation, terminal grace,
USB, cleanup, and process termination. The original ignored harness turned
green and was removed. Focused boundary, pause/resume, mapping, marker,
cross-language, and real-process tests pass together with every mandatory,
privacy, reference, firmware, selector, sensitive-output, and diff gate.
API-009 remains `implemented`; no hardware or attempt-014 ran. See the linked
`CLOSURE.md` for the remaining hardware evidence condition and non-claims.

Resumable-epoch-fixed attempt-014 plan:
`docs/parity/work-plans/20260814T023531Z-API-009/PLAN.md`.

- [x] Commit and push the immutable single-attempt contract and pass every
      focused, mandatory, privacy, reference, firmware, and exact-package gate.
- [x] Run exactly one fresh detector-gated attempt-014 and consume each
      IDENTIFY checkpoint only after its matching live physical report.
- [x] Promote only on the complete command/restart/device-user quorum;
      otherwise preserve `implemented`, earliest failure, evidence withholding,
      safe stop, cleanup, recovery, and stop without attempt-015.

Attempt-014 authorization: pushed commit `0ae08421` reproduces and fixes the
exact attempt-013 pre-active lease-expiry boundary with separately bounded
activation and resumable-active epochs, a typed activation-timeout terminal
category, expanded child/parent/fixture bounds, exact-boundary regressions, and
a real-process guard. Standing task authorization permits this one campaign
only after the linked immutable contract and all named gates pass at clean
pushed HEAD. Campaign start consumes attempt-014. No inferred physical
observation, prior protected-artifact reuse, external or owner pool,
destructive/fault-injection action, direct UART, pin/pad/GPIO manipulation,
attempt-015, or unchanged retry is authorized.

Attempt-014 closure review: exact pushed source `5d8108c2`, package validation,
one detector, and both exact-package flash writes passed. Runtime identity and
the protocol gate were trusted, the local fixture closed cleanly, and USB
cleanup was ready. The campaign nevertheless retained only two milliseconds of
active state before the persisted paused intent reasserted, then expired as
`safety_stale` without a genuine notification or any IDENTIFY checkpoint.
Public evidence remains withheld, safe stop remains unconfirmed, API-009 stays
`implemented`, and no attempt-015 is authorized. The production seam and its
existing host test confirm that command-effects startup overrides pause only
until the first active snapshot, after which the unchanged boot-time paused
intent immediately stops hardware. The next safe action is a new software-only
plan that gives the command-effects lease an initial run intent while keeping
mine-on-boot disabled and preserving explicit pause/resume command ownership.

Command-effects startup-intent fix plan:
`docs/parity/work-plans/20260814T033105Z-API-009/PLAN.md`.

- [x] Commit and push the immutable software-only plan after all plan gates.
- [x] Add the real-seam red regression, implement the lease-scoped initial run
      request, and preserve explicit pause/resume plus terminal pause behavior.
- [x] Pass focused and mandatory gates, close the software plan with API-009
      still `implemented`, and leave attempt-015 unauthorized.

This continuation authorizes no hardware effect. It isolates the current-boot
requested-intent owner from persisted `mineonboot=0` without changing NVS,
automating a command, or weakening safety. A later hardware ordinal requires a
separate clean selector and immutable bounded plan after this fix is pushed.

Startup-intent fix closure review: a real-seam regression first failed because
the command-effects lease had no current-boot run bootstrap. The production
owner now applies one typed bootstrap only for an authorizing command-effects
lease, after tracker construction and before readiness is observed. Disabled
mine-on-boot remains persisted and ordinary campaigns remain unchanged;
explicit pause/resume commands still replace the request, and consumed leases
still force safe pause. Focused and mandatory gates pass. No hardware ran,
API-009 remains `implemented`, and attempt-015 requires a separate clean
selector plus immutable bounded plan.

Startup-intent-fixed attempt-015 plan:
`docs/parity/work-plans/20260814T035526Z-API-009/PLAN.md`.

- [x] Commit and push the immutable single-attempt contract and pass every
      focused, mandatory, privacy, reference, firmware, and exact-package gate.
- [x] Run exactly one fresh detector-gated attempt-015 and consume each
      IDENTIFY checkpoint only after its matching live physical report.
- [x] Promote only on the complete command/restart/device-user quorum;
      otherwise preserve `implemented`, earliest failure, evidence withholding,
      safe stop, cleanup, recovery, and stop without attempt-016.

Attempt-015 authorization: pushed commit `13f75265` reproduces and fixes the
exact attempt-014 startup-intent race with a lease-scoped current-boot run
bootstrap before production-owner readiness. Persisted mine-on-boot remains
disabled, later explicit pause/resume stays authoritative, ordinary campaigns
do not opt in, and consumed leases force pause. Standing task authorization
permits this one campaign only after the linked immutable contract and all
named gates pass at clean pushed HEAD. Campaign start consumes attempt-015. No
inferred physical observation, protected-artifact reuse, external or owner
pool, destructive/fault-injection action, direct UART, pin/pad/GPIO
manipulation, attempt-016, or unchanged retry is authorized.

Attempt-015 closure review: exact pushed source `843ac0c9`, package validation,
one detector, and both exact-package flash writes passed. Runtime identity and
the protocol gate were trusted, safe stop and USB cleanup were confirmed, and
no holder or related process remained. The campaign emitted no consumed
physical checkpoint and closed `marker_invalid`: its first serial candidate
was a 1,490-byte malformed ingress line at the stream boundary, followed by
144 accepted markers measuring 2,516–2,578 bytes. The live receive path feeds
its first chunk directly to line analyzers without discarding the untrusted
pre-open fragment. Public evidence remains withheld, API-009 stays
`implemented`, and attempt-016 is not authorized. The next safe action is a
software-only plan for shared serial line-boundary admission before both the
campaign and network analyzers.

Receive-ingress line-admission fix plan:
`docs/parity/work-plans/20260814T042002Z-API-009/PLAN.md`.

- [x] Commit and push the immutable software-only plan after all plan gates.
- [x] Add the real receive-session red regression and implement one resettable
      newline-boundary admission before every ephemeral analyzer callback.
- [x] Prove campaign and network consumers share only admitted bytes across
      initial open and reader reacquisition, then pass every focused and
      mandatory gate with API-009 still `implemented`.

This continuation authorizes source, tests, task, and public plan artifacts
only. It must not access credentials, protected attempt contents beyond the
already recorded category/count/length facts, the detector, USB, device or
network interfaces, HTTP, display, mining, restart, hardware controls, direct
UART, pins/pads/GPIO, evidence promotion, or attempt-016.

Receive-ingress fix closure review: the real target first failed because no
line-admission owner existed. A focused allocation-free owner now discards
bytes through the first newline after every ephemeral reader open/reopen, then
passes later chunks unchanged through the existing single callback fan-out.
Ordinary retained monitor capture remains unchanged, while every malformed
post-admission marker remains terminal. Boundary, split-chunk, pass-through,
reopen, source-ownership, flash, automation, real-process, firmware, mandatory,
privacy, reference, selector, and diff gates pass. No hardware ran; API-009
remains `implemented`, and attempt-016 requires a separate clean selector plus
immutable bounded plan.

Receive-ingress-fixed attempt-016 plan:
`docs/parity/work-plans/20260814T044503Z-API-009/PLAN.md`.

- [x] Commit and push the immutable single-attempt contract and pass every
      focused, mandatory, privacy, reference, firmware, and exact-package gate.
- [x] Run exactly one fresh detector-gated attempt-016 and consume each
      IDENTIFY checkpoint only after its matching live physical report.
- [x] Promote only on the complete command/restart/device-user quorum;
      otherwise preserve `implemented`, earliest failure, evidence withholding,
      safe stop, cleanup, recovery, and stop without attempt-017.

Attempt-016 authorization: pushed commit `0fdff982` reproduces and fixes the
exact attempt-015 first-line ingress failure with one newline-boundary admission
reset on every ephemeral reader open/reopen. Downstream malformed evidence
remains terminal, and ordinary retained monitor capture is unchanged. Standing
task authorization permits this one campaign only after the linked immutable
contract and all named gates pass at clean pushed HEAD. Campaign start consumes
attempt-016. No inferred physical observation, protected-artifact reuse,
external or owner pool, destructive/fault-injection action, direct UART,
pin/pad/GPIO manipulation, attempt-017, or unchanged retry is authorized.

Completion review: attempt-016 admitted the exact package and trusted runtime,
observed 286 valid markers, reached the operator-ready checkpoint, and then
failed closed when no timely physical-ready report arrived. Its typed terminal
facts are `network_correlation_failed` / `safety_prerequisites_stale`; safe stop
is confirmed, USB cleanup is ready, the campaign process is absent, and the
public projection is withheld. A later ready confirmation was recorded only
after campaign closure and was never consumed, so it supplies no physical
IDENTIFY evidence. API-009 remains `implemented`; attempt-017 and an unchanged
retry are not authorized. See
`docs/parity/work-plans/20260814T044503Z-API-009/CLOSURE.md`.

Paused asynchronous IDENTIFY signal plan:
`docs/parity/work-plans/20260814T120645Z-API-009/PLAN.md`.

- [x] Move the one-shot ready checkpoint after the pause/safe-stop join and
      before resume, so the device remains safe-stopped while awaiting an
      asynchronous user or agent signal.
- [x] Add a checked one-hour ready window plus an explicit repo-owned signal-
      sender command, preserving exact ordering, private modes, redaction, and
      no resume or IDENTIFY effect before the signal.
- [x] Pass focused, real-process, firmware, mandatory, privacy, reference,
      selector, and diff gates; close with API-009 `implemented` and no hardware
      or attempt-017 authorization.

Software-only authorization: this plan may modify and test repository source,
run deterministic fixtures and local child processes, and build firmware. It
may not access credentials, detector/protected attempt contents beyond recorded
redacted closure facts, USB, device/network interfaces, HTTP commands, flash,
reset, restart, mining, display, hardware controls, direct UART,
pins/pads/GPIO, public parity evidence, attempt-017, or checklist status.

Completion review: pushed source `fe0995fd` keeps the command-effects device
paused and safe-stopped after its logical/serial pause join, publishes the
private ready checkpoint, and waits within a checked one-hour operator window.
Only one valid private signal may issue resume; the first IDENTIFY request now
waits for same-session active recovery. The repo-owned
`signal-api-command-identify` command sends ready/rendered/cleared signals for
the named private campaign root without exposing the path publicly. Focused
state-machine, replay/malformed, CLI, cross-language budget, checkpoint real-
child, automation, firmware, mandatory, privacy, reference, selector, and diff
gates pass. API-009 remains `implemented`; no hardware ran and attempt-017
requires a separate clean immutable plan. See
`docs/parity/work-plans/20260814T120645Z-API-009/CLOSURE.md`.

Paused-signal-fixed attempt-017 plan:
`docs/parity/work-plans/20260814T123336Z-API-009/PLAN.md`.

- [x] Commit and push the immutable single-attempt contract and pass every
      focused, mandatory, privacy, reference, firmware, and exact-package gate.
- [x] Run exactly one fresh detector-gated attempt-017, keep the device paused
      and safe-stopped while waiting, and send each private checkpoint only
      after its matching live physical report.
- [x] Promote only on the complete command/restart/device-user quorum;
      otherwise preserve `implemented`, earliest failure, evidence withholding,
      safe stop, cleanup, recovery, and stop without attempt-018.

Attempt-017 authorization: pushed source `fe0995fd` fixes attempt-016's active
operator-wait boundary with a one-hour paused safe-stop window, private one-shot
sender, resume-after-ready ordering, and IDENTIFY-after-active recovery backed
by focused and real-process regressions. Standing task authorization permits
this one campaign only after the linked immutable contract and all named gates
pass at clean pushed HEAD. Campaign start consumes attempt-017. No inferred or
expired physical observation, protected-artifact reuse, external or owner pool,
destructive/fault-injection action, direct UART, pin/pad/GPIO manipulation,
attempt-018, or unchanged retry is authorized.

Completion review: pushed plan commit `f0bdb192` passed every named software,
privacy, reference, firmware, selector, and exact-package gate. The sole fresh
detector admitted one ready Ultra 205, and attempt-017 completed both factory
and NVS flashes before serial observation. It then failed closed before the
ready checkpoint as `network_target_unavailable`: serial observation accepted
the paused terminal marker but captured zero runtime-attestation candidates,
so no trusted same-session runtime origin or network worker could be admitted.
Safe stop is confirmed, USB cleanup is ready, recovery was not required, and
the public projection is withheld. No ready/rendered/cleared signal was sent,
no physical IDENTIFY claim is made, API-009 remains `implemented`, and this
contract authorizes no attempt-018. See
`docs/parity/work-plans/20260814T123336Z-API-009/CLOSURE.md`.

Post-attempt-017 runtime-admission investigation:

- [x] Reproduce the pre-checkpoint failure with a temporary fast diagnostic at
      the production startup/campaign boundary, without requiring a permanent
      regression test.
- [x] Trace and falsify the startup, Wi-Fi, platform-readiness, recurring
      attestation, receive-only admission, and terminal-ordering hypotheses.
- [x] Implement the smallest confirmed root-cause fix, remove temporary
      instrumentation, and pass focused plus mandatory firmware/repository
      verification before committing and pushing.

Software-only authorization: inspect the protected attempt-017 artifacts only
through the already admitted categorical/count/timing facts, modify repository
source and tests, run local deterministic diagnostics, and build firmware. Do
not read credentials or raw traces; access USB, device/network interfaces, or
HTTP; flash, reset, restart, mine, actuate controls, manipulate direct UART or
pins/pads/GPIO; publish parity evidence; promote API-009; or create/run
attempt-018.

Completion review: the focused production-order diagnostic failed twice before
the fix because Wi-Fi admission was hidden inside `start_runtime_services` and
therefore necessarily preceded `publish_platform_readiness`. ESP-IDF's blocking
Wi-Fi adapter can wait indefinitely for driver-start events, while attempt-017
captured zero runtime-attestation candidates during its 305-second lifetime.
The fix splits network admission into `start_network_services`, starts storage
and the HTTP route shell, and begins recurring exact-package attestation before
entering Wi-Fi admission; the existing network-change notification still runs
after Wi-Fi returns. HTTP independently initializes the ESP network stack, and
the existing campaign tracker already admits two same-session attestations
before a later matching runtime origin. The diagnostic turns green, focused
boot-evidence/Wi-Fi/flash suites and the real firmware build pass, and the full
Cargo, Bright Builds, Bazel, parity, privacy, reference, firmware, and diff
sequence passes. No temporary debug markers remain, no hardware or protected
input was accessed, API-009 remains `implemented`, and attempt-018 remains out
of scope.

Startup-order-fixed attempt-018 plan:
`docs/parity/work-plans/20260814T143040Z-API-009/PLAN.md`.

- [x] Commit and push the immutable single-attempt contract and pass every
      focused, mandatory, privacy, reference, firmware, selector, and exact-
      package gate.
- [x] Run exactly one fresh detector-gated attempt-018, keep the device paused
      and safe-stopped while waiting, and send each private checkpoint only
      after its matching live physical report.
- [x] Promote only on the complete command/restart/device-user quorum;
      otherwise preserve `implemented`, earliest failure, evidence withholding,
      safe stop, cleanup, recovery, and stop without attempt-019.

Attempt-018 authorization: pushed source `16ee4ce8` fixes attempt-017's exact
zero-attestation boundary by publishing recurring platform readiness before
potentially unbounded ESP-IDF blocking Wi-Fi admission; `eacaea0f` documents
the invariant, and focused plus mandatory gates pass. Standing task
authorization permits this one campaign only after the linked immutable
contract and all named gates pass at clean pushed HEAD. Campaign start consumes
attempt-018. No inferred or expired physical observation, protected-artifact
reuse, external or owner pool, factory reset, destructive/fault-injection
action, direct UART, pin/pad/GPIO manipulation, attempt-019, or unchanged retry
is authorized.

Completion review: attempt-018 admitted the exact package and one Ultra 205;
both flashes completed once as `ready`, and runtime attestation is `trusted`,
proving the startup-order repair resolved attempt-017's zero-attestation
boundary. The campaign proved the genuine notification, positive block count,
pause and safe stop, live ready signal, resume and active recovery, two
IDENTIFY requests, and the live rendered observation. It then stopped as
`network_target_unavailable` / `safety_prerequisites_stale` after 40,707 ms
active: all five Ultra 205-required observations were fresh, while
`vr_temp_celsius` was correctly marked not required; the aggregate mining
safety sample itself was stale. The cleared signal was sent after its live
physical report but was not consumed before the terminal transition; dismiss
and restart did not run.
Safe stop, recovery without a secondary failure, USB cleanup, private modes,
process cleanup, and evidence withholding are confirmed. API-009 remains
`implemented`; no attempt-019 is authorized. See
`docs/parity/work-plans/20260814T143040Z-API-009/CLOSURE.md`.

Post-attempt-018 active-observation freshness investigation:

- [x] Build and run a fast deterministic red-capable diagnostic for the exact
      active IDENTIFY-window `safety_prerequisites_stale` transition.
- [x] Minimize the failing sequence and test ranked sampling, scheduling,
      campaign-ordering, and observation-admission hypotheses one at a time.
- [x] Implement the smallest confirmed root-cause fix, remove temporary
      instrumentation, and pass focused plus mandatory repository verification
      before committing and pushing.

Software-only authorization: use the already admitted attempt-018 categorical,
boolean, count, and bounded-duration facts; inspect and modify repository source
and tests; run deterministic fixtures and local child processes; and build
firmware. Do not read credentials, protected sensor values, origins, ports,
USB/network identities, or raw traces; access USB, device/network interfaces,
or HTTP; flash, reset, restart, mine, actuate controls, manipulate direct UART
or pins/pads/GPIO; publish parity evidence; promote API-009; or create/run
attempt-019.

Completion review: the red diagnostic failed twice because consuming the
operator-ready signal also released the initial safe-stop pause, leaving mining
active throughout both unbounded physical IDENTIFY observation windows. The
attempt-018 projection independently falsified missing-sensor and parser
hypotheses: all five Ultra 205-required observations were fresh and
`vr_temp_celsius` was correctly not required. The smallest fix keeps the single
pause held while ready, rendered, and cleared signals are consumed, sends both
IDENTIFY requests while paused, resumes exactly once only after the cleared
signal, confirms active recovery, and then dismisses the notification. The red
diagnostic is green, focused flash and real-child automation tests pass, and
the complete Cargo, Bright Builds, Bazel, parity, privacy, reference, firmware,
and diff sequence passes. No temporary instrumentation, protected input,
hardware access, attempt-019, evidence publication, or status promotion
occurred; API-009 remains `implemented` pending a separately planned hardware
attempt.

Pause-held attempt-019 plan:
`docs/parity/work-plans/20260814T152151Z-API-009/PLAN.md`.

- [x] Commit and push the immutable single-attempt contract and pass every
      focused, mandatory, privacy, reference, firmware, selector, and exact-
      package gate.
- [x] Run exactly one fresh detector-gated attempt-019, keep the device paused
      and safe-stopped through both physical IDENTIFY observations, and send
      each private checkpoint only after its matching live physical report.
- [x] Promote only on the complete command/restart/device-user quorum;
      otherwise preserve `implemented`, earliest failure, evidence withholding,
      safe stop, cleanup, recovery, and stop without attempt-020.

Attempt-019 authorization: pushed source `5fb8f4ce` deterministically proves
and fixes attempt-018's active observation-window failure by retaining the
single safe-stop pause through ready, rendered, and cleared, then resuming once
before dismissal. Focused plus mandatory gates pass. Standing task
authorization permits this one campaign only after the linked immutable
contract and all named gates pass at clean pushed HEAD. Campaign start consumes
attempt-019. No inferred or expired physical observation, protected-artifact
reuse, external or owner pool, factory reset, destructive/fault-injection
action, direct UART, pin/pad/GPIO manipulation, attempt-020, or unchanged retry
is authorized.

Completion review: attempt-019 admitted the exact package and one Ultra 205;
both flashes completed once as `ready`, runtime attestation was `trusted`, and
the device reached one confirmed pause before the operator-ready checkpoint.
After the live ready signal, the first IDENTIFY request succeeded and the
rendered checkpoint opened two seconds later, but the display effect retained
its upstream 30-second expiry while the assistant-mediated physical response
path remained unbounded. The user truthfully reported that the frame was not
visible, so no rendered or cleared signal was sent and no resume, dismiss, or
restart occurred. The owned signal supervisor closed the now-impossible
campaign; the typed result is `terminal_state_unconfirmed` / `operator_paused`,
with pause confirmed but terminal safe stop only `pending`, no recovery pause
request, USB cleanup `ready`, private modes valid, all attempt processes absent,
and public evidence withheld. API-009 remains `implemented`; no attempt-020 is
authorized. See
`docs/parity/work-plans/20260814T152151Z-API-009/CLOSURE.md`.

Post-attempt-019 checkpoint-expiry and decline investigation:

- [x] Build a deterministic red diagnostic proving a rendered checkpoint
      cannot outlive the 30-second IDENTIFY effect without becoming invalid.
- [x] Add a typed negative/expired checkpoint path that triggers prompt pause-
      preserving closure, recovery accounting, cleanup, and evidence
      withholding without a raw process signal.
- [x] Make time-critical IDENTIFY activation originate from the local operator
      signal command, pass focused and mandatory verification, and keep
      attempt-020 out of scope.

Software-only authorization: use attempt-019 categorical, boolean, count, and
bounded-duration facts; inspect and modify repository source and tests; run
deterministic fixtures and local child processes; and build firmware. Do not
read credentials, protected values, origins, ports, USB/network identities, or
raw traces; access USB, device/network interfaces, or HTTP; flash, reset,
restart, mine, actuate controls, manipulate direct UART or pins/pads/GPIO;
publish parity evidence; promote API-009; or create/run attempt-020.

Completion review: the red regression failed at the production checkpoint seam
because a truthful negative observation had no typed representation and the
rendered checkpoint remained open after the device's exact 30-second effect
expired. The signal command now accepts closed `confirmed` or `declined`
outcomes, the rendered state expires at the exact effect deadline, and either
negative terminal issues the one recovery pause before asking the owned serial
capture to stop. The public v3 checkpoint signal marks the ready command as the
local activation boundary, publishes the 30-second rendered response limit,
and advertises typed decline support. Recovery evidence claims a paused safe
stop only from the joined pause-confirmed, never-resumed, operator-paused, and
single-recovery-request facts; older explicit safe-stop records remain valid.
The red test is green, real-child and focused Bazel suites pass, and the full
Cargo, Bright Builds, canonical Bazel, parity, redaction, reference, and
firmware build gates pass. No protected input, hardware, network, HTTP, USB,
attempt-020, parity evidence, or status promotion was used; API-009 remains
`implemented` pending a separately planned attempt.

Asynchronous human-checkpoint lifetime repair:

- [x] Prove at the real host-orchestration seam that an operator-gated campaign
      cannot inherit a guessed one-hour or parent-process deadline.
- [x] Keep ready and other safe human waits unbounded while retaining exact
      bounds for automated safety, recovery, protocol, and finite physical-
      effect evidence windows.
- [x] Add a repo-local asynchronous checkpoint rule, update the existing
      durable lesson, pass focused and mandatory verification, and perform no
      hardware attempt or parity promotion.

Software-only authorization: inspect and modify repository source, tests,
tracker, lessons, and repo-local guidance; run deterministic fixtures and local
child processes; and build firmware. Do not read protected inputs or access
USB, device/network interfaces, HTTP, mining, hardware controls, direct UART,
pins/pads/GPIO, parity evidence, or a new hardware-attempt ordinal.

Completion review: operator-gated processes and the safe IDENTIFY ready and
cleared checkpoints now have no elapsed-time deadline. Automated notification,
recovery, cleanup, transport, and finite physical-effect observation remain
explicitly bounded; the IDENTIFY rendered proof retains its exact 30-second
window. Real-child-process, TypeScript, Rust, Bazel, Bright Builds, full test,
parity, redaction, reference-cleanliness, build, formatting, lint, and diff
checks passed. No protected input, hardware, network/device HTTP, USB attempt,
parity evidence, or status promotion was used. Residual risk: an unbounded wait
still depends on the host process and machine remaining available; persisted
resume state across host shutdown is separate future work.

Operator-gated attempt-020 plan:
`docs/parity/work-plans/20260814T185221Z-API-009/PLAN.md`.

- [x] Commit and push the immutable single-attempt contract and pass every
      focused lifetime, mandatory, privacy, reference, firmware, selector, and
      exact-package gate.
- [x] Run exactly one fresh detector-gated attempt-020, keep the device paused
      and safe-stopped through unbounded human waits, and start the exact
      30-second IDENTIFY window only from the local ready signal.
- [x] Promote only on the complete command/restart/device-user quorum;
      otherwise preserve `implemented`, earliest failure, evidence withholding,
      safe stop, cleanup, recovery, and stop without attempt-021.

Attempt-020 authorization: pushed source `a4a48db7` removes attempt-019's
guessed human-response, fixture, and parent-process deadlines while retaining
exact automated safety, recovery, cleanup, and physical-effect bounds. Focused
plus mandatory gates must pass before this plan is pushed. Standing task
authorization permits this one campaign only after the linked immutable
contract and all named gates pass at clean pushed HEAD. Campaign start consumes
attempt-020. No inferred or expired physical observation, protected-artifact
reuse, external or owner pool, factory reset, destructive/fault-injection
action, direct UART, pin/pad/GPIO manipulation, attempt-021, or unchanged retry
is authorized.

Completion review: attempt-020 admitted exact package `6aec8582` and one Ultra
205; both flashes completed once as `ready`, runtime attestation was `trusted`,
and the campaign proved notification, positive block count, pause, paused safe
stop, both IDENTIFY requests, and live rendered plus cleared observations. The
unbounded human waits behaved correctly. After cleared, the one resume request
was issued but active recovery was not confirmed before the earliest
`safety_stale` terminal marker; every required terminal observation freshness
flag was false. Dismiss and restart did not run. Recovery pause was attempted
but secondary recovery failed, terminal safe stop is unconfirmed, USB cleanup
is `ready`, private modes and process cleanup pass, and public evidence is
withheld. API-009 remains `implemented`; attempt-020 is consumed and no
attempt-021 is authorized. See the linked `CLOSURE.md`.

Post-attempt-020 resume-freshness investigation:

- [x] Build and run a fast red regression for the production host seam where a
      recoverable stale resume sample precedes a fresh observation wakeup.
- [x] Rank and falsify paused acquisition, readiness publication, and host
      terminal-policy hypotheses using the smallest deterministic sequence.
- [x] Implement the smallest confirmed root-cause fix, preserve fail-closed
      active-mining safety, and pass focused plus mandatory verification.

Software-only authorization: use attempt-020 categorical, boolean, count, and
freshness facts; inspect and modify repository source and tests; run
deterministic fixtures and local child processes; and build firmware. Do not
read credentials, protected sensor values, origins, ports, USB/network
identities, or raw traces; access USB, device/network interfaces, or HTTP;
flash, reset, restart, mine, actuate controls, manipulate direct UART or
pins/pads/GPIO; publish parity evidence; promote API-009; or create/run
attempt-021.

Completion review: the production-shaped regression failed deterministically
as `safety_stale` when a stopped, armed resume marker with stale readiness was
followed by a fresh observation wakeup and a valid terminal marker. That
falsified a permanently stopped acquisition or firmware recovery failure and
confirmed that the host latched the transient marker too broadly. The host now
allows recovery only for that exact non-actuating command-effects resume state;
active stale telemetry and stale observation-stage telemetry remain terminal.
Focused command-effects, negative-control, observation-stage, full flash-tool,
mandatory Cargo, Bright Builds, Bazel, parity, privacy, reference, firmware,
and diff gates pass. No protected input or hardware was accessed, no evidence
was published, API-009 remains `implemented`, and attempt-021 remains out of
scope.

Resume-repaired attempt-021 plan:
`docs/parity/work-plans/20260814T194002Z-API-009/PLAN.md`.

- [x] Commit and push the immutable single-attempt contract and pass every
      focused resume-freshness, mandatory, privacy, reference, firmware,
      selector, and exact-package gate.
- [x] Run exactly one fresh detector-gated attempt-021, keep the device paused
      and safe-stopped through unbounded human waits, and start the exact
      30-second IDENTIFY window only from the local ready signal.
- [x] Promote only on the complete command/restart/device-user quorum;
      otherwise preserve `implemented`, earliest failure, evidence withholding,
      safe stop, cleanup, recovery, and stop without attempt-022.

Attempt-021 authorization: pushed source `5df560f0` fixes the exact
attempt-020 resume-freshness terminal-policy defect while retaining terminal
active-mining and observation-stage stale telemetry. Focused plus mandatory
gates must pass before this plan is pushed. Standing task authorization permits
this one campaign only after the linked immutable contract and all named gates
pass at clean pushed HEAD. Campaign start consumes attempt-021. No inferred or
expired physical observation, protected-artifact reuse, external or owner pool,
factory reset, destructive/fault-injection action, direct UART, pin/pad/GPIO
manipulation, attempt-022, or unchanged retry is authorized.

Completion review: attempt-021 admitted exact package `edd05b1d` and one Ultra
205, then reached the safe unbounded ready checkpoint. The ready signal started
the one exact 30-second IDENTIFY window, but the operator reported not watching
the display and could not confirm the rendered frame. The supported `declined`
outcome was recorded without inference or replay. The campaign closed as
`hardware_blocked` with terminal safe stop confirmed, cleanup complete,
successful recovery, no secondary recovery failure, valid private modes,
attempt processes absent, and public evidence withheld. API-009 remains
`implemented`; attempt-021 is consumed and no attempt-022 is authorized. See
the linked `CLOSURE.md`.

Replayable IDENTIFY checkpoint software plan:
`docs/parity/work-plans/20260814T200547Z-API-009/PLAN.md`.

- [x] Commit and push the immutable software-only plan before implementation.
- [x] Add a deterministic red regression for a missed IDENTIFY window followed
      by one explicit replay, then implement the smallest safe typed protocol.
- [x] Prove one bounded replay, unbounded operator wait, exact per-effect
      duration, no early/duplicate request, late-confirmation rejection,
      decline/cleanup, redaction, and existing behavior compatibility.
- [x] Pass focused, real-process, mandatory, privacy, reference, firmware, and
      diff gates; keep API-009 `implemented` and close without hardware or
      attempt-022.

Software-only authorization: inspect and modify repository source, tests,
tracker, worklog, closure, and deterministic child-process fixtures, and build
firmware. Do not access credentials, protected attempt artifacts, USB,
device/network interfaces, HTTP, display, mining, hardware controls, direct
UART, pins/pads/GPIO, parity evidence, checklist promotion, attempt-022, or any
other hardware attempt.

Replayable IDENTIFY plan completion review: the host transaction now accepts
one explicit replay after a missed rendered window, queues it until the prior
30-second effect is conservatively inactive, opens one new exact confirmation
window, rejects late or duplicate confirmation/replay, and binds the distinct
checkpoint sequence to closed replay/request counts. Operator waits remain
unbounded, while each physical effect and the replay count remain bounded.
Focused loopback HTTP, parser/state-machine, CLI, evidence mismatch, supervisor
real-process, all mandatory, privacy, reference, firmware, selector, digest,
and diff gates pass. No credential, protected attempt, detector, USB, device,
network, display, mining, hardware-control, direct UART, or pin interface was
accessed. API-009 remains `implemented`; no attempt-022 or hardware evidence
was created. See the linked `CLOSURE.md`.

Replay-capable attempt-022 plan:
`docs/parity/work-plans/20260814T224914Z-API-009/PLAN.md`.

- [x] Commit and push the immutable single-attempt contract, then pass every
      focused replay, mandatory, privacy, reference, firmware, selector, and
      exact-package gate at clean synchronized HEAD.
- [x] Run exactly one fresh detector-gated attempt-022, keep the Ultra 205
      paused and safe-stopped through every unbounded operator wait, and use at
      most one explicit IDENTIFY replay only if the first window is missed.
- [x] Promote API-009 only on the complete command/restart/device-user quorum;
      otherwise preserve `implemented`, the earliest typed failure, evidence
      withholding, safe stop, cleanup, and recovery, then stop without
      attempt-023.

Attempt-022 authorization: pushed source `420c2236` adds the typed, bounded
single-replay protocol after attempt-021's missed observation. Standing task
authorization permits this one exact-package campaign only after the linked
immutable contract and all named gates pass. Campaign start consumes
attempt-022. No inferred observation, protected-artifact reuse, external or
owner pool, factory reset, erase, destructive/fault-injection action, direct
UART, pin/pad/GPIO manipulation, attempt-023, or unchanged retry is authorized.

Attempt-022 completion review: exact package `79756b9a` and one detector-admitted
Ultra 205 passed through trusted identity, genuine notification, positive block
count, pause, stopped hardware, and the live rendered observation. The rendered
response reached the host 34 seconds after its checkpoint opened, beyond the
exact 30-second evidence window, so the campaign correctly closed as
`operator_checkpoint_expired` / `operator_paused`. One recovery pause confirms
terminal safe stop; USB cleanup, recovery, private modes, and process cleanup
pass; public evidence is withheld. API-009 remains `implemented`, attempt-022
is consumed, and no attempt-023 is authorized. The next safe action is a fresh
software-only post-expiry replay-choice plan; see the linked `CLOSURE.md`.

Latency-tolerant interaction plan:
`docs/parity/work-plans/20260814T232306Z-API-009/PLAN.md`.

- [x] Commit and push the immutable software-only plan without accessing
      credentials, protected attempts, USB, device/network interfaces, or
      hardware.
- [x] Make rendered/replayed reports unbounded and attempt-bound while keeping
      every IDENTIFY effect exactly 30 seconds and every replay explicit.
- [x] Use conservative natural expiry before the cleared checkpoint instead of
      a latency-sensitive second IDENTIFY toggle; update the closed request
      count and evidence quorum.
- [x] Pass focused, real-process, mandatory, privacy, reference, firmware,
      selector, digest, and diff gates; close with API-009 still `implemented`
      and no attempt-023.

Software-only authorization: modify source, tests, documentation, tracker,
worklog, closure, and deterministic fixtures, and build firmware. Do not access
credentials, protected attempt artifacts, detector, USB, device/network
interfaces, HTTP, display, mining, hardware controls, direct UART, pins/pads,
public parity evidence, checklist promotion, attempt-023, or any hardware
attempt.

Latency-tolerant interaction completion review: rendered and replayed responses
are now unbounded, attempt-local attestations that the exact frame was observed
during the uniquely bound 30-second device effect. The campaign performs one
IDENTIFY request per effect, waits for conservative natural expiry before the
cleared checkpoint, and requires `1 + replay_count` requests in closed evidence.
Focused Rust and real-child regressions plus every mandatory, privacy,
reference, firmware, selector, digest, and diff gate pass. API-009 remains
`implemented`; this software-only plan created no hardware evidence and did not
authorize attempt-023. See the linked `CLOSURE.md`.

Latency-tolerant attempt-023 plan:
`docs/parity/work-plans/20260814T234917Z-API-009/PLAN.md`.

- [x] Commit and push the immutable single-attempt contract, then pass every
      focused, mandatory, privacy, reference, firmware, selector, and
      exact-package gate at clean synchronized HEAD.
- [x] Run exactly one fresh detector-gated attempt-023, keep the Ultra 205
      paused and safe-stopped through every unbounded operator wait, and accept
      a delayed rendered/replayed report only as an attestation that the exact
      frame was observed during its uniquely bound 30-second effect.
- [x] Promote API-009 only on the complete command/restart/device-user quorum;
      otherwise preserve `implemented`, the earliest typed failure, evidence
      withholding, safe stop, cleanup, and recovery, then stop without
      attempt-024.

Attempt-023 authorization: pushed source `168e599e` implements the unbounded,
attempt-local observed-during-effect attestation and natural-expiry clear
transaction after attempt-022's chat-latency failure. Standing task
authorization permits this one exact-package campaign only after the linked
immutable contract and all named gates pass. Campaign start consumes
attempt-023. No inferred observation, protected-artifact reuse, external or
owner pool, factory reset, erase, destructive/fault-injection action, direct
UART, pin/pad/GPIO manipulation, attempt-024, or unchanged retry is authorized.

Attempt-023 completion review: the exact pushed package and one admitted Ultra
205 proved notification, positive block count, pause plus stopped hardware,
one IDENTIFY request, the live exact rendered observation, natural expiry, and
the live cleared observation. The subsequent resume request was issued, but
active mining did not return before the fixed 15-second automated phase
deadline, so the earliest typed result is `network_correlation_failed`; no
dismissal or restart ran. USB cleanup is ready, private modes and result seal
pass, processes are absent, recovery reported a secondary failure, safe stop
was not confirmed, and public evidence is withheld. API-009 remains
`implemented`; attempt-023 is consumed and no attempt-024 is authorized. See
the linked `CLOSURE.md`.

Resume-correlation software plan:
`docs/parity/work-plans/20260815T010813Z-API-009/PLAN.md`.

- [x] Commit and push the immutable software-only plan without accessing
      credentials, protected attempts, detector, USB, device/network, display,
      mining, or hardware-control interfaces.
- [x] Split resume intent, bounded reactivation, and active recovery into typed
      host states; retain the 600-second active-mining budget while excluding
      unbounded operator-paused time from that budget.
- [x] Replace immediate failure breakage with a bounded recovery-pause join
      that preserves the primary failure and independently proves or rejects
      terminal safe stop before network-worker shutdown.
- [x] Add focused engine, firmware-owner, host-state, evidence, recovery, CLI,
      and real-process regressions and pass every mandatory, privacy,
      reference, firmware, selector, digest, and diff gate. Close with API-009
      still `implemented` and no attempt-024.

Software-only authorization: modify source, tests, deterministic fixtures,
documentation, tracker, worklog, and closure, and build firmware. Do not access
credentials, current or prior protected attempt artifacts, detector, USB,
device/network, HTTP to a device, display, mining, hardware controls, direct
UART, pins/pads/GPIO, public parity evidence, checklist promotion, attempt-024,
or any hardware attempt.

Resume-correlation completion review: resumable campaigns now charge only
active mining segments against the 600-second lease. The host distinguishes
resume intent from bounded production reactivation and, on any later failure,
preserves the primary category while joining one recovery pause across fresh
API-paused and authoritative serial stopped-hardware observations. Closed
command evidence v6 records those facts without runtime identifiers or traces.
Focused engine, firmware-owner, host, recovery, CLI, automation, and real-child
regressions pass with the complete ordered Cargo, Bright Builds, all 44 Bazel
tests, parity, progress, redaction, reference, and firmware-build gates. The
immutable plan digest and diff checks pass. API-009 remains `implemented`; no
hardware interface was accessed and attempt-024 remains unauthorized by this
closed plan. A fresh immutable exact-package hardware plan is the next safe
action. See the linked `CLOSURE.md`.

Resume-correlation attempt-024 plan:
`docs/parity/work-plans/20260815T015842Z-API-009/PLAN.md`.

- [x] Commit and push the immutable single-attempt contract, then pass every
      focused, mandatory, privacy, reference, firmware, selector, and
      exact-package gate at clean synchronized HEAD.
- [x] Run exactly one fresh detector-gated attempt-024, keep the Ultra 205
      paused and safe-stopped through every unbounded operator wait, and use
      the split resume-intent/reactivation and correlated recovery protocol.
- [x] Promote API-009 only on the complete command/restart/device-user quorum;
      otherwise preserve `implemented`, the earliest typed failure, evidence
      withholding, safe stop, cleanup, and recovery, then stop without
      attempt-025.

Attempt-024 authorization: pushed source `89d05d35` repairs attempt-023's exact
resume and recovery boundary. Standing task authorization permits this one
exact-package campaign only after the linked immutable contract and all named
gates pass. Campaign start consumes attempt-024. No inferred observation,
protected-artifact reuse, external or owner pool, factory reset, erase,
destructive/fault-injection action, direct UART, pin/pad/GPIO manipulation,
attempt-025, or unchanged retry is authorized.

Attempt-024 completion review: exact pushed package `a5e0a447` and one admitted
Ultra 205 passed trusted identity, genuine notification, positive block count,
pause plus stopped hardware, one IDENTIFY request, live rendered observation,
natural expiry, live cleared observation, and one resume-intent confirmation.
The next live transition exposed a distinct firmware boundary: while the
resumable campaign was armed and hardware-ready during primary-pool
reconnection, an unchanged stale safety observation was treated as terminal
`safety_stale` instead of a resumable reactivation safe stop. Active recovery,
dismissal, and restart did not run. Terminal hardware safe stop, USB cleanup,
private modes, fixture cleanup, and result sealing pass; the public projection
is withheld. API-009 remains `implemented`, attempt-024 is consumed, and no
attempt-025 is authorized. See the linked `CLOSURE.md`.

Reactivation-safety software plan:
`docs/parity/work-plans/20260815T022250Z-API-009/PLAN.md`.

- [x] Commit and push the immutable software-only plan without accessing
      credentials, protected attempts, detector, USB, device/network, display,
      mining, or hardware-control interfaces.
- [x] Preserve a resumable lease and active-time budget when safety freshness
      lapses after hardware preparation but before active reactivation.
- [x] Keep initial-activation and already-active safety lapses terminal; add the
      exact live-shaped transition and negative regressions.
- [x] Pass every focused, mandatory, privacy, reference, firmware, selector,
      digest, and diff gate. Close with API-009 still `implemented` and no
      attempt-025.

Software-only authorization: modify source, tests, deterministic fixtures,
documentation, tracker, worklog, and closure, and build firmware. Do not access
credentials, current or prior protected attempt artifacts, detector, USB,
device/network, HTTP to a device, display, mining, hardware controls, direct
UART, pins/pads/GPIO, public parity evidence, checklist promotion, attempt-025,
or any hardware attempt.

Completion review: The pure production session now recognizes only the
post-pause, prior-active, currently-preactive stale-safety gap as resumable. It
safe-stops prepared hardware, retains the same lease and accumulated active
budget, and can later reprepare and reactivate from fresh observations. Initial
activation and already-active safety staleness remain terminal. The exact
attempt-024-shaped regression, negative controls, focused engine and firmware
owner tests, ordered Rust gates, Bright Builds, all 44 Bazel tests, parity and
progress, redaction, reference, firmware build, selector, plan digest, task,
sensitive-output, and diff checks pass. One combined `just parity` invocation
encountered a transient host `os error 35`; the exact isolated retry passed.
API-009 remains `implemented`; no hardware interface was accessed, and
attempt-025 remains unauthorized by this closed software-only plan. See the
linked `CLOSURE.md`.

Reactivation-safety attempt-025 plan:
`docs/parity/work-plans/20260815T024341Z-API-009/PLAN.md`.

- [x] Commit and push the immutable single-attempt contract, then pass every
      focused, mandatory, privacy, reference, firmware, selector, and
      exact-package gate at clean synchronized HEAD.
- [x] Run exactly one fresh detector-gated attempt-025, keep the Ultra 205
      paused and safe-stopped through every unbounded operator wait, and use
      the repaired resumable pre-active safety cycle during reactivation.
- [x] Promote API-009 only on the complete command/restart/device-user quorum;
      otherwise preserve `implemented`, the earliest typed failure, evidence
      withholding, safe stop, cleanup, and recovery, then stop without
      attempt-026.

Hardware authorization: exactly the command, effects, evidence, privacy,
recovery, retry, and stop contract in the immutable linked plan. No erase,
factory reset, OTA, rollback, power cycle, external pool, direct UART,
pin/pad/GPIO manipulation, probe, jumper, soldering, injected signal, fault
injection, arbitrary hardware override, non-205 device, or second campaign run
is authorized.

Attempt-025 completion review: exact pushed source and package `bcb46964`, one
fresh detector, and the sole campaign passed trusted identity, notification,
positive block count, pause with stopped hardware, the complete live IDENTIFY
render/expiry transaction, resume intent, recovery from stale to five fresh
safety observations, active reactivation, and an accepted share. This proves
the attempt-024 firmware repair on hardware. The sealed result then closed as
`network_correlation_failed`: the one post-reactivation dismissal request was
issued, but notification clearing, block-count preservation, and the dependent
terminal HTTP check were not confirmed before terminal safe stop. Generic
continuity-window counters are non-applicable to this command-specific
campaign and did not cause the failure. Terminal safe stop, USB cleanup,
protected modes, fixture cleanup, seal, and process cleanup pass. Public
evidence is withheld, API-009 remains `implemented`, attempt-025 is consumed,
and no attempt-026 is authorized. See the linked `CLOSURE.md`.

Post-reactivation dismissal software plan:
`docs/parity/work-plans/20260815T034210Z-API-009/PLAN.md`.

- [x] Commit and push the immutable software-only plan without accessing
      credentials, detector, USB, device/network, display, mining, or hardware-
      control interfaces.
- [x] Reproduce the post-reactivation dismissal race at the command-effects
      state-machine seam with a live-shaped red regression.
- [x] Move the one dismissal and count-preserving clear join into the proven
      safe-stopped pause before IDENTIFY, then resume directly into terminal
      validation after active recovery.
- [x] Clarify and test that generic soak continuity windows are non-applicable
      to the command-effects quorum; run every mandatory gate and close without
      API-009 promotion or attempt-026.

Software-only authorization: modify source, tests, deterministic loopback
fixtures, documentation, tracker, worklog, and closure, and build firmware. Do
not access credentials, protected raw traces, detector, USB, device/network,
device HTTP, display, mining, hardware controls, direct UART, pins/pads/GPIO,
public parity evidence, checklist promotion, attempt-026, or any physical
effect.

Software-plan completion review: pushed source `a90a0405` moves the single
dismissal and its exact clear/count-preservation readback into the joined
safe-stopped pause, before IDENTIFY. The live-shaped regression proves active
reactivation now advances directly to terminal validation without a second
dismissal, and the command-specific evidence constructor explicitly marks
generic soak windows non-applicable. Ordered Cargo, Bright Builds, all 44
Bazel tests, parity/progress, redaction, reference, firmware, immutable-plan,
unique-task, selector, sensitive-output, and diff gates pass. API-009 remains
`implemented`; the residual risk is live device timing, so a fresh immutable
attempt-026 hardware plan is the next safe action.

Paused-dismissal hardware attempt-026 plan:
`docs/parity/work-plans/20260815T041103Z-API-009/PLAN.md`.

- [x] Commit and push the immutable single-attempt contract, then pass every
      focused, mandatory, privacy, reference, firmware, selector, and exact-
      package gate at clean synchronized HEAD.
- [x] Run exactly one fresh detector-gated attempt-026, keeping the Ultra 205
      paused and safe-stopped through every unbounded operator wait and
      requiring the dismissal clear/count join before IDENTIFY readiness.
- [x] Promote API-009 only on the complete command/restart/device-user quorum;
      otherwise preserve `implemented`, the earliest typed failure, evidence
      withholding, safe stop, cleanup, and recovery, then stop without
      attempt-027.

Hardware authorization: exactly the command, effects, evidence, privacy,
recovery, retry, and stop contract in the immutable linked plan. No erase,
factory reset, OTA, rollback, power cycle, external pool, direct UART,
pin/pad/GPIO manipulation, probe, jumper, soldering, injected signal, fault
injection, arbitrary hardware override, non-205 device, or second campaign run
is authorized.

Attempt-026 completion review: exact pushed source and package `5621ea0f`, one
fresh detector, and the sole campaign passed trusted identity, a genuine
positive notification, one pause with stopped hardware, one dismissal request,
and notification-clear confirmation. The count-preservation join then failed
before any operator checkpoint opened because the host still compared against
the first active-notification count instead of the current paused count at the
dismissal boundary. Recovery confirmed both API pause and serial safe stop;
USB, fixture, process, private-mode, and seal cleanup pass. Public evidence is
withheld, API-009 remains `implemented`, attempt-026 is consumed, and no
attempt-027 is authorized. See the linked `CLOSURE.md`.

Paused-count baseline software plan:
`docs/parity/work-plans/20260815T042822Z-API-009/PLAN.md`.

- [x] Commit and push the immutable software-only plan.
- [x] Reproduce the stale initial-notification count comparison with a red
      state-machine regression.
- [x] Bind preservation to the positive paused count sampled immediately
      before the sole dismissal request; run all mandatory gates and close
      without API-009 promotion or attempt-027.

Software-only authorization: source, tests, loopback fixtures, documentation,
tracker, worklog, closure, and firmware build only. Credentials, protected
traces, detector, USB, device/network, display, mining, controls, public
evidence, checklist promotion, and attempt-027 are prohibited.

Paused-count software-plan completion review: pushed source `697688f0` binds
the preservation baseline to the positive count sampled after HTTP pause and
serial safe stop have joined, immediately before the sole dismissal request.
The live-shaped regression failed against the prior source when count advanced
during pause convergence and passes after the fix; a separate test proves zero
cannot reach dismissal. Focused tests, all 292 flash tests, ordered Cargo,
Bright Builds, all 44 Bazel tests, parity/progress, redaction, reference,
firmware, immutable-plan, unique-task, selector, sensitive-output, and diff
gates pass. API-009 remains `implemented`; the residual risk is live timing,
so a separately planned attempt-027 is the next safe action.

Paused-count hardware attempt-027 plan:
`docs/parity/work-plans/20260815T044046Z-API-009/PLAN.md`.

- [x] Commit and push the immutable single-attempt contract, then pass every
      focused, mandatory, privacy, reference, firmware, selector, and exact-
      package gate at clean synchronized HEAD.
- [x] Run exactly one fresh detector-gated attempt-027, keeping the Ultra 205
      paused and safe-stopped through every unbounded operator wait and
      requiring the immediate paused-count dismissal join before IDENTIFY.
- [x] Promote API-009 only on the complete command/restart/device-user quorum;
      otherwise preserve `implemented`, the earliest typed failure, evidence
      withholding, safe stop, cleanup, and recovery, then stop without
      attempt-028.

Hardware authorization: exactly the command, effects, evidence, privacy,
recovery, retry, and stop contract in the immutable linked plan. No erase,
factory reset, OTA, rollback, power cycle, external pool, direct UART,
pin/pad/GPIO manipulation, probe, jumper, soldering, injected signal, fault
injection, arbitrary hardware override, non-205 device, or second campaign run
is authorized.

Attempt-027 closure review: the sole campaign reached its live IDENTIFY-ready
checkpoint and remained paused and safe-stopped during the unbounded operator
wait. The programmatic-platform redesign intentionally declined that checkpoint
through the repo-owned typed signal. The response was consumed, the complete
campaign process group and USB holder exited, and the public projection remains
withheld. The physical UAT is deferred without attributing failure to an
operator action. API-009 remains `implemented`; attempt-027 is consumed and no
attempt-028 is authorized by this closed plan.

### task-parity-ui004-live-browser-attempt-001 | 2026-08-13 | Verify exact-package AxeOS browser workflows

- [x] Add a typed private-first evidence join and independent validator for the
      exact current package, embedded static assets, one live real-browser
      session, and existing theme/settings/log/update projections.
- [ ] Prove all seven production routes, desktop/mobile responsive behavior,
      mobile navigation, same-origin API/log traffic, blank write-only secrets,
      disabled no-file firmware action, OTAWWW-unavailable behavior, clean
      console/network state, safe device state, cleanup, and redaction.
- [x] Pass focused, real-firmware/package, mandatory, privacy, reference,
      generated-contract, immutable-plan, task-uniqueness, and diff gates.
- [x] Commit and push the implementation, then spend exactly one detector and
      at most one conditional exact-package hardware/browser attempt-001.
- [ ] Promote only UI-004 when the independently validated closed projection
      satisfies every plan criterion; otherwise withhold evidence and stop.

Plan: `docs/parity/work-plans/20260813T045300Z-UI-004/PLAN.md`.

Hardware contract: The only effectful repo-owned command is the exact
`capture-operator-snapshot-evidence` invocation frozen in the immutable plan,
after one successful `just detect-ultra205`. It may flash the current board-205
package with owner-supplied Wi-Fi credentials and `mineonboot=false`, capture
receive-only USB and same-origin state, perform one normal software restart,
and use one exact-package recovery flash only if final package/safe-state
restoration cannot otherwise be confirmed. The subsequent isolated browser
session is read-only and may only navigate, resize, open/close mobile
navigation, and observe normal API/log traffic.

Evidence and privacy: Protected artifacts belong only beneath mode-`0700`,
gitignored `scratch/ui004-live-workflows` and
`output/playwright/ui004-attempt-001` roots with mode-`0600` files. The sole
public output is
`docs/parity/evidence/ui004-live-workflows/ui-workflow-projection.json` and may
contain only closed facts and digests. Origins, hostnames, addresses, ports,
USB/network/process identities, credentials, page values, HTTP/WebSocket
bodies, commands, traces, screenshots, and private paths must not enter Git or
terminal output.

Retry, recovery, and stop limits: Starting the capture consumes attempt-001;
do not retry it. Preserve the earliest typed failure through cleanup and
optional exact-package recovery. Accepted terminal categories are
`package_invalid`, `process_failed`, `timeout`, `hardware_blocked`,
`browser_blocked`, `evidence_invalid`, and `recovery_failed`. Any non-success,
unexpected device/browser mutation, unclean resource, mode failure, sensitive
output, or failed gate withholds evidence and leaves UI-004 implemented.

Prohibited effects: settings or theme submission, pause/resume/restart/identify
from the browser, block dismissal, firmware upload, OTA/OTAWWW, recovery-page
upload, mining, ASIC work, pool access, voltage/frequency/fan/thermal/power
control, camera or panel claim, physical button, direct UART, pins, pads,
headers, GPIO, probes, arbitrary writes, erase-flash, or other boards.

Blocked outcome (2026-08-13): attempt-001 successfully completed the exact
package operator snapshot and private read-only browser quorum, but the sole
projector invocation correctly rejected two shell-redirected wrapper files
created as mode `0644`. No public projection or candidate exists. All device,
browser, USB, and private-mode cleanup passed. See the plan-bound `CLOSURE.md`.

Next safe action: Start a fresh software-only continuation with an owner-only
projector redirection contract and exact captured-source/projector-source
binding. Preserve the ignored attempt-001 artifacts and do not rerun its
hardware or browser work.

Completion review: UI-004 remains `implemented`; verification is not claimed.
The user's report of more visible panel information remains helpful context,
not typed UI-001/UI-002 evidence.

### task-api009-programmatic-pilot-attempt-028 | 2026-08-15 | Prove autonomous command effects on Ultra 205

- [x] At clean synchronized pushed HEAD, build and validate the exact package,
      require non-empty ignored `wifi-credentials.json` without reading it,
      and require fresh detector, attempt, and public-projection paths.
- [x] Create mode-`0700` `scratch/api009-command-effects/detector-028` with a
      mode-`0600` `detector.stdout`, then run exactly one
      `just detect-ultra205` and admit only one holder-free ESP32-S3 board 205.
- [x] Run exactly once:
      `just api-command-effects-campaign --private-root scratch/api009-command-effects/attempt-028 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/api009-command-effects/detector-028/detector.stdout --projection docs/parity/evidence/api009-command-effects/command-effects-projection.json --duration-seconds 600`.
- [x] Validate a sealed, redacted programmatic projection proving the pause,
      dismissal, IDENTIFY render/clear receipts, resume, same-device restart,
      exact package, safe stop, recovery, and cleanup quorum; otherwise record
      the earliest typed failure and withhold evidence.

Objective and allowed effects: prove the no-human API-009 programmatic campaign
on the connected Ultra 205. This single attempt may perform one exact-package
USB flash/reset, use the private Wi-Fi seed and generated local-fixture NVS
input, initialize and mine with the conservative profile for at most 600
accumulated active seconds, issue exactly one pause, notification dismissal,
IDENTIFY enable, resume, and software restart, observe HTTP/WebSocket and
receive-only native USB, and perform terminal safe stop, same-device recovery,
and process/USB cleanup.

Evidence and privacy: the attempt and detector roots are ignored private
mode-`0700` directories and their files are mode `0600`. The sole public output
is the redacted projection named above, written only after a ready closed
quorum. Origins, hostnames, settings values, addresses, ports, USB/network or
process identities, credentials, fixture inputs, frame text, sensor values,
and raw traces stay private and must not enter console summaries or committed
evidence.

Recovery, retry, and stops: the repo-owned campaign must safe-stop mining,
restore the admitted package/session state, terminate child processes, release
USB holders, and preserve the earliest primary failure before returning. The
campaign has finite automated effect and recovery deadlines. Attempt-028 is
consumed when the campaign command starts; no same-contract retry or attempt-029
is authorized. `ready` permits independent validation and later planning of
the physical-display UAT. Any typed non-ready category, malformed/missing
projection, identity ambiguity, build mismatch, cleanup/recovery failure,
unexpected holder, detector failure, or unavailable device stops this task
with API-009 still `implemented` and evidence withheld. No OTA, erase, factory
reset, power cycle, external UART/BAP, USB request/response protocol, direct
pin/pad/GPIO work, arbitrary settings writes, external pool, stress profile,
voltage/frequency/fan/thermal override, fault injection, non-205 device, or
human display claim is authorized.

Attempt-028 closure review: exact package/runtime identity, safety, the local
fixture, genuine notification, accepted work/share activity, one pause request,
HTTP `operator_paused`, stopped hardware, and the receive-only serial safe-stop
marker all passed. The primary `network_correlation_failed` occurred because
the new host flow incorrectly required both USB and WebSocket transition-marker
generations before crediting the otherwise complete pause claim. Recovery's
secondary pause request could not reach HTTP; cleanup still completed and the
public projection was withheld. Attempt-028 is consumed, no attempt-029 is
authorized by this block, and API-009 remains `implemented`.

Regression-backed follow-up: a deterministic production-seam test now
reproduces the exact HTTP-generation + paused-state + serial-safe-stop boundary
without optional log-marker quorum. It failed twice before the fix and passes
after restoring claim-specific proof: pause/resume use HTTP generation, system
state, and runtime witness; dismiss uses generation plus notification state and
count; IDENTIFY uses HTTP generation/flush receipts plus a retained marker from
at least one independent log channel. USB and WebSocket marker completeness is
still recorded but no longer forms a fragile generic gate. Focused flash and
automation suites and every full repository gate pass; publication remains
pending.

### task-api009-programmatic-pilot-attempt-029 | 2026-08-15 | Re-run the corrected claim-specific command proof

- [x] Require clean synchronized pushed source containing commit `26cf68f7`,
      rebuild/validate its exact package, confirm non-empty ignored Wi-Fi input
      without reading it, and require fresh detector/attempt/projection paths.
- [x] Create protected `scratch/api009-command-effects/detector-029` and run
      exactly one `just detect-ultra205`; admit only one holder-free ESP32-S3
      board 205 and preserve its raw output as mode `0600` private evidence.
- [x] Invoke exactly once:
      `just api-command-effects-campaign --private-root scratch/api009-command-effects/attempt-029 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/api009-command-effects/detector-029/detector.stdout --projection docs/parity/evidence/api009-command-effects/command-effects-projection.json --duration-seconds 600`.
- [x] Independently validate and redact the sealed projection only on the full
      claim-specific command, same-device restart, package, recovery, safe-stop,
      and cleanup quorum; otherwise withhold it and record the earliest failure.

Objective and effects: test only the pushed claim-specific quorum correction
that follows consumed attempt-028. The sole run may flash/reset the exact
package, seed private Wi-Fi and the generated local fixture, initialize and
mine the conservative profile for at most 600 active seconds, issue one each
pause/dismiss/IDENTIFY/resume/software-restart request, observe HTTP plus
WebSocket and receive-only USB, and perform same-device recovery, terminal safe
stop, process termination, and USB cleanup. Automated effects and recovery have
finite repo-owned bounds; no human checkpoint is part of this attempt.

Evidence/privacy/recovery: the detector and attempt directories are ignored
mode-`0700` roots with mode-`0600` files. Only the named aggregate projection
may become public after ready validation and redaction. Origins, ports,
hostnames, addresses, USB/network/process identity, credentials, fixture data,
frame text, sensors, and raw traces remain private. The earliest primary failure
must survive recovery; safe stop, child termination, and holder cleanup run on
every exit. Campaign start consumes attempt-029; no attempt-030 or same-contract
retry is authorized. Any detector/identity/build ambiguity, non-ready category,
missing or malformed projection, failed safe stop/recovery/cleanup, or absent
device stops with API-009 `implemented` and evidence withheld. OTA, erase,
factory reset, power cycle, external UART/BAP, USB duplex, pins/pads/GPIO,
arbitrary settings, external pool, stress, controls, fault injection, non-205
hardware, and human display claims remain prohibited.

Attempt-029 closure review: exact package/runtime identity, the local fixture,
genuine notification, accepted activity, one pause request, HTTP
`operator_paused`, stopped hardware, the serial safe-stop marker, recovery safe
stop, and USB cleanup passed. The earliest primary category was
`safety_stale`; a transient stale-sensor marker was incorrectly fatal after the
pause had already stopped the device. The public projection is withheld,
attempt-029 is consumed, no attempt-030 is authorized by this block, and
API-009 remains `implemented`. No user action contributed to the failure.

Regression-backed follow-up: the production-shaped stopped-pause marker failed
deterministically before the fix and passes afterward. The HTTP seam now also
has paired guards proving identity-only admission while stopped and continued
fresh-safety enforcement while active. Both the marker and HTTP transaction
paths therefore treat a confirmed stopped pause by its safe-stop witness while
retaining strict safety admission for notification, active resume, and active
terminal phases. Publication and any new hardware ordinal remain pending full
repository verification. Focused Cargo/Bazel regressions, the ordered full
Cargo gates, Bright Builds, firmware build, all 44 Bazel tests, parity and
progress, redaction, pinned-reference cleanliness, and diff checks pass; the
fix is ready to publish.

### task-api009-programmatic-pilot-attempt-030 | 2026-08-15 | Prove command effects with stopped-state safety admission

- [x] Require clean synchronized pushed source containing commit `b2a8a066`,
      rebuild and validate its exact package, confirm non-empty ignored Wi-Fi
      input without reading it, and require fresh detector, attempt, and public
      projection paths.
- [x] Confirm the detector step did not execute after preflight stopped before
      creating `scratch/api009-command-effects/detector-030`; no board, port,
      USB holder, or hardware effect was admitted under this attempt.
- [x] Invoke exactly once:
      `just api-command-effects-campaign --private-root scratch/api009-command-effects/attempt-030 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/api009-command-effects/detector-030/detector.stdout --projection docs/parity/evidence/api009-command-effects/command-effects-projection.json --duration-seconds 600`.
- [x] Independently validate and redact the sealed projection only on the full
      claim-specific command, same-device restart, exact-package, recovery,
      safe-stop, and cleanup quorum; otherwise withhold it and record the
      earliest typed failure.

Objective and effects: test only the pushed stopped-state safety-admission fix
that follows consumed attempt-029. The sole run may flash/reset the exact
package, seed private Wi-Fi and the generated local fixture, initialize and
mine the conservative profile for at most 600 active seconds, issue one each
pause/dismiss/IDENTIFY/resume/software-restart request, observe HTTP plus
WebSocket and receive-only USB, and perform same-device recovery, terminal safe
stop, child termination, and USB cleanup. Active mining observations must retain
strict fresh-safety validation; only exact stopped command states may use their
safe-stop witness. Automated effects and recovery retain finite repo-owned
bounds, and no human checkpoint is part of this attempt.

Evidence/privacy/recovery: detector and attempt directories are ignored
mode-`0700` roots with mode-`0600` files. Only the named aggregate projection
may become public after ready validation and redaction. Origins, ports,
hostnames, addresses, USB/network/process identity, credentials, fixture data,
frame text, sensors, and raw traces remain private. The earliest primary failure
must survive recovery; safe stop, child termination, and holder cleanup run on
every exit. Campaign start consumes attempt-030; no attempt-031 or same-contract
retry is authorized. Any detector/identity/build ambiguity, non-ready category,
missing or malformed projection, failed safe stop/recovery/cleanup, or absent
device stops with API-009 `implemented` and evidence withheld. OTA, erase,
factory reset, power cycle, external UART/BAP, USB duplex, pins/pads/GPIO,
arbitrary settings, external pool, stress, controls, fault injection, non-205
hardware, and human display claims remain prohibited.

Attempt-030 closure review: the exact package and private-input presence checks
passed, but a manual preflight incorrectly required a nonexistent manifest
`board` field and stopped before the detector command. Its nonzero exit was not
surfaced, so the one campaign invocation then failed as `process_failed` while
reading the absent detector artifact. No detector, USB, device, fixture, mining,
or recovery effect occurred; no attempt root or public projection exists and no
process remains. Attempt-030 is consumed by its campaign-start rule, no
attempt-031 is authorized here, and API-009 remains `implemented`.

Regression-backed follow-up: detector handoff absence and malformed content now
produce a typed, privacy-safe `evidence_invalid` result with
`detector_admitted=false` before workflow launch. Focused TypeScript and real
Bazel automation tests pass. The durable orchestration lesson requires
repo-owned package contracts and explicit exit-code checks before advancing;
the ordered Cargo gates, Bright Builds, firmware build, all 44 Bazel tests,
parity/progress, redaction, reference cleanliness, and diff checks pass. The
typed preflight correction is ready to publish.

### task-api009-programmatic-pilot-attempt-031 | 2026-08-15 | Run the fully gated programmatic command proof

- [x] Require clean synchronized pushed source containing commits `b2a8a066`
      and `35bc2280`, non-empty ignored Wi-Fi input without reading it, and
      fresh detector, attempt, and public-projection paths.
- [x] Run `just package` as the sole package admission/build surface and advance
      only after its zero exit and required manifest artifact are confirmed.
- [x] Create protected `scratch/api009-command-effects/detector-031`, run
      exactly one `just detect-ultra205`, and advance only after its zero exit,
      mode-`0600` output, and repo-owned one-device admission are confirmed.
- [x] Invoke exactly once:
      `just api-command-effects-campaign --private-root scratch/api009-command-effects/attempt-031 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/api009-command-effects/detector-031/detector.stdout --projection docs/parity/evidence/api009-command-effects/command-effects-projection.json --duration-seconds 600`.
- [x] Validate and redact the sealed projection only on the complete command,
      same-device restart, exact-package, recovery, safe-stop, and cleanup
      quorum; otherwise withhold it and record the earliest typed failure.

Objective and effects: prove the no-human API-009 programmatic campaign using
the published claim-specific and stopped-state corrections. The sole run may
flash/reset the exact package, seed private Wi-Fi and the generated local
fixture, initialize and mine the conservative profile for at most 600 active
seconds, issue one each pause/dismiss/IDENTIFY/resume/software-restart request,
observe HTTP plus WebSocket and receive-only USB, and perform same-device
recovery, terminal safe stop, child termination, and USB cleanup. Active mining
observations require strict fresh safety; exact stopped command states use their
safe-stop witness. Automated effects and recovery retain finite repo-owned
bounds; no human checkpoint is part of this attempt.

Evidence/privacy/recovery: detector and attempt directories are ignored
mode-`0700` roots with mode-`0600` files. Only the named aggregate projection
may become public after ready validation and redaction. Origins, ports,
hostnames, addresses, USB/network/process identity, credentials, fixture data,
frame text, sensors, and raw traces remain private. The earliest primary failure
must survive recovery; safe stop, child termination, and holder cleanup run on
every exit. Campaign start consumes attempt-031; no attempt-032 or same-contract
retry is authorized. Any nonzero preflight, missing artifact, detector/identity/
build ambiguity, non-ready category, malformed projection, failed safe stop,
recovery or cleanup, or absent device stops with API-009 `implemented` and
evidence withheld. OTA, erase, factory reset, power cycle, external UART/BAP,
USB duplex, pins/pads/GPIO, arbitrary settings, external pool, stress, controls,
fault injection, non-205 hardware, and human display claims remain prohibited.

Attempt-031 closure review: clean synchronized source, repo-owned exact-package
build, one protected detector admission, trusted runtime/package identity, the
local fixture, genuine notification, accepted work, safe stop, process cleanup,
and USB cleanup passed. The campaign stopped as `hardware_blocked` with earliest
category `safety_stale`: while the device was actively mining and hardware-ready,
all five required observations aged stale with an unchanged observation epoch.
Later terminal observations were fresh, proving transient recovery, but the
firmware correctly consumed the campaign rather than retroactively accepting an
active-mining safety lapse. No recovery request was required and the public
projection is withheld. Attempt-031 is consumed, no attempt-032 is authorized,
API-009 remains `implemented`, and no user action contributed to the result.

Terminal disposition: the programmatic platform has now distinguished and
fixed the prior host false positives from a genuine device/runtime safety
boundary. Weakening the one-second active-safety rule, ignoring the marker, or
rerunning unchanged would violate the safety and evidence contracts. Further
progress requires a separately scoped sensor-producer/I2C latency diagnostic
that can identify which bounded acquisition delayed the complete observation
sweep without exposing sensor values, followed by a verified root-cause change.

### task-parity-io002-adc-observation-attempt-002 | 2026-08-15 | Verify disabled-state ADC observation

- [x] Rebind the closed ADC evidence workflow to immutable plan
      `docs/parity/work-plans/20260815T222316Z-IO-002/PLAN.md`, protected
      `attempt-002` paths, and public schema ordinal 2 without changing the
      production ADC, safety, mining, or hardware-control behavior.
- [x] Run the complete focused and mandatory software/firmware/privacy gates,
      commit and push the exact implementation, and build its package before
      any device access.
- [x] Run only the plan's exact detector command and, after successful one-
      device admission plus local Wi-Fi input availability, its exact one-shot
      `just capture-adc-observation-evidence ... --capture-timeout-seconds 360`
      attempt-002 command.
- [x] Promote only IO-002 on the complete exact-package, passive disabled-state
      ADC/API quorum; otherwise withhold the projection, preserve
      `implemented`, record the earliest typed blocker and accepted terminal
      stop outcome, and do not retry.

Plan: `docs/parity/work-plans/20260815T222316Z-IO-002/PLAN.md`

Objective and effects: verify the corrected millivolt-domain contract with one
passive safe-state Ultra 205 observation. The sole run may factory-flash/reset
the exact clean package, seed private Wi-Fi, derive a same-origin device only
from protected current-session serial evidence, perform read-only HTTP,
WebSocket, and retained-log observations, and use at most one exact-package
recovery flash after a post-flash failure. It must keep mining and hardware
control disabled. Settings/restart requests, pool input, ASIC work, voltage,
frequency, fan or power control, raw ADC/GPIO/I2C, OTA, erase, fault injection,
physical power actions, direct UART, and all pin/pad/header/probe/jumper/solder/
signal manipulation are prohibited.

Evidence/privacy/recovery: wrapper and attempt directories are ignored
mode-`0700` roots with mode-`0600` files. Raw ADC values, stamps, logs, commands,
origins, ports, hostnames, USB/network/process identity, credentials, settings,
and traces remain private. Only the named aggregate projection may become
public after independent validation and redaction. Starting the capture
consumes attempt-002; no unchanged retry or attempt-003 is authorized. Every
post-flash failure preserves the earliest category and runs bounded recovery
and cleanup. Detector failure/ambiguity, missing credentials, unsafe state,
malformed or incomplete proof, failed cleanup/recovery, privacy failure, or a
nonzero command stops with IO-002 `implemented` and evidence withheld.

Verification: blocked before ADC validation at the typed task-contract boundary.
Promotion would prove only passive disabled-state ADC acquisition, cadence/
coherence, and exact HTTP/WebSocket projection on board 205. Energized-rail
values or accuracy, external calibration, induced failure, load behavior, long-
duration drift, other boards, and release readiness remain explicit non-claims.

Attempt-002 closure review: exact clean pushed package `d7efb2ea` admitted one
Ultra 205, observed safe boot with mining and hardware control disabled,
completed cleanup, and passed the independent protected system-info and
redaction contracts. ADC post-processing then stopped as `evidence_invalid`
because this task block omitted the literal `bitaxe-adc-observation-evidence-v1`
binding required by the immutable validator. The public ADC projection is
withheld, no retry ran, attempt-002 is consumed, and IO-002 remains
`implemented`. See
`docs/parity/work-plans/20260815T222316Z-IO-002/CLOSURE.md`.

Completion review: The corrected millivolt validator remains software-verified,
but this attempt produced no ADC parity evidence because its task-document
preflight was incomplete. A fresh task and immutable plan must test the real
task block, include the exact schema identifier, and authorize attempt-003;
this task authorizes neither a retry nor further hardware use.

### task-parity-io002-adc-observation-attempt-003 | 2026-08-15 | Preflight and verify disabled-state ADC observation

- [x] Bind `bitaxe-adc-observation-evidence-v1` to immutable plan
      `docs/parity/work-plans/20260815T225042Z-IO-002/PLAN.md`, protected
      `attempt-003` paths, and public schema ordinal 3.
- [x] Move the exact real task/plan contract check before sensitive inputs and
      hardware effects, declare the plan in Bazel runfiles, and regression-test
      both the checked-in artifacts and missing-schema fail-before-effect path.
- [x] Run the complete focused and mandatory software/firmware/privacy gates,
      commit and push the exact implementation, and rebuild its clean package
      before device access.
- [x] Run only the plan's exact detector command and, after successful one-
      device admission plus local Wi-Fi input availability, its exact one-shot
      `just capture-adc-observation-evidence ... --capture-timeout-seconds 360`
      attempt-003 command.
- [x] Promote only IO-002 on the complete exact-package passive disabled-state
      ADC/API quorum; otherwise withhold the projection, preserve
      `implemented`, record the earliest typed blocker and accepted stop
      outcome, and do not retry.

Plan: `docs/parity/work-plans/20260815T225042Z-IO-002/PLAN.md`

Objective and effects: close the newly discriminating attempt-002 task-contract
boundary before effects, then verify the corrected millivolt-domain contract
with one passive safe-state Ultra 205 observation. The sole run may factory-
flash/reset the exact clean package, seed private Wi-Fi, derive a same-origin
device only from protected current-session serial evidence, perform read-only
HTTP, WebSocket, and retained-log observations, and use at most one exact-
package recovery flash after a post-flash failure. It must keep mining and
hardware control disabled. Settings/restart requests, pool input, ASIC work,
voltage, frequency, fan or power control, raw ADC/GPIO/I2C, OTA, erase, fault
injection, physical power actions, direct UART, and every pin/pad/header/probe/
jumper/solder/signal manipulation are prohibited.

Evidence/privacy/recovery: wrapper and attempt directories are ignored
mode-`0700` roots with mode-`0600` files. Raw ADC values, stamps, logs, commands,
origins, ports, hostnames, USB/network/process identity, credentials, settings,
and traces remain private. Only the named aggregate projection may become
public after independent validation and redaction. Starting capture consumes
attempt-003; no unchanged retry or attempt-004 is authorized. Every post-flash
failure preserves the earliest category and runs bounded recovery and cleanup.
Detector failure/ambiguity, missing credentials, unsafe state, malformed or
incomplete proof, failed cleanup/recovery, privacy failure, or nonzero command
stops with IO-002 `implemented` and evidence withheld.

Verification: software and pre-hardware gates passed on clean pushed source
`9f48a1dbc07d7df83b05452e10edee4ff8989d12`. Detector-gated attempt-003
admitted one Ultra 205, observed the exact package in disabled safe state,
completed cleanup, and independently passed the protected system-info and ADC
input validators. The ADC input validator confirmed finite integer millivolts,
fresh and monotonic acquisition stamps, and coherent HTTP/WebSocket snapshots.
Final promotion stopped at `evidence_invalid` because the source-provenance
breadcrumb `.bitwidth = ADC_BITWIDTH_DEFAULT` occurs three times in pinned
`reference/esp-miner/main/adc.c` although the guard requires uniqueness. No
public ADC projection was published, no retry is authorized, and IO-002 remains
`implemented`.

Completion review: attempt-003 consumed its sole ordinal and correctly failed
closed after device capture. The remaining correction is software-only: replace
the ambiguous upstream bit-width breadcrumb with an exact contextual fragment
and ensure the checked-in source-semantics regression cannot pass from stale or
undeclared reference inputs. A fresh immutable plan must authorize any later
hardware ordinal. Energized-rail values or accuracy, external calibration,
induced failure, load behavior, long-duration drift, other boards, and release
readiness remain explicit non-claims.

## Future

### task-cross-platform-device-session-adapters | 2026-07-22 | Qualify Linux and Windows ESP device sessions

- [ ] Implement Linux physical/enumeration identity, exclusive ownership,
  receive-only observation, and bounded reacquisition behind the canonical
  device-session contract.
- [ ] Implement the corresponding Windows adapter without weakening
  exclusive ownership, request-once, or private-artifact guarantees.
- [ ] Add platform-native real-process tests.
- [ ] Keep unsupported platforms fail-closed until each exact adapter and its
      task-gated hardware evidence qualify.

Verification: Pending.

Completion review: Pending. macOS remains the only production adapter. Standing
task authorization permits ordinary implementation and task-gated evidence;
credentials, network discovery, direct UART or pin work, and evidence promotion
remain governed by their specific contracts.

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
