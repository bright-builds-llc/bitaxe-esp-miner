---
status: fix_implemented_hardware_pending
trigger: "A freshly replugged Ultra 205 is often reachable, but later detector or monitor interactions can fail while barrel and USB power remain connected."
created: 2026-07-11T19:55:00Z
updated: 2026-07-11T20:24:12Z
---

## Current Focus

hypothesis: Confirmed in software. The retained-runtime wrapper mislabeled bare `espflash monitor --no-reset` as passive and did not prove stable USB identity, expected active ownership, or zero post-cleanup holders.
test: Direct and Bazel runs of the serial trace, monitor capture, detector, and five-cycle diagnostic test targets.
expecting: Satisfied in deterministic fixtures. Hardware must still prove five no-unplug cycles and a successful final detector on the connected Ultra 205.
next_action: Run `just diagnose-ultra205-session cycles=5 capture-seconds=30` under the parent phase workflow; do not infer physical resolution from software fixtures alone.

## Symptoms

expected: After one successful detector gate, repeated passive monitor sessions and a final detector interaction work while the Ultra 205 remains connected to barrel and USB power.
actual: A fresh physical replug often permits the first interaction, but later `board-info` or monitor interactions can fail until another physical recovery.
errors: Historical hardware runs reported `Error while connecting to device`. The latest read-only OS snapshot had a serial node, no holder, and no relevant process.
reproduction: Leave barrel and USB connected after a successful interaction, close the session, then attempt another detector or passive-monitor interaction.
started: Recurred during Phase 28.1.1 accepted-state lifecycle attempts after several process, checkpoint, and framing defects had already been repaired.

## Feedback Loop

command: `bash scripts/phase13-monitor-capture-test.sh`
red_output: The test expected `espflash monitor --chip esp32s3 --port /dev/test --non-interactive --before no-reset-no-sync --after no-reset --no-reset`, but the rendered command was the unsafe legacy form ending only in `--no-reset`.
properties: Fast, deterministic, agent-runnable, and located at the real command-rendering/execution seam.

## Ranked Hypotheses

1. **Unsafe monitor connection policy (high):** If default reset/sync/stub connection mutates the retained runtime, requiring `no-reset-no-sync` before and `no-reset` after will remove that host-induced state transition while preserving capture.
2. **Unobserved serial ownership leak (medium):** If a descendant or unrelated process retains the device, pre/post FD-holder snapshots will show a nonzero holder count or cleanup will leave a surviving process group.
3. **USB endpoint identity changed while the path appeared stable (medium):** If macOS re-enumeration races reuse, three readiness snapshots will disagree on device identity or presence before attachment.
4. **Detector connection policy differs from the intended gate (medium-low):** If detector reset semantics obscure the failing boundary, explicit `usb-reset`/`hard-reset` trace fields and categorized failures will distinguish list/open/connect/cleanup outcomes.
5. **Physical USB Serial/JTAG state requires re-enumeration despite clean host ownership (low):** If passive attachment, stable identity, and zero holders still fail, a bounded no-unplug diagnostic will retain evidence that falsifies host-leak hypotheses and isolates the device/driver boundary for hardware follow-up.

## Evidence

- timestamp: 2026-07-11T19:55:00Z
  checked: `esp-rs/espflash` v4.0.1 `connection/mod.rs`, `flasher/mod.rs`, and `cli/monitor/mod.rs`, plus the ESP-IDF ESP32-S3 USB Serial/JTAG console guide
  found: Default monitor connection drives the USB-JTAG reset strategy and synchronizes before monitoring; `NoResetNoSync` bypasses that connection work when the chip is explicit, while the monitor-level `no_reset` flag only skips `reset_after_flash`. Espressif separately documents that USB Serial/JTAG host state and physical USB re-enumeration are distinct from MCU power state.
  implication: The complete passive command contract is source-derived rather than inferred from the user-visible failure, and hardware reuse still needs its own bounded confirmation.
- timestamp: 2026-07-11T19:55:00Z
  checked: `scripts/phase13-monitor-capture.sh`
  found: Caller `--no-reset` appends only `--no-reset` to the `espflash monitor` command.
  implication: The code does not express a passive before-connect policy or an explicit after policy.
- timestamp: 2026-07-11T19:55:00Z
  checked: `bash scripts/phase13-monitor-capture-test.sh`
  found: The new exact-command regression failed in about two seconds and printed the legacy command.
  implication: A tight red-capable feedback loop exists before the production fix.
- timestamp: 2026-07-11T19:55:00Z
  checked: `scripts/process-group.sh` and existing monitor tests
  found: Process-group termination is bounded and descendant-tested, but serial FD ownership and USB identity are not observed before or after a session.
  implication: Process death alone cannot prove the serial resource is reusable.

