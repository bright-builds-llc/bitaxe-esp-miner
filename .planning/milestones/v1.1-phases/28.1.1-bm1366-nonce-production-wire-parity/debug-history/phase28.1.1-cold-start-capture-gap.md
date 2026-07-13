---
status: closed_wont_do_unresolved
trigger: "Phase 28.1.1 UAT Test 3 cannot reliably capture all five one-shot accepted-state markers across a true Ultra 205 cold start because removing both power paths also removes the native USB console."
created: 2026-07-10T02:10:00Z
updated: 2026-07-13T16:30:00Z
---

## Current Focus

closure: Closed without resolution by the Phase 28.1.1 Won't Do decision. Preserve this record as historical diagnostic evidence; do not resume it or infer verification.

hypothesis: Confirmed and implemented. The evidence-transport and evidence-correctness defects are closed, including independent-review fixes for process-tree cleanup, deadline-crossing tokens, and unavailable precedence. Hardware closure remains blocked because the strict five-stage candidate belongs to superseded head 4e2d165, hardware-attempt head d275a0e reached an expired finite-recovery checkpoint, and review-fix head ab7f5b9 has no eligible package, reinit, or lifecycle arm.
test: Strict five-stage exact-identity audit, hardware-attempt bounded detector recovery, persisted monotonic checkpoint validation, and mandatory process-tree cleanup.
expecting: A lifecycle watcher may arm only when the current exact head has a timely validated detector gate and a strict five-stage reinit member. Stale package identity and expired checkpoint state must stop without repair.
next_action: Do not resume or refresh Plan 11. Its finite recovery contract is exhausted. Any further hardware attempt requires a new formally planned recovery contract.

## Symptoms

expected: With one corrected accepted-state package retained, a true removal and restoration of both barrel/DC and USB power produces one stable boot, all five accepted-state stages, and a redacted comparison with an exact-commit flash/reinit capture.
actual: USB-only cycling leaves the device powered by barrel/DC. Barrel-only cycling leaves the ESP32 powered over USB. Removing both paths creates a true cold start but also removes the native USB serial device; the existing no-reset monitor process does not survive re-enumeration, and late attachment misses the one-shot boot markers.
errors: The two safe cold-start monitor logs are header-only: each has ten wrapper lines, zero firmware boot markers, zero accepted-state markers, and `capture_status=timed_out_after_capture`. No firmware error is established by those files.
reproduction: Start `scripts/phase13-monitor-capture.sh --no-reset` against the native USB port, remove both device power paths, restore them, and allow the bounded capture to finish. The serial endpoint disappears, while the wrapper has no reconnect/retry state machine.
started: Exposed while resuming the Phase 28.1.1 physical cold-start lifecycle prerequisite after the Phase 27 bridge stack-overflow correction.

## Eliminated

- ASIC observation/classification defect as the cause of the missing log: eliminated. The exact-commit flash/reinit capture emitted all five closed category markers, and the comparator accepts/deduplicates stages through a stage-keyed map.
- A USB-only or barrel-only procedure as a true cold start: eliminated by the confirmed two-supply topology. Either remaining path keeps part of the device powered.
- Reset/EN after monitor attachment: eliminated. It is a warm software/hardware reset and re-runs ASIC initialization; it is not evidence for removal/restoration of both power paths.
- Reflash as the cold-start member: eliminated. It is the separate reinit member of the lifecycle pair and changes the initialization/recovery operation being compared.
- NVS/factory erase: eliminated. It is destructive, changes the configuration boundary, is unnecessary for serial observability, and would invalidate retained-package lifecycle evidence.
- Waiting before or between ASIC initialization stages: eliminated. It would make the observer change the timing being measured.
- Stale `DEVICE_URL`, mDNS, ARP/router inspection, or network scan: eliminated by repo-local hardware rules. `/api/system/logs` retains the needed markers, but reaching it still requires a fresh same-session target; the current cold-start path provides no reliable fresh target after the USB console race.
- Raw register logging or persistent raw dumps: eliminated. The existing category marker is sufficient and the evidence contract prohibits promoting raw register values or secrets.

