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

Hardware update: Exact HEAD `e622253d2fc4aea4589e0dcf5524081b6b054aaf` passed detector, credential binding, reference, package, strict reflash/reinit heartbeat validation, response-free watcher arming, USB appearance, passive attachment, capture duration, and cleanup. The retained cold-start payload was still zero bytes, so heartbeat, boot evidence, listener evidence, and snapshots were all absent and the attempt correctly ended `blocked_safe_evidence_invalid`. The checkbox remains open; no hardware retry is authorized from this result.

## task-native-usb-late-attach-byte-delivery | 2026-07-12 14:03 | Prove application bytes after late native-USB attachment

- [x] Preserve the exact-head failed trace with mode-0700/0600 permissions and prove zero remaining processes and holders.
- [x] Separate successful firmware heartbeat production during reinit from zero-byte delivery during retained cold start.
- [x] Design and software-verify a transport-level A-B-A diagnostic that distinguishes ESP32-S3 USB Serial/JTAG late-attach behavior from `espflash` passive-reader behavior without reset, flash, or hidden recovery.
- [ ] Decide whether formal cold-start evidence needs an always-connected external UART/data-only arrangement or another machine-observable channel.
- [x] Run one-shot hardware only after the diagnostic was planned, software-verified, committed, and pushed at exact tool HEAD `a6623c8cebe54b85e4cb9e14bdcd83cd1d31b141`.
- [x] Plan and implement the next evidence seam from the one-shot result: passive `espflash` is observational, OS-native is mandatory, and formal Plan 13 requires a private exact-head qualification.
- [x] Implement schema-v2 OS-native cold qualification with owner-observed removal, resumable status, private exact-head authority, and formal Plan 13 reader selection.
- [x] Commit and push the complete software authority at `7cab0c63b9887e3670b9db20e0eaec50dc4fbf0f`, then run exactly one OS-native cold qualification after the planned both-power recovery.
- [x] Stop before Plan 13 after the qualification failed closed on the restoration identity contract; persist the failure and plan an external UART or independent data channel without retrying.

Completion review: Pending. The heartbeat feature is software-verified and hardware-observed during reinit, but it cannot close Plan 13 while the late-attached native-USB session delivers zero application bytes.

Software update: The new resumable diagnostic performs one baseline detector, same-session connected `espflash` and OS-native preflights, response-free exact-node watcher arming, and a cold `espflash` / OS-native / `espflash` sequence. Direct and forced-uncached Bazel tests cover the complete classifier matrix, real Unix-socket owner binding, private permissions, identity/epoch/ownership failures, leases, stale handles, crashes, cleanup, stream separation, and forbidden operations. The software authority is committed and pushed at `a6623c8cebe54b85e4cb9e14bdcd83cd1d31b141`.

Hardware update: Tool HEAD `a6623c8cebe54b85e4cb9e14bdcd83cd1d31b141` was clean and pushed before the single authorized attempt. The baseline detector completed, but the connected preflight failed closed: passive `espflash` delivered zero bytes and zero heartbeats, while the following read-only OS-native reader delivered 16 well-formed heartbeats from one session. The removal checkpoint and cold A-B-A sequence never ran. Private escrow is retained; terminal audit found zero diagnostic processes, sockets, and serial holders, but the selected node was absent, so accessibility and USB identity could not be re-proven. No reset, reconnect, or retry followed.

Software update: Schema v2 treats passive `espflash` as an observational control and gates on the read-only OS-native reader. A persistent lifecycle owner publishes removal only after its exact-node watcher is active, accepts the token only after owner-observed disappearance, performs OS-native as the first and only cold reader, and emits a private exact-head qualification for formal Plan 13. Ten repeated real-process runs, direct/focused suites, forced-uncached Bazel targets, reference/protected gates, shell hygiene, and the mandatory Rust sequence pass. No hardware run or closure claim has occurred.

Hardware update: Clean pushed tool HEAD `7cab0c63b9887e3670b9db20e0eaec50dc4fbf0f` completed the planned full-power recovery, baseline detector, connected controls, owner-observed removal, and bounded absence. Restoration failed closed before any cold reader opened with `appearance_identity_changed`. The identity helper hashes macOS enumeration-variant tty and IORegistry entry fields, so this result cannot prove a different physical device; it proves the current physical-identity contract is invalid across the required new enumeration. No qualification was issued and Plan 13 did not start. Terminal audit found the node present and accessible with zero holders, processes, and sockets. No post-run detector or retry occurred.
