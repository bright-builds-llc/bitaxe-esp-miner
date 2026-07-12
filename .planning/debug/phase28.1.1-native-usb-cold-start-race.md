---
status: awaiting_hardware
trigger: "Plan 13 passed physical lifecycle, USB ownership, passive capture, and cleanup, but the retained cold-start log contained no boot or listener markers."
created: 2026-07-12T04:00:00Z
updated: 2026-07-12T05:00:00Z
---

## Current Focus

hypothesis: The prearmed watcher repair is hardware-confirmed, but replay ownership is still too narrow. Replay runs only from the live Stratum socket pump, so optional Wi-Fi/pool-session progress can suppress boot evidence even after native USB is attached correctly.
test: Move the allowlisted 10-second replay cadence to a boot-lifetime task with the same 1,880,000 ms window, remove Stratum-owned replay, and verify the ownership seam statically plus the timing schedule in Rust.
expecting: Software checks prove that booted evidence replays independently of network progress and listener evidence joins the same session if and when listener readiness occurs. A later fresh exact-head hardware run remains required to prove native-USB delivery.
next_action: Complete focused and canonical verification, commit and push the boot-lifetime replay correction, and stop before another hardware run.

## Symptoms

expected: Once both power paths are removed, the lifecycle is already observing the selected node before the user restores barrel power and USB; capture proves that one boot reached listener readiness even when native USB missed original bytes.
actual: The repaired lifecycle armed the exact-node watcher before restoration, observed USB automatically, acquired passive ownership within the attachment bound, captured for 360 seconds, and cleaned up with zero holders. The raw serial stream was still empty because firmware replay was conditional on entering and remaining in the Stratum socket pump.
errors: Cold monitor bytes `0`; boot-evidence markers `0`; accepted-state snapshots `0`; terminal `blocked_safe_evidence_invalid`. The session trace reports pre/post readiness, expected active ownership, stable identity, timeout-after-capture, and complete cleanup.
reproduction: On exact HEAD `4891ce06bb51f872fd41c0baa2412cd660c877eb`, complete reflash/reinit, remove both power paths, wait for `plan13-restore-watcher-armed-v1`, then restore barrel and USB without a response. Attachment succeeds but no replay bytes arrive during the bounded capture.

## Feedback Loop

command: `node scripts/phase28.1.1-hardware-attempt-state-test.mjs && node scripts/phase28.1.1-accepted-state-lifecycle-compare-test.mjs`
red_output: The state authority exposed no response-free public restore action, and the lifecycle validator exposed no session-tagged boot-evidence parser.
properties: Deterministic, hardware-free, and located at the state/evidence contract that failed in the real attempt.

## Working Diagnosis

root_cause: Native USB required the prearmed watcher, and boot proof additionally requires a lifecycle independent of optional services. The first repair stored replay state correctly but invoked its cadence only from `pump_live_socket_until_cleanup`, making proof contingent on Wi-Fi and Stratum progress.
confidence: high; the hardware trace proves transport appearance/ownership/cleanup while capturing zero bytes, and source inspection shows there is no other replay owner.
smallest_correct_seam: Keep the watcher and validators unchanged, but move the existing allowlisted replay cadence to a boot-lifetime diagnostic task started with boot evidence.

## Fix in Progress

- The private attempt state uses a response-free `restore_watcher_armed` action, a 30-minute exact-node appearance window, and a 60-second passive ownership-attachment bound while retaining the 4,145,000 ms lease.
- Firmware generates one 128-bit hardware-RNG boot nonce and retains redacted `booted` and `listener_armed` proof in Plan 13 evidence mode. The follow-up correction moves the allowlisted replay task out of the Stratum adapter and schedules it from boot for 10-second ticks strictly before 1,880,000 ms.
- Validation requires original boot/listener lines for reinit, but permits replay-only cold-start proof; equivalent duplicates pass while malformed, missing, conflicting, or multiple-session proof fails.
- Raw monitor, wrapper, and session traces are escrowed under a mode-0700 ignored root as mode-0600 files before validation and again before tombstoning.