## Working Diagnosis

root_cause: The repo labels a retained-runtime capture `--no-reset`, but that option controls only one part of `espflash` behavior. The wrapper does not opt out of pre-connect reset/synchronization, and it records neither serial-resource ownership nor USB endpoint stability. This makes a supposedly observational step capable of changing device state while leaving later failures under-instrumented.
confidence: high for the command-policy defect; hardware confirmation remains required for its contribution to the recurring physical symptom.
smallest_correct_seam: Keep the caller flag, render the full passive `espflash` policy at the monitor wrapper, and add bounded host-side readiness/cleanup traces without changing classifier state or introducing recovery actions.

## Fix and Verification

- timestamp: 2026-07-11T20:13:30Z
  checked: `scripts/phase13-monitor-capture.sh` and its direct/Bazel regression
  found: Caller `--no-reset` now renders and executes `--before no-reset-no-sync --after no-reset --no-reset`. The passive path requires three stable pre-attach snapshots, an active holder belonging to the monitor process group, and three stable zero-holder post-cleanup snapshots matching the pre-attach identity.
  implication: The retained-runtime capture is observational at the espflash connection-policy seam and fails closed on host ownership or identity ambiguity.
- timestamp: 2026-07-11T20:13:30Z
  checked: `scripts/serial-session-trace.sh` and `serial-session-trace-test.sh`
  found: Detailed JSONL traces are mode 0600 under mode 0700 ignored roots and include UTC/monotonic time, reset policy, stable device/USB identity, holder PIDs, PID/PGID/group counts, cleanup policy, exit state, and readiness classifications. Darwin identity hashes only stable registry/callout/dialin/TTY fields and rejects unmatched ioreg output.
  implication: Future failures retain enough local evidence to distinguish stale ownership, re-enumeration, cleanup, and device/driver boundaries without promoting raw identities.
- timestamp: 2026-07-11T20:13:30Z
  checked: `scripts/detect-ultra205.sh` and detector fixtures
  found: Board-info states `usb-reset` before and `hard-reset` after explicitly; list, missing-node, ambiguous-port, open, connection, and generic board-info failures have separate categories and local trace digests.
  implication: Detector reset semantics and failure boundaries are no longer implicit.
- timestamp: 2026-07-11T20:13:30Z
  checked: `scripts/diagnose-ultra205-session.sh`, its safety fixtures, and the Just/Bazel surface
  found: The default command performs one baseline detector, five bounded passive capture/cleanup cycles, and one final same-port detector; it stops on the first failure. Tests prove no flash, erase, factory-reset, credential, network-discovery, or raw-write command is invoked and the summary exposes only categories, counts, booleans, durations, and trace completeness.
  implication: A bounded agent-runnable hardware feedback loop now exists without adding hidden recovery behavior.
- timestamp: 2026-07-11T20:13:30Z
  checked: focused verification
  found: `bash -n`, four direct test scripts, `shfmt -d`, warning-or-higher `shellcheck`, Just dry-run rendering, and four affected Bazel targets all pass.
  implication: The software fix and its regression surface are internally consistent; physical confirmation is the remaining acceptance gate.
- timestamp: 2026-07-11T20:24:12Z
  checked: main-agent simplification/review pass, complete Plan 13 safety matrix, reference guard, and both mandatory pre-commit Rust sequences
  found: Unavailable Darwin/Linux USB identity now returns a failure instead of hashing empty or placeholder input; the summary records that physical intervention was not requested rather than claiming it was electronically observed; five monitor digests roll into one category-safe trace-set digest. All 84 invalid exact-head cases, adapter/state/classifier/Phase-30 tests, `cargo fmt`, strict Clippy, all-target build, and all-feature tests pass.
  implication: The implementation is fail-closed, evidence-honest, and ready for a committed exact HEAD before real hardware confirmation.

## Resolution

root_cause: The Phase 13 wrapper treated bare `espflash monitor --no-reset` as passive even though it left espflash's pre-connect reset/synchronization policy at its default. It also equated process cleanup with serial-resource cleanup and recorded no stable USB-session identity or expected active-holder proof. This combination could perturb retained runtime state and made later connection failures impossible to classify precisely.
confidence: high for the software defect and repair; pending for how much of the recurring hardware symptom it explains.
fix: Preserve the caller flag but translate it to the complete passive espflash policy. Add fail-closed pre/active/post ownership and identity gates, private detailed traces, explicit detector reset/failure semantics, and a bounded five-cycle no-unplug diagnostic command.
hardware_status: pending; no device command was run in this debug implementation turn.