## Evidence

- timestamp: 2026-07-10T02:10:00Z
  checked: `.planning/phases/28.1.1-bm1366-nonce-production-wire-parity/28.1.1-UAT.md:31-35,52-62`
  found: Test 3 is the sole major issue. The failed truth explicitly requires a true both-power-path cold start after USB re-enumeration without changing ASIC timing or exposing raw values.
  implication: The gap is specifically in evidence capture semantics, not permission to substitute reset/reflash or relax redaction.
- timestamp: 2026-07-10T02:10:00Z
  checked: `scripts/phase13-monitor-capture.sh:65-83,93-139`
  found: The wrapper opens `espflash monitor` once, optionally with `--no-reset`, and then waits only for that child to exit or time out. It has no port-disappearance, port-reappearance, or monitor-reattach state.
  implication: Arming this wrapper before a true power removal cannot preserve native USB capture across device re-enumeration.
- timestamp: 2026-07-10T02:10:00Z
  checked: safe scratch status extraction from `cold-start-current/monitor.raw.log` and `cold-start-fixed-live/monitor.raw.log`
  found: Both files contain ten wrapper/header lines, zero `bitaxe-rust boot` markers, zero `accepted_state_snapshot` markers, and `capture_status=timed_out_after_capture`.
  implication: The attempted watcher proves only that the host wrapper remained bounded; it does not prove a firmware boot or lifecycle outcome.
- timestamp: 2026-07-10T02:10:00Z
  checked: exact-commit accepted-state capture event timing using category/event-only extraction
  found: Boot identity appeared at about 1.1 seconds; boot-owned accepted-state stages appeared at about 3.4, 11.8, and 12.2 seconds; Wi-Fi/API became ready at about 16.3/16.6 seconds; the retained-runtime stages appeared at about 23.9 and 31.3 seconds.
  implication: Post-enumeration attachment has a real race with the first three one-shot stages. Later runtime stages do not reconstruct the missed boot observations.
- timestamp: 2026-07-10T02:10:00Z
  checked: `firmware/bitaxe/src/asic_adapter.rs:289-303,435-445,522-524` and `firmware/bitaxe/src/asic_adapter/production.rs:254-301`
  found: The first three stages are captured exactly once at their actual boot boundaries using bounded safe reads. Each observation is immediately rendered to a category-only marker and passed to `info_retained`; no raw register value is logged.
  implication: Those reads and their timing should remain untouched. The fix belongs after classification, at the evidence transport/replay seam.
- timestamp: 2026-07-10T02:10:00Z
  checked: `firmware/bitaxe/src/live_stratum_runtime.rs:811-873,898-916,1100-1131,1517-1520`
  found: Post-mask-reload and post-first-work use the same closed marker and retained-log path. Post-first-work completes after all earlier reachable stages and is a natural existing-runtime point for a one-shot replay; it occurs after ASIC initialization and does not require another read.
  implication: A bounded category-only replay after the final stage can expose the already-recorded observations without delaying or repeating ASIC initialization.
- timestamp: 2026-07-10T02:10:00Z
  checked: `firmware/bitaxe/src/log_buffer.rs:7-39`, `firmware/bitaxe/src/http_api.rs:321-335`, and `crates/bitaxe-api/src/logs.rs:10-13,35-139`
  found: Accepted-state markers already enter a RAM-only retained log buffer and are downloadable from `/api/system/logs`; the buffer is reset with the firmware process and has 512 KiB primary/32 KiB fallback capacity.
  implication: Retention already exists and proves the marker does not need to be re-read from the ASIC. HTTP retrieval alone is not the smallest complete fix because fresh target discovery remains unavailable after the console race.
- timestamp: 2026-07-10T02:10:00Z
  checked: `scripts/phase28.1.1-accepted-state-diagnostic.sh:90-107,110-128,135-153`
  found: Hardware mode always packages and delegates to the Phase 27 flash/evidence wrapper. `--attempt lifecycle` omits the `accepted_state_snapshot` package token, but still packages and flashes.
  implication: The current lifecycle branch cannot represent either half of its intended contract correctly: it destroys retained-package cold-start state, and its newly built lifecycle image is not guaranteed to contain the accepted-state instrumentation.
