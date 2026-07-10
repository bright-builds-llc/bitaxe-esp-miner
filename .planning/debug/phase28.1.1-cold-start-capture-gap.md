---
status: root_cause_found
trigger: "Phase 28.1.1 UAT Test 3 cannot reliably capture all five one-shot accepted-state markers across a true Ultra 205 cold start because removing both power paths also removes the native USB console."
created: 2026-07-10T02:10:00Z
updated: 2026-07-10T02:22:00Z
---

## Current Focus

hypothesis: Confirmed. The failed truth is an evidence-transport race, compounded by a lifecycle wrapper that always packages/flashes and therefore cannot represent a retained-package cold start. The accepted-state observations themselves already reduce raw ASIC reads to closed category markers correctly.
test: Trace marker production, retention, serial capture, lifecycle wrapper branching, and API log retrieval; compare those contracts with the safe scratch capture outcomes and exact-commit event timing.
expecting: A correct gap plan will preserve the one-shot ASIC observation timing, retain only the five category markers, replay them after USB re-enumeration on the existing runtime path, and add an explicit no-flash cold-start capture mode that reattaches without reset.
next_action: Plan the gap around category replay after initialization plus a retained-package, no-flash USB reattach capture. Do not change ASIC reads or initialization timing.

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
