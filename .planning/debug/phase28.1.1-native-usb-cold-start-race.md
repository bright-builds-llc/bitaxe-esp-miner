---
status: awaiting_hardware
trigger: "Plan 13 passed physical lifecycle, USB ownership, passive capture, and cleanup, but the retained cold-start log contained no boot or listener markers."
created: 2026-07-12T04:00:00Z
updated: 2026-07-12T04:25:00Z
---

## Current Focus

hypothesis: Confirmed in source. Barrel power starts the ESP32-S3 before native USB exists, while the lifecycle waits for a post-plug human token before opening the passive monitor. The original boot and listener lines can therefore be gone before capture owns the node.
test: Arm an exact-node watcher before publishing the restore action, attach automatically on USB appearance, and validate replayed session-tagged boot/listener proof without requiring original cold-start bytes.
expecting: Software regressions prove response-free watcher arming, manual-spoof rejection, exact passive ownership, one boot session with both evidence states, private trace escrow, and bounded cleanup. A fresh exact-head hardware run remains the final confirmation.
next_action: Finish focused software verification, commit a clean exact HEAD, then run one detector-gated Plan 13 chain without a post-plug response.

## Symptoms

expected: Once both power paths are removed, the lifecycle is already observing the selected node before the user restores barrel power and USB; capture proves that one boot reached listener readiness even when native USB missed original bytes.
actual: The old lifecycle published a restore checkpoint, waited for `plan13-barrel-then-usb-restored`, then watched for USB and opened the monitor. The accepted-state validator rejected the resulting zero-marker capture.
errors: `accepted_state_diagnostic_error: cold-start capture does not contain exactly one stable boot`; terminal `blocked_safe_evidence_invalid`.
reproduction: Complete Plan 13 removal, restore barrel first and USB second, then acknowledge restoration. USB ownership succeeds, but early application logs may precede monitor attachment.

## Feedback Loop

command: `node scripts/phase28.1.1-hardware-attempt-state-test.mjs && node scripts/phase28.1.1-accepted-state-lifecycle-compare-test.mjs`
red_output: The state authority exposed no response-free public restore action, and the lifecycle validator exposed no session-tagged boot-evidence parser.
properties: Deterministic, hardware-free, and located at the state/evidence contract that failed in the real attempt.

## Working Diagnosis

root_cause: Native USB enumeration is not an always-connected serial transport. With barrel restored first, firmware can emit boot/listener logs before the selected serial node exists. Waiting for a human post-plug response adds an unavoidable race; serial ownership correctness cannot recover bytes emitted before attachment.
confidence: high; confirmed by lifecycle ordering, firmware replay allowlist, and the successful zero-marker hardware trace.
smallest_correct_seam: Replace the restore response checkpoint with an already-armed exact-node watcher and make boot/listener readiness replayable as one opaque per-boot session.

## Fix in Progress

- The private attempt state uses a response-free `restore_watcher_armed` action, a 30-minute exact-node appearance window, and a 60-second passive ownership-attachment bound while retaining the 4,145,000 ms lease.
- Firmware generates one 128-bit hardware-RNG boot nonce, retains redacted `booted` and `listener_armed` proof in Plan 13 evidence mode, and replays only boot evidence plus accepted-state snapshots every 10 seconds for 1,880,000 ms.
- Validation requires original boot/listener lines for reinit, but permits replay-only cold-start proof; equivalent duplicates pass while malformed, missing, conflicting, or multiple-session proof fails.
- Raw monitor, wrapper, and session traces are escrowed under a mode-0700 ignored root as mode-0600 files before validation and again before tombstoning.

## Remaining Verification

- Complete the exact-head adapter and diagnostic suites, shell formatting/lint, affected Bazel targets, reference guard, and canonical Rust checks.
- Do not mark this session resolved until a fresh exact-head Plan 13 hardware chain observes the armed action before restoration and passes the retained cold-start validator without a post-plug message.

## Software Verification

- timestamp: 2026-07-12T04:25:00Z
  checked: state authority, lifecycle comparator, strict classifier, exact-head broker/adapter, diagnostic wrapper, and passive monitor suites
  found: The state/lifecycle/classifier Node tests pass; the exact-head suite passes all 84 invalid cases; the diagnostic wrapper passes its response-free lifecycle fixtures; and the phase13 monitor suite proves a mode-0600 active-owner readiness signal plus passive cleanup.
  implication: The software contract is internally consistent and the next hardware attempt can be created only after the coordinating agent completes canonical formatting, Bazel/reference, Rust, commit, and push gates.
- timestamp: 2026-07-12T04:25:00Z
  checked: shell and source hygiene
  found: Bash/Node syntax checks, `shfmt -d`, warning-level `shellcheck`, `git diff --check`, and the reference-tree diff pass. The direct `bitaxe-api` replay tests pass. A direct host-target `bitaxe-firmware` test is not supported because `esp-idf-sys` rejects `aarch64-apple-darwin`; the canonical ESP target build remains required.
  implication: No known software-only blocker remains, but firmware compilation and real native-USB timing still require the repo-owned target and hardware gates.

## Resolution State

root_cause: The old post-plug response checkpoint serialized human acknowledgment ahead of monitor attachment on a transport that did not exist at barrel-power boot. The resulting race could not be fixed by serial ownership cleanup or a countdown.
fix: Publish restoration only after a private exact-node watcher is armed; treat USB appearance as the event; acquire passive ownership within 60 seconds; replay redacted per-boot session proof and accepted-state snapshots inside a bounded window; validate reinit originals and cold-start replay separately; escrow raw traces privately before validation/tombstoning.
hardware_status: pending fresh exact-head Plan 13 confirmation; no hardware was accessed during this debug implementation.