- timestamp: 2026-07-10T02:10:00Z
  checked: `scripts/phase28.1.1-accepted-state-diagnostic-test.sh:25-85`
  found: Tests cover blocked lifecycle input and hardware accepted-state input only. There is no hardware lifecycle test proving no package/reflash action, retained package identity, re-enumeration/reattach behavior, or five-stage replay normalization.
  implication: The wrapper contract gap was not guarded and could remain latent while lifecycle execution was blocked on physical prerequisites.
- timestamp: 2026-07-10T02:10:00Z
  checked: `scripts/phase28.1.1-accepted-state-compare.mjs:61-117`
  found: The comparator stores parsed observations in a `Map` keyed by the five closed stage names, so repeated equivalent stage input resolves deterministically to one observation per stage.
  implication: A distinct replay transport can be normalized/deduplicated safely before comparison, but the raw serial marker vocabulary should distinguish original observations from replay to keep counts and provenance unambiguous.
- timestamp: 2026-07-10T02:22:00Z
  checked: `firmware/bitaxe/src/live_stratum_runtime.rs:781-793`
  found: Continuous result handling has one existing transition from production-ready to `listener_armed`, after boot-owned ASIC initialization completes and before post-mask-reload/post-first-work observations.
  implication: Replaying the retained, closed boot-stage markers at this transition does not add a task, sleep, ASIC read, or pre-init side effect. The later two stages continue to arrive on their original runtime boundaries.
- timestamp: 2026-07-10T02:22:00Z
  checked: `scripts/phase27-live-hardware-bridge-evidence.sh:95-106`
  found: The delegated hardware capture command is `just flash-monitor`, confirming that the current lifecycle wrapper cannot use it for the cold-start member without reflashing.
  implication: The gap plan needs a separate retained-package capture path rather than another flag passed through the existing Phase 27 flash wrapper.

## Working Diagnosis

root_cause: The design assumed that `--no-reset` on a single native-USB monitor process was equivalent to an independent capture channel across total device power loss. It is not: the USB endpoint disappears, the monitor wrapper does not reattach, and three accepted-state markers can be emitted before a new monitor is reliably attached. The planned lifecycle wrapper also always packages/flashes and omits the diagnostic token for `--attempt lifecycle`, so it has no valid retained-package cold-start path.
confidence: high.
smallest_correct_seam: Preserve the existing one-shot safe reads and category rendering. Reuse the RAM retained-log buffer to select only exact closed `accepted_state_snapshot` lines and replay the boot-owned lines once at the existing `listener_armed` transition, after ASIC initialization. Do not replay ASIC reads. The post-mask-reload and post-first-work markers then arrive normally; normalization deduplicates by the five stage keys. Extend the Phase 28.1.1 lifecycle wrapper with an explicit retained-package cold-start mode that performs no package, flash, reset, board-info, network discovery, or credential read during the cold-start member; it waits for the previously detector-gated USB port to reappear, attaches `espflash monitor --no-reset`, captures at least 360 seconds, and requires five unique normalized stages. The exact-commit flash/reinit member must use the same replay-capable package, then leave it retained for the cold-start member.
files_likely_in_fix_plan:
  - `firmware/bitaxe/src/asic_adapter.rs`
  - `firmware/bitaxe/src/live_stratum_runtime.rs`
  - a small firmware-owned accepted-state replay store module or equivalent narrow helper
  - `scripts/phase28.1.1-accepted-state-diagnostic.sh`
  - `scripts/phase28.1.1-accepted-state-diagnostic-test.sh`
  - a focused pure test for five-stage bounded replay and redaction
verification_direction: Build one accepted-state/replay-capable exact-commit package; pass detector/reference gates; flash and capture the reinit member for at least 360 seconds; retain the package; arm the cold-start reattach wrapper; remove both power paths, restore both, and capture at least 360 seconds without any reset/reflash; require one boot, stable runtime, exactly five unique normalized stage observations, no raw/credential/network fields in promoted output, and a category-only lifecycle comparison. Post-capture board-info is allowed only after capture is complete.