## Remaining Verification

- Complete the boot-lifetime ownership regression, shell formatting/lint, affected Bazel targets, reference guard, and canonical Rust checks.
- Commit and push a clean exact HEAD, but do not run another hardware chain in the failed attempt or mark this session resolved without later retained cold-start proof.

## Software Verification

- timestamp: 2026-07-12T04:25:00Z
  checked: state authority, lifecycle comparator, strict classifier, exact-head broker/adapter, diagnostic wrapper, and passive monitor suites
  found: The state/lifecycle/classifier Node tests pass; the exact-head suite passes all 84 invalid cases; the diagnostic wrapper passes its response-free lifecycle fixtures; and the phase13 monitor suite proves a mode-0600 active-owner readiness signal plus passive cleanup.
  implication: The software contract is internally consistent and the next hardware attempt can be created only after the coordinating agent completes canonical formatting, Bazel/reference, Rust, commit, and push gates.
- timestamp: 2026-07-12T04:25:00Z
  checked: shell and source hygiene
  found: Bash/Node syntax checks, `shfmt -d`, warning-level `shellcheck`, `git diff --check`, and the reference-tree diff pass. The direct `bitaxe-api` replay tests pass. A direct host-target `bitaxe-firmware` test is not supported because `esp-idf-sys` rejects `aarch64-apple-darwin`; the canonical ESP target build remains required.
  implication: No known software-only blocker remains, but firmware compilation and real native-USB timing still require the repo-owned target and hardware gates.
- timestamp: 2026-07-12T05:00:00Z
  checked: boot-lifetime replay correction
  found: The ownership shell regression, lifecycle/state/classifier/exact-head/monitor suites, seven host cadence/allowlist tests, four affected Bazel targets, canonical `//firmware/bitaxe:firmware` ESP32-S3 build, reference guard, and mandatory Rust format/Clippy/build/test sequence all pass. The exhaustive broker suite retains all 84 invalid cases.
  implication: The follow-up software seam is clean and may be committed as a fresh exact HEAD. Hardware delivery remains deliberately unclaimed.

## Hardware Verification

- timestamp: 2026-07-12T04:54:00Z
  checked: fresh exact-head Plan 13 chain at `4891ce06bb51f872fd41c0baa2412cd660c877eb`
  found: Detector, credential binding, reference guard, package, and reflash/reinit completed. After both power paths were removed, the owner armed the restore watcher before action publication, observed USB restoration without a response token, acquired the stable holder-free node, verified the expected active monitor owner, captured for 360 seconds, and returned to zero processes and zero holders. The cold serial payload contained zero bytes and therefore zero boot evidence, listener evidence, or accepted-state snapshots.
  implication: The native-USB race and cleanup paths are repaired. The remaining blocker is firmware replay availability, not USB appearance, monitor ownership, capture duration, or dangling handles.
- timestamp: 2026-07-12T04:54:00Z
  checked: private trace retention and secrecy
  found: The mode-0700 escrow contains only mode-0600 files, including duplicated pre-validation and tombstone copies with digests. No active attempt directory, lifecycle process, espflash monitor process, or serial holder remains.
  implication: The failed run is diagnosable without resuming or repeating it, and raw local identities remain outside committed evidence.

## Resolution State

root_cause: The old post-plug checkpoint raced native USB, and the first replay repair remained conditional on Stratum-session execution. Transport ownership alone could not recover early bytes or guarantee later replay.
fix: Keep the response-free exact-node watcher and private trace/validation contract; move allowlisted replay into a bounded boot-lifetime task so external-service progress cannot gate proof.
hardware_status: watcher, passive attachment, 360-second capture, and cleanup passed on `4891ce06bb51f872fd41c0baa2412cd660c877eb`; replay delivery failed with an empty cold stream. A later clean committed HEAD requires fresh hardware confirmation.
