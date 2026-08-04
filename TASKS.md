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

### task-parity-cfg006-defaults-matrix | 2026-08-04 | Complete board defaults matrix

- [x] Add typed exact defaults for all 20 numbered upstream board seeds and the
      explicit custom seed.
- [x] Bind every discriminator to a provenance-bearing golden fixture and the
      existing board catalog.
- [x] Run focused and mandatory gates, then transition only `CFG-006` to
      `implemented` while withholding every non-205 hardware claim.

Plan: `docs/parity/work-plans/20260804T133030Z-CFG-006/PLAN.md`

Authorization: pure software and public upstream seed data only. No hardware,
credentials, network, settings, mining, controls, OTA, direct UART, or pins.

Verification: Focused strict Clippy, all 51 `bitaxe-config` tests through Cargo
and Bazel, the mandatory Rust sequence, Bright Builds, all 28 Bazel test
targets, parity/progress, redaction, reference cleanliness, and diff checks
passed on implementation commit `1583feb3`.

Completion review: The bounded pure matrix implementation is complete and
`CFG-006` is now `implemented` with `unit,golden` evidence. The task remains
active and unarchived because the parity row is not verified: live seeded
defaults and runtime behavior for non-205 profiles require separately admitted
hardware evidence. No runtime selection or hardware behavior changed.

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

### task-parity-io001-i2c-retry-contract | 2026-08-04 | Match shared I2C transfer policy

- [x] Add an exact host-testable 500 ms, three-attempt, 10 ms-delay transfer
      policy matching the pinned reference.
- [x] Route every display, sensor, and actuation transfer through the single
      retry owner without widening address or effect capabilities.
- [x] Extend bypass regressions, build the real firmware, run all mandatory
      gates, and transition only `IO-001` to `implemented`.

Plan: `docs/parity/work-plans/20260804T135918Z-IO-001/PLAN.md`

Authorization: software-only I2C contract work. No hardware, credentials,
network requests, mining, voltage/fan effects, OTA, direct UART, or pins.

Verification: Four focused retry tests, source-ownership tests, and the real
ESP-IDF firmware build passed. The mandatory ordered Rust sequence, Bright
Builds checks, all 29 Bazel test targets, parity/progress, redaction, reference
cleanliness, and diff checks passed on implementation commit `b15073c9`.

Completion review: The exact bounded transfer policy is implemented and
`IO-001` is `implemented` with `unit,workflow,hardware-smoke` evidence. The task
remains active and unarchived because the hardware breadcrumb predates the
retry change; live transient-fault, timeout, and shared-bus behavior remain
unverified. No hardware, credentials, mining, controls, OTA, UART, or pins ran.

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

### task-parity-net003-scan-ipv6 | 2026-08-04 | Implement Wi-Fi scan and IPv6 reporting

- [x] Add the bounded 20-network scan response and explicit numeric auth-mode
      plus link-local/global IPv6 projection contracts.
- [x] Retain one ESP-IDF Wi-Fi owner for exclusive scans, restore AP-only mode,
      register the access-gated endpoint, and publish station-only IPv6 events.
- [x] Add focused/ownership regressions, build the real firmware, run every
      mandatory gate, and transition only `NET-003` to `implemented`.

Plan: `docs/parity/work-plans/20260804T170000Z-NET-003/PLAN.md`

Authorization: local software and build work only. No hardware attempt,
credentials, external network request, mining, ASIC traffic, voltage/fan/power
effect, OTA, recovery, direct UART, or pins.

Verification: Five focused scan/IPv6 tests, 23 API-comparator tests, six Wi-Fi
ownership tests, the synthetic route fixture, and the real ESP32-S3 build pass.
The ordered Rust sequence, Bright Builds, all 30 Bazel targets, API compare,
parity/progress, redaction, reference cleanliness, and diff checks also pass.

Completion review: The bounded scan and station IPv6 implementation is
complete, and `NET-003` is `implemented` with `unit,workflow,api-compare`
evidence under transition `20260804T174500Z-NET-003`. This task remains active
and unarchived because live scan results, connection preservation, IPv6
assignment, and API behavior remain below verified and require separate
detector-gated evidence.

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