## Root Cause

root_cause: The lifecycle design treated one `espflash monitor --no-reset` process on the Ultra 205 native USB device as an independent capture channel across complete power removal. Complete removal destroys that endpoint, while the wrapper neither reconnects nor reopens it. The boot-owned diagnostic stages are correctly observed and retained but can be emitted before a new host monitor attaches. Separately, the Phase 28.1.1 lifecycle wrapper always calls the package/flash workflow and omits `accepted_state_snapshot` for its lifecycle package, so it cannot execute the retained-package half of the promised pair.
confidence: high.
artifacts:
  - `scratch/phase28.1.1-accepted-state/cold-start-current/monitor.raw.log` — safe status inspection: header-only, zero boot/accepted-state markers
  - `scratch/phase28.1.1-accepted-state/cold-start-fixed-live/monitor.raw.log` — safe status inspection: header-only, zero boot/accepted-state markers
  - `scratch/phase28.1.1-accepted-state/20260710T003559Z-accepted-state/hardware/live-capture-runtime/flash-monitor.log` — ignored raw exact-commit capture; only event timing/category counts were inspected
missing:
  - a replay-capable exact-commit package that leaves all ASIC observation points unchanged
  - a lifecycle cold-start wrapper branch that never packages, flashes, resets, or runs board-info before capture
  - a USB disappearance/reappearance reattach fixture and hardware run
  - five unique cold-start stage markers and category-only cold-vs-reinit comparison
suggested_fix_direction: Use `gsd-plan-phase --gaps` to add one post-init category replay seam and one no-flash lifecycle capture seam. The replay must select only exact redacted accepted-state lines already retained in RAM, run once at listener readiness, and never repeat a register read. The wrapper must distinguish `reinit` from `cold-start`, prove the cold-start branch never calls package/flash/reset, reattach after USB re-enumeration with `--no-reset`, normalize stage duplicates, keep raw logs ignored, and promote only closed category/count fields.
secondary_option_not_selected: A host-only reattach wrapper could derive one fresh origin-only `DEVICE_URL` from the same post-cold-start serial session and download `/api/system/logs`; this does not require stale discovery and is narrower in code volume. It is not the recommended seam because it makes serial evidence depend on Wi-Fi/API target acquisition and handling a broader raw retained-log response. Direct category-only replay is fail-closed, avoids target discovery entirely, and exposes only the already-approved evidence vocabulary.

## Plan 10 Execution Update

- timestamp: 2026-07-10T03:19:10Z
  checked: `70f4631` and focused/full Rust verification
  found: Exact complete-line selection and a 90-second/2-second bounded replay cadence pass host tests; firmware arms only at `listener_armed` under `accepted_state_snapshot` and emits with `log::info!` rather than `info_retained`.
  implication: The firmware fix does not repeat ASIC reads, add tasks/sleeps, or change default firmware behavior.
- timestamp: 2026-07-10T03:19:10Z
  checked: `5b284d4`, Node lifecycle fixtures, shell routing fixtures, `shellcheck`, and `shfmt -d`
  found: The lifecycle wrapper reuses the retained package, requires disappearance/reappearance inside one 89-second maximum budget, invokes `--no-reset` for at least 360 seconds, and excludes package/flash/reset/detector/credential actions between arming and capture completion.
  implication: The original evidence-transport and wrapper-routing defects are regression guarded.
- timestamp: 2026-07-10T03:19:10Z
  checked: first exact-commit 360-second reinit attempt
  found: One boot, one listener-ready marker, zero hazards, and three equivalent boot-owned stages. The pool session subscribed/authorized and then reconnected before work dispatch, leaving the exact five-stage prerequisite incomplete.
  implication: The physical checkpoint must remain unarmed; replay cannot synthesize the two runtime-owned stages.
