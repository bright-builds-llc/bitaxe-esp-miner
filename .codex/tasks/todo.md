## task-plan12-contract-repair | 2026-07-11 01:00 | Repair exact-head attempt authority

- [x] Extend the closed state authority for exact Plan 13 checkpoints, effect results, and lifecycle ownership.
- [x] Repair begin/resolve/deliver/effect dispatch and terminal cleanup without real hardware backends.
- [x] Add exhaustive deterministic runner, lease/socket, secrecy, stale-handle, crash, and guard regressions.
- [x] Run the mandatory Rust sequence before every commit, then focused Node/shell/Bazel/style/diff gates.
- [x] Update the Plan 12 summary truthfully and record residual risks.

Completion review: The corrected Plan 12 software authority is committed at `287cf33bc3599d9f8afa8118037f4572cedf489f`; the exact Rust sequence and all focused runner, Bazel, Node, shell, style, reference, and diff gates pass. Residual risk is intentionally confined to Plan 13: the fixed production adapter and physical lifecycle still require detector-gated same-chain hardware evidence before any parity promotion.

## task-ultra205-serial-session-reuse | 2026-07-11 14:55 | Diagnose and repair connected-device session reuse

- [x] Tombstone the expired Plan 13 attempt before any new device command and prove zero live process or serial holder.
- [x] Add a red-capable regression for the reset-capable bare `espflash monitor --no-reset` command.
- [x] Implement the fully passive monitor contract, private structured session traces, readiness and cleanup checks, and a bounded five-cycle diagnostic command.
- [x] Run focused shell/Bazel checks and the mandatory Rust verification sequence before commits.
- [x] Pass five passive monitor reuse cycles with barrel and USB retained, followed by a successful detector check without manual recovery.
- [x] Start a fresh exact-head Plan 13 chain and record the final completion review or residual blocker.

Completion review: Durable guidance is committed at `6f0629c`; the passive session repair is committed at `b48337f`. Direct/Bazel/Plan-13/shell/reference/Rust gates pass, and the five-cycle connected-device regression plus final detector passed with zero leaked processes or holders. The fresh exact-head Plan 13 run at `4891ce06bb51f872fd41c0baa2412cd660c877eb` also proved response-free USB appearance, stable passive ownership, a full 360-second capture, and zero leaked processes or holders. It failed closed at boot evidence because replay was still coupled to Stratum-session progress; that separate firmware-lifetime defect is tracked below.

## task-plan13-boot-lifetime-replay | 2026-07-12 04:55 | Decouple cold-start proof from external services

- [x] Retain and classify the failed exact-head hardware trace without an ad hoc retry.
- [x] Move allowlisted Plan 13 replay from the Stratum socket pump to a bounded boot-lifetime firmware task.
- [x] Add regression coverage for replay ownership and the 10-second/1,880-second schedule.
- [x] Run focused shell/Bazel/reference checks and the mandatory Rust verification sequence.
- [x] Commit and push the verified firmware repair at `447f735c4df4363d84ea7b1354e32d57e28a68a5`.
- [ ] Confirm the correction with a new detector-gated Plan 13 hardware chain before closing Phase 28.1.1.

Completion review: In progress. The first watcher-based hardware run isolated the remaining failure to absent replay bytes after successful USB appearance, monitor ownership, capture duration, and cleanup. The boot-lifetime ownership regression, focused lifecycle/state/classifier/monitor suites, affected Bazel targets, canonical ESP firmware build, reference guard, and mandatory Rust sequence pass. The durable finding commit `2b504d5` and firmware repair commit `447f735c4df4363d84ea7b1354e32d57e28a68a5` are pushed on `main`. Hardware confirmation must use a later clean committed HEAD; the failed attempt will not be resumed or retried.