- timestamp: 2026-07-10T03:19:10Z
  checked: second exact-commit 360-second reinit attempt and final `just detect-ultra205`
  found: The delegated capture closed `blocked_safe_prerequisite` with no firmware markers. The final detector found the USB candidate but `board-info` could not connect.
  implication: Repo hardware guidance requires stopping without another flash, reset, scan, stale target, or physical power request.

## Final Plan 10 Disposition

implementation_status: complete
hardware_lifecycle_status: blocked_safe_prerequisite
lifecycle_checkpoint_armed: false
physical_power_action_performed: false
phase30_promotion_input: pending

## Plan 11 Execution Update

- timestamp: 2026-07-11T01:57:59Z
  checked: Plan 11 software commits `1fe4918`, `c210f11`, `a6b634e`, focused review commit `4e2d165`, and polling compatibility commit `d275a0e`
  found: Rust formatting is clean; accepted-state completeness, replay timing, measured USB absence, checkpoint deadlines, cleanup, and tri-state evidence denial have deterministic regression coverage. The focused review has no remaining warning.
  implication: The earlier evidence-correctness defects are closed without changing ASIC initialization or the Phase 30 promotion gate.
- timestamp: 2026-07-11T01:57:59Z
  checked: exact-identity category summary for the 360-second reinit candidate
  found: Source commit `4e2d16524d57037e0814b191fed1b87fca4d0623` produced one stable boot, one listener-ready marker, zero hazards, all five closed stages, and a passing strict self-comparison.
  implication: The candidate was complete for that source identity, but commit `d275a0e7af6a1534df5fca07820066791ae4af19` changed the lifecycle script and made the prior package stale. It could not arm the hardware-attempt run and cannot be imported into review-fix head `ab7f5b9` or documentation-closure head `bf6897b`.
- timestamp: 2026-07-11T01:57:59Z
  checked: `d275a0e` hardware-attempt finite recovery checkpoint state
  found: The initial detector failed `board-info`; USB replug was consumed; the operator reported the both-power recovery, but the persisted deadline of `23949477` monotonic milliseconds had expired when continuation observed `44667284` milliseconds.
  implication: Plan 11 lines 183-185 require a conservative stop without inference, repair, refresh, or another recovery attempt.
- timestamp: 2026-07-11T01:57:59Z
  checked: post-expiry continuation audit
  found: A continuation mistakenly invoked one bounded repo-owned detector before asserting expiry. It reported exactly one ESP32-S3 and successful `board-info`, but the observation is invalid and unpromoted. No credential access, package, flash, reset, monitor, or later hardware command ran; cleanup proved no child remained.
  implication: The invalid observation cannot satisfy detection, package, reinit, lifecycle, or promotion prerequisites. The finite recovery contract remains exhausted.

## Final Plan 11 Disposition

implementation_status: complete_review_warnings_resolved
hardware_lifecycle_status: blocked_safe_prerequisite
hardware_attempt_valid_detector_status: unavailable
hardware_attempt_valid_reinit_status: unavailable
lifecycle_checkpoint_armed: false
lifecycle_power_action_accepted: false
cleanup_verified: true
live_child_process: false
phase30_promotion_input: pending

## Independent Review Closure

- timestamp: 2026-07-11T02:27:38Z
  checked: independent review findings and commit `ab7f5b932a9c86045eb5dba66c1ec3ca3c3fe477`
  found: Direct-child cleanup could leave an orphan descendant watcher; a token could complete at or after its deadline; and Rust missing observations could be masked by mismatch or result progress.
  implication: The lifecycle contract was not fully fail closed until process-group termination, post-read deadline validation, and unavailable-first classification precedence were added.
- timestamp: 2026-07-11T02:27:38Z
  checked: Phase 13 process-tree suite, Phase 28 lifecycle suite, and focused accepted-state Rust tests
  found: Timeout, TERM, normal-exit, detached-descendant cancellation, token-at-deadline rejection, unavailable-plus-mismatch, and unavailable-plus-result-progress regressions pass.
  implication: The three independent findings are resolved in host software. Because `ab7f5b9` changed exact head without hardware access, it supplies no reusable detector/package/reinit/lifecycle evidence and preserves `blocked_safe_prerequisite`.
