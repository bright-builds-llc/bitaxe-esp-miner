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

### task-native-usb-boot-chain-integrity-205 | 2026-09-01 | Verify installed recovery boot bytes and OTA selection

Status: Blocked historical evidence. The consumed commands/plans below are historical, not current execution authority. ADR-0021 and `task-fixed-usb-serial-qualification` own the replacement transport/baseline; missing historical evidence is not promoted or erased.

- [x] Create the immutable read-only boot-chain discriminator plan.
- [x] Implement exact boot metadata, OTA selection, selected-app readback, and
      protected evidence.
- [x] Verify, commit/push, package, and run the single authorized Ultra 205
      readback.
- [x] Record the closed result without repairing or reopening predecessor work.

Plan: `docs/parity/work-plans/20260902T022334Z-NATIVE-USB-BOOT-CHAIN-INTEGRITY/PLAN.md`

Depends on: terminal `application_missing` under
`task-native-usb-rom-exit-discriminator-205`; immutable recovery-006 snapshot
bundle; connected Ultra 205; and existing native-USB/process/privacy owners.

Authorization: plan/source/test/docs/rules/build/package, commit/push,
effect-free preflight/finalization, one closed display-state capture, one
built-in BOOT/RESET sequence, one ROM admission, exact read-only bootloader,
partition-table, OTA-data, and selected-app flash reads, one managed hard-reset
application exit, protected evidence, and cleanup. All device writes, erase,
network, mining, hardware-control, electrical-interface, other-device,
durability, recovery mutation, and parity-promotion effects are excluded.

Verification: Plan commit `2f150921` is pushed. Exact read-only esptool range
ownership, ESP-IDF partition/OTA selection, selected-app identity validation,
closed display/manual checkpoints, consume-once evidence, and public
allowlisting are implemented with focused tests. Full gates, implementation
commit/package, and hardware readback were admitted only after verification.

Hardware result: Implementation commit `897b83cc` and its exact clean package
are pushed. The display checkpoint independently recorded `active_ui` before
reset. One built-in BOOT/RESET sequence admitted the same physical Ultra 205 in
ROM. Exact read-only bootloader, partition-table, and OTA-data reads completed,
and all three match the immutable recovery-006 snapshots byte-for-byte. Both
OTA select copies reduce to invalid, so the pinned bootloader rule selects the
factory application. The single selected factory-partition read then failed
mid-transfer at the macOS USB boundary with closed signature
`device_not_configured`; no selected-app file, machine result, public
projection, or `RESULT.md` was produced. Passive final inspection found the
same Serial/JTAG transport with unknown execution owner. No write, erase, NVS,
network, mining, or hardware-control effect occurred; no process or USB holder
remains.

Completion review: Terminal and blocked after the one authorized readback.
Recovery-006 boot metadata is exact and its factory-selection state is proved;
the remaining discriminator is the unstable long USB read or selected factory
application boundary. The readback root is consumed, another reset/read is
prohibited, and follow-up requires a new contract. Recovery-006 bytes remain
unchanged.

### task-native-usb-rom-exit-discriminator-205 | 2026-08-31 | Separate ROM exit from Serial/JTAG application ownership

Status: Blocked historical evidence. The consumed commands/plans below are historical, not current execution authority. ADR-0021 and `task-fixed-usb-serial-qualification` own the replacement transport/baseline; missing historical evidence is not promoted or erased.

- [x] Create and push the immutable ROM-exit discriminator plan before source
      changes or hardware effects.
- [x] Split transport profile from execution owner behind `UsbOwnership`.
- [x] Correct the force-download ROM exit and add boot-lifetime profile evidence.
- [x] Create the immutable passive-first/manual-fallback successor plan after
      the rebooted host proved Serial/JTAG transport without ROM admission.
- [x] Verify, commit/push, package, and run one no-write Ultra 205 discriminator.
- [x] Publish or record the closed result without reopening predecessor tasks.

Plan: `docs/parity/work-plans/20260831T190744Z-NATIVE-USB-ROM-EXIT-DISCRIMINATOR/PLAN.md`

Successor plan:
`docs/parity/work-plans/20260901T161405Z-NATIVE-USB-SERIAL-OWNER-RECOVERY/PLAN.md`.
The predecessor plan and unused `scratch/native-usb-rom-exit/attempt-001`
remain immutable historical context.

Depends on: accepted immutable `nvs_match` under
`task-native-usb-config-ap-recovery-205`; terminal repeated
`same_serial_jtag` traces; installed recovery-006; connected Ultra 205; and
the existing native-USB ownership, recovery, privacy, and process supervisors.

Authorization: repository source/test/docs/rules/build/package, commit/push,
effect-free preflight/finalization, one 25-second passive owner observation,
one read-only ROM admission probe only after silence or insufficient samples,
one conditional built-in BOOT/RESET sequence, one contained read of the exact
force-download bit, one contained esptool hard-reset application exit, at most
30 seconds of same-device passive application observation, protected evidence,
and cleanup. Firmware/NVS/settings/theme writes, flash, erase, OTA, Wi-Fi or
HTTP actions, discovery, mining, ASIC work, fan/voltage/power effects, direct
UART/pins/pads/headers/probes, other devices, durability, recovery mutation,
and parity promotion are excluded.

Verification: Plan commit `29950014` is pushed. Red-to-green coverage passes
for execution-owner separation, ROM and application admission, force-bit
parsing, hard-reset exit, periodic boot-profile replay, CLI shape, projection
allowlisting, and ownership/source guards. Ordered Cargo gates, Bright Builds,
all 74 Bazel tests, firmware build/package, native-USB ownership, parity,
progress, redaction, reference cleanliness, whitespace, sensitive-value scan,
and final diff review pass. The separate implementation commit/package,
effect-free preflight, one no-write hardware discriminator, finalization, and
cleanup remain.

Successor verification: Immutable successor plan commit `ed71ba15` is pushed.
The software implementation makes ordinary Serial/JTAG detection inspection
only, adds the sealed passive-first owner recovery Interface, binds two-sample
recovery-006 attestation admission, permits the manual branch only for missing
or insufficient samples, retains one-probe/no-repeat semantics, and projects
only closed allowlisted fields. Focused Rust, TypeScript, Swift fixture,
detector, flash, device-session, runfiles, and native-USB ownership tests pass.
The ordered Cargo gates, Bright Builds, all 75 Bazel tests, firmware
build/package, ownership verification, parity/progress, redaction, reference,
whitespace, sensitive-value scan, and final source review pass. The separate
implementation commit was admitted only after this complete gate set.

Successor hardware result: Implementation commit `9d3aec4b` and its exact clean
package are pushed. Inspection-only detection admitted one Serial/JTAG
transport without synchronization traffic, and the effect-free preflight
passed. The single 25-second passive observation reported `missing`; its one
read-only ROM probe failed without identity drift and sealed `manual_required`.
The authorized built-in BOOT/RESET checkpoint then admitted the same physical
board in ROM. The exact force-download bit was clear, the contained managed
esptool hard-reset application exit completed once, and the subsequent
30-second observation again reported `missing` on the same Serial/JTAG
transport without enumeration change. The sealed terminal category is
`application_missing`. Device writes, NVS reads, host-network effects, and
repetition remained zero; private modes, USB cleanup, zero holders, and zero
owned processes passed. No public projection or `RESULT.md` was produced.

Predecessor progress: Implementation commit `88fd860e` is pushed and its exact clean
package is built. A Mac reboot cleared the uninterruptible host-I/O child. Fresh
retain-ROM detection then found exactly one same-device Serial/JTAG transport,
but read-only bootloader synchronization failed without enumeration change.
The detector left no holder or owned process. No preflight, task root,
force-bit read, reset, monitor, projection, device write, or network effect
occurred. The successor corrects this admission inversion by authenticating the
application passively before requesting ROM proof and allows one built-in
BOOT/RESET recovery only when passive evidence is unavailable.

Predecessor checkpoint: Blocked at the shared Serial/JTAG execution-owner
boundary before hardware admission. Predecessor tasks and recovery-006 remain
unchanged.

Completion review: Terminal and blocked after the one authorized passive/manual
owner-recovery sequence. Recovery-006 remains installed, but execution owner
cannot be authenticated from its silent Serial/JTAG runtime. Configuration-AP
Stage 2, another reset, and another owner-recovery ordinal remain prohibited
until a new contract addresses the missing application evidence boundary.

### task-native-usb-display-recovery-205 | 2026-08-30 | Authenticate recovery-006 from the displayed origin

Status: Blocked historical evidence. The consumed commands/plans below are historical, not current execution authority. ADR-0021 and `task-fixed-usb-serial-qualification` own the replacement transport/baseline; missing historical evidence is not promoted or erased.

- [x] Create and push the immutable display-bound recovery plan before
      implementation or hardware/network effects.
- [x] Add the task-gated display capture, private USB MAC receipt, strict HTTP
      restoration Module, no-effect finalizer, and focused guardrails.
- [ ] Authenticate recovery-006, restore exact settings/theme, and prove
      mine-on-boot disabled, inactive zero-work/share state, and cleanup.
- [ ] Publish the redacted recovery projection, write `RESULT.md`, and archive
      only this child task.

Plan: `docs/parity/work-plans/20260830T161148Z-NATIVE-USB-DISPLAY-RECOVERY/PLAN.md`

Depends on: terminal predecessor `task-native-usb-recovery-transition-205`;
completed recovery-006 snapshot and Wi-Fi-seed receipts under
`scratch/native-usb-transition/recovery-002`; connected Ultra 205 displaying
an RFC1918 address; recovery-006 bundle/readiness/validator evidence; and the
existing strict HTTP and native-USB owners.

Authorization: plan/source/test/docs/rules/build/package, lesson audit and one
corrective lesson, commit/push, effect-free preflight/finalization, one
operator display-address capture with one pre-mutation correction branch, one
read-only USB/MAC admission, one exact settings PATCH, one exact theme POST,
bounded reconciliation reads, final detector admission, protected evidence,
and cleanup. Local development output may show the RFC1918 address; committed
evidence remains redacted. Flash, NVS write, erase, manual buttons, transition
diagnostic, mining, ASIC, fan/voltage, fault injection, OTA, discovery, mDNS,
ARP, router state, scans, direct UART/pins/pads/probes/headers, other devices,
other boards, durability, and parity promotion are excluded.

Verification: Immutable plan/audit/task commit `29a98fcc` is pushed. Focused
RFC1918, MAC normalization, restoration payload, strict HTTP route, CLI,
projection allowlist, Swift fixture/runfiles, file-mode, ownership, and source
guard tests pass. Ordered Rust gates, Bright Builds, all 72 Bazel tests, normal
and rollback firmware links, canonical package, native-USB ownership, parity
and progress, redaction, reference cleanliness, sensitive-value scan,
whitespace, and final diff review pass. The implementation commit, exact clean
package, one display-bound restoration, finalization, and cleanup remain
pending.

Terminal update: implementation commit `54020484` and nested-root fix
`cfcbfa9b` are pushed. Effect-free preflight accepted. The first capture
returned `capture_cancelled` without creating a device or network effect; the
operator then confirmed that the Bitaxe display shows no IP address. The
plan's required displayed-origin assumption is therefore false. No USB
admission, HTTP request, settings/theme mutation, final detector, or public
projection occurred.

Completion review: Blocked at `display_origin_unavailable`. Recovery-006
remains installed, but Wi-Fi association and exact settings/runtime state are
still unauthenticated. Another display capture, network discovery, or settings
request is ineligible. Follow-up work requires a separate task contract for
the recovery-006 Wi-Fi NVS/schema and runtime association boundary. The
blocked predecessor, parent native-USB task, STR-005, and BWG remain
unchanged.

### task-native-usb-config-ap-recovery-205 | 2026-08-30 | Recover through the detector-bound setup AP

Status: Blocked historical evidence. The consumed commands/plans below are historical, not current execution authority. ADR-0021 and `task-fixed-usb-serial-qualification` own the replacement transport/baseline; missing historical evidence is not promoted or erased.

- [x] Create and push the immutable configuration-AP recovery plan before
      implementation or hardware/host-network effects.
- [x] Add the exact read-only NVS admission and protected typed semantic
      discriminator; prohibit later actions until it seals `nvs_match`.
- [ ] After an accepted `nvs_match`, add detector-bound AP association, strict
      restoration, restart/resume, and host cleanup in a separate commit.
- [ ] Prove exact recovery-006 settings/theme, station recovery, inactive
      zero-work/share state, final USB admission, and cleanup.
- [ ] Publish the redacted projection, write `RESULT.md`, and archive only this
      child task.

Plan: `docs/parity/work-plans/20260831T033840Z-NATIVE-USB-CONFIG-AP-RECOVERY-NVS-FIRST/PLAN.md`

Supersedes immutable planning contract
`docs/parity/work-plans/20260830T184150Z-NATIVE-USB-CONFIG-AP-RECOVERY/PLAN.md`;
the earlier plan remains historical and unmodified.

Depends on: blocked display-origin and transition recovery tasks; completed
recovery-006 snapshot/Wi-Fi-seed receipts; visible USB-derived `Bitaxe_ABCD`
configuration AP; connected Ultra 205; protected Wi-Fi/pool/settings inputs;
and the existing native-USB, strict HTTP, and CoreWLAN owners.

Authorization: plan/source/test/docs/rules/build/package, commit/push,
effect-free preflight/finalization, one exact NVS readback, one directed scan
and association to the USB-derived AP, one settings PATCH, one theme POST, one
software restart, one host Wi-Fi restoration, bounded reconciliation reads,
one no-timeout station-IP checkpoint, final detector admission, protected
evidence, and cleanup. Firmware/NVS writes, erase, manual buttons, transition
diagnostics, mining, ASIC, fan/voltage, OTA, broad scans, ARP, mDNS, router
inspection, subnet discovery, direct UART/pins/pads/probes/headers, other
devices/boards, durability, and parity promotion are excluded.

Verification: Successor plan commit `7db48345` is pushed. Stage 1 software
implementation commit `5e75017b` and its exact clean package are pushed and
verified. The first `read-nvs` launch stopped before root creation, USB
acquisition, or NVS read because the wrapper lacked a typed child-admission
checkpoint and rejected the managed virtualenv Python symlink. The targeted
admission-only/closed-failure fix passes the real no-effect managed-tool
boundary in pushed commit `ee527b40`. Its exact clean package, fresh detector,
and strengthened preflight passed. The progress-backed read consumed exactly
24 KiB at `0x9000`, sealed `nvs_match` across all 30 expected typed entries,
performed no device write or host-network effect, returned to the admitted ROM
profile, and proved private modes and zero owned processes. Stage 2
configuration-AP recovery remains blocked because the required recovery-006
application profile did not reappear after the read. The first sealed `resume`
proved that `board-info --after hard-reset` left the same device continuously
in `SerialJtagRuntime`. A targeted successor uses the pinned esptool `run`
command to execute the installed application without repeating the NVS read or
writing the device. Pushed commit `aba74ad9`, its exact package, and all gates
passed, but the successor repeated the same authoritative signature: 132
bounded observations were all `same_serial_jtag`, with no overflow, completion
receipt, NVS reread, device write, host-network effect, or owned-process leak.
The plan's repeated-signature stop is terminal. Stage 2 association, recovery,
and finalization are prohibited until a separate contract resolves whether the
installed application can be authenticated in `SerialJtagRuntime` without
assuming Worker reappearance.

Completion review: Blocked after accepted `nvs_match` at the recovery-006
runtime-profile boundary. Recovery-006 remains installed and the protected
readback is immutable. No public projection was produced. Blocked predecessors,
parent native-USB task, STR-005, and BWG remain unchanged.

### task-parity-bap001-firmware-interface | 2026-08-20 | Implement the firmware BAP owner

- [x] Add a single-owner UART2 firmware shell for the pinned GPIO39/GPIO40
      115200/8-N-1 BAP interface.
- [x] Route bounded ingress, requests, and subscriptions through the existing
      pure BAP core with redaction-safe failure handling.
- [x] Add focused lifecycle and source-ownership coverage, build/package the
      firmware, run every gate, and transition only `BAP-001` to at most
      `implemented` with `unit,workflow` evidence.

Plan: `docs/parity/work-plans/20260821T020549Z-BAP-001/PLAN.md`

Authorization: repository source, fixture, test, documentation, build, commit,
and push work only. No detector, USB/device session, flash, monitor, accessory,
external UART, physical pins/pads/headers, credentials, network discovery,
mining, ASIC traffic, hardware setting mutation, voltage/frequency/fan/power
effects, restart, fault injection, OTA, or recovery.

Evidence and status boundary: `implemented` requires the complete firmware
ownership and lifecycle surface, exact UART configuration, bounded protocol
handoff, current tests, canonical firmware build/package, and every mandatory
gate. `verified` additionally requires a separately authorized live accessory
and detector-gated named-board hardware regression with cleanup and redaction;
none are authorized or claimed here.

Verification: Implementation commit
`80f88df1799be63e3a71c291ad015f89c65cd8ae`; four focused lifecycle tests,
two source-ownership/privacy tests, all twelve BAP core tests, the canonical
six-file ESP32-S3 package, ordered Rust gates, Bright Builds, all 50 Bazel
tests, parity/progress, redaction, reference cleanliness, sensitive-value,
file-size, and diff checks passed. Evidence:
`docs/parity/evidence/bap001-firmware-interface/summary.md`.

Completion review: The software-only interface ownership and lifecycle scope
is complete and supports `implemented` with `unit,workflow` evidence. This
task remains active and unarchived because electrical UART behavior, a live
accessory request/subscription session, Wi-Fi-password delivery, setting
persistence/effects, detector-gated hardware evidence, cleanup, and redaction
remain outstanding. Transition `20260821T031800Z-BAP-001` and deterministic
progress synchronization accepted source/evidence commit
`294b5cf0a998f53ccfe5a6537ad8cf39dfcd6fff`. No hardware or external effect
occurred.

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

### task-parity-asic009-bm1368-core | 2026-08-20 | Implement the pure BM1368 protocol core

- [x] Add typed BM1368 init, work, result, register, framing, and error
      behavior under the pure ASIC crate.
- [x] Add pinned-reference provenance plus golden protocol fixtures and focused
      behavioral coverage.
- [x] Keep BM1368 firmware dispatch deferred, run every gate, and transition
      only `ASIC-009` to at most `implemented` with `unit,golden` evidence.

Plan: `docs/parity/work-plans/20260820T060848Z-ASIC-009/PLAN.md`

Authorization: local Rust/fixture/docs edits, tests, build/package, Git commit,
and push only. No credentials, protected attempt roots, detector,
USB/device/network runtime, flash, monitor, mining, restart, recovery, hardware
attempt, fault injection, external UART/BAP, pins, or electrical work.

Evidence and status boundary: `implemented` requires the complete pure protocol
surface, pinned fixture provenance, current tests, deferred dispatch, unchanged
Ultra 205 packaging, and all mandatory gates. `verified` additionally requires
a supported BM1368 board, firmware adapter, detector-gated hardware smoke/
regression, safe stop, and redaction; none are authorized or claimed here.

Verification: Implementation commit
`1dc17d9b9a8e12319b5ca01db297d6800bd38d46`; 12 focused BM1368 tests,
124 ASIC-crate tests with 1 existing ignored test, the deferred-dispatch
regression, Bazel ASIC tests, reference/package checks, ordered Rust gates, and
Bright Builds checks passed. Evidence:
`docs/parity/evidence/asic009-bm1368-core/summary.md`.

Completion review: The authorized pure software scope is complete and supports
`implemented` with `unit,golden` evidence only. Transition
`20260820T063300Z-ASIC-009` and progress sync accepted the exact plan and source
commit while leaving parity at 88/94 verified. This task remains active and
unarchived because firmware dispatch, a supported BM1368 board, detector-gated
hardware regression, safe-stop proof, and redaction are outstanding. Automatic
selection may treat ASIC-009 as hardware-blocked until those prerequisites
exist and continue to the next software-actionable row.

### task-parity-asic010-bm1397-core | 2026-08-20 | Implement the pure BM1397 protocol core

- [x] Add typed BM1397 init/frequency, work, result, framing, and error
      behavior under the pure ASIC crate.
- [x] Add pinned-reference provenance plus golden one/four-midstate protocol
      fixtures and focused behavioral coverage.
- [x] Keep BM1397 firmware dispatch deferred, run every gate, and transition
      only `ASIC-010` to at most `implemented` with `unit,golden` evidence.

Plan: `docs/parity/work-plans/20260820T064119Z-ASIC-010/PLAN.md`

Authorization: local Rust/fixture/docs edits, deterministic tests,
build/package, Git commit, and push only. No credentials, protected attempt
roots, detector, USB/device/network runtime, flash, monitor, mining, restart,
recovery, hardware attempt, fault injection, external UART/BAP, pins, or
electrical work.

Evidence and status boundary: `implemented` requires the complete pure BM1397
surface, pinned fixture provenance, current tests, deferred dispatch, unchanged
Ultra 205 packaging, and every mandatory gate. `verified` additionally requires
a supported BM1397 board, firmware adapter, detector-gated hardware regression,
safe stop, and redaction; none are authorized or claimed here.

Verification: Implementation commit
`3909a304213f81babf9d3fed38800bd2b515c0a5`; 16 focused BM1397 tests,
140 ASIC-crate tests with 1 existing ignored helper, the deferred-dispatch
regression, Bazel ASIC tests, reference/package checks, ordered Rust gates, and
Bright Builds checks passed. Evidence:
`docs/parity/evidence/asic010-bm1397-core/summary.md`.

Completion review: The authorized pure software scope is complete and supports
`implemented` with `unit,golden` evidence only. Transition
`20260820T070850Z-ASIC-010` and progress sync accepted the exact plan and source
commit while leaving parity at 88/94 verified. This task remains active and
unarchived because firmware dispatch, a supported BM1397 board, detector-gated
hardware regression, safe-stop proof, and redaction remain outstanding.
Automatic selection may treat ASIC-010 as hardware-blocked until those
prerequisites exist and continue to the next software-actionable row.

### task-ultra205-virtual-board-validation | 2026-09-30 | Full functional virtual Ultra 205 and pre-flash validation

Status: Verified software milestone; full qualification blocked by corrected-target
panic recurrence and unsupported mandatory integration coverage.
Cross-reference | 2026-10-02: archived `task-device-noise-worker-stack` resolved
run017's composed Noise boundary in emulation (campaign004) and passed a real
device Noise handshake (attempt-003). Other mandatory profiles and coverage
remain open.
Objective: reuse production control logic in deterministic host and Xtensa/QEMU
scenarios, model every Ultra 205 functional peripheral, then qualify one canonical
pre-flash gate. References: `task-str005-v2-accepted-share-probe`, its immutable
status001/recovery006 results, and the retained installed ce8f/453d package.

- [ ] Extract shared host-buildable runtime interfaces and production logic;
  preserve physical policies, existing host regressions and historical readers.
- [ ] Implement seeded virtual BM1366, sensor/fan/voltage/ADC/display/input,
  persistence/reset/core, networking and control-transport models with explicit
  unsupported outcomes and independent model-conformance evidence.
- [ ] Add capability-aware memory/fragmentation and named-phase allocation faults,
  including observed3451-free/2176-largest and8192-internal allocation profiles.
- [ ] Exercise composed startup/Start/status/Stop/recovery with real protocol,
  admission/accounting and local fixture; preserve earliest typed failure and
  separate qualification, historical resources and current safety.
- [ ] Integrate pinned SDK5.5.4 QEMU esp_develop_9.2.2_20250817, managed checksum
  bootstrap/doctor and an unmistakable virtual dual-core S3/16MB/8MB-OPI target.
  Physical flash must reject virtual artifacts; report paired configuration diffs.
- [ ] Provide Bazel-backed virtual-board, qualify-virtual-board and preflash-validate
  commands, versioned source/package/config/model/emulator/validator-bound results.
- [ ] Qualify complete healthy lifecycle, all mandatory faults, target panic/core
  decoding, resource/ownership tests, independent goldens/mutations, repeatability
  and host/QEMU semantic agreement. Unsupported required coverage blocks qualification.
- [ ] After baseline qualification only, enforce centrally selected local profiles
  for affected flash/update/live commands before discovery or sensitive inputs.
  Emergency Stop/Close/failure-only recovery remain independently available.
- [ ] Run affected Bazel suites, ordered Cargo fmt/Clippy/build/tests, production-ELF
  audits, standards/reference/redaction/Markdown/diff checks. Commit and push verified
  milestones with truthful remaining gaps; archive this task only when all pass.

Execution contract: local software/test artifacts and synthetic virtual effects only.
No physical device detection, USB access, real signer/pool credentials, hardware
flashing, physical mining/grants, ROM operations or physical faults. Synthetic
signed frames are permitted only inside isolated virtual scenarios. All existing device-effect
flags remain disabled. Virtual builds are separately identified and cannot become
flashable physical candidates. Synthetic NVS/identities/trust and bounded private
local evidence only; no modification of sealed predecessor inventories or raw dumps.
Pinned upstream reference remains read-only. Functional models are not electrical,
RF, cycle-accurate or physical USB timing proof. Keep calibration gaps and modeled
memory distinct from actual SDK observations; no hardware parity promotion.
Parity remains90/95; accepted-share stays active/unresolved. Changed SDK/model/source/
package/validator identities invalidate affected results. Unknown changes select the
full validation suite. Missing required profile, calibration or process-release proof
is a concrete blocker, never success or permission for another hardware attempt.

Local milestone review | 2026-09-30: runtime/model/command foundations implemented.
Initial ordered Cargo checks and 17 affected Bazel targets pass; the production
package builds. Host step-5 command passes; healthy host deliberately returns
unsupported for missing encrypted share integration. No device access occurred.
Full qualification and mandatory rollout remain unresolved; do not archive.

QEMU diagnostic continuation | 2026-09-30: pinned virtual guest component probes
prove boot, 8192-byte internal allocation/release, task join, NVS reset increment
and deliberate panic/core decoding. Healthy composed execution instead faults in
SDK TLSF during possession canonical serialization. Splitting the guest UART
reader reduces the dispatch frame from4800 to832 bytes with the same16384-byte
main stack, but the healthy failure recurs. This is virtual-target evidence;
physical status001's cause remains unproved. Preserve every prior private run.

Bounded next diagnostic contract: one instrumented synthetic healthy QEMU run
only, paired exact virtual ELF/source/config and pinned emulator; allocation-free
heap-integrity and task-stack observations before controller/possession boundaries,
plus an offline native call-frame audit. Allowed: local compilation, virtual flash
copies, GDB on the owned emulator and private logs/core inspection. Prohibited:
physical detection/USB/network access, signer credentials, flashing, mining,
grants, partition clearing, changed stack/safety limits and blind retries. Bound
emulator runtime to30 seconds and debugger output/time; always release/reap the
owned process group and retain partial evidence if core capture is unavailable.
Stop this diagnostic on recurrence after a targeted correction, unproved process
release, source/digest drift or missing meaningful crash state. Broader independent
host/model integration may continue without another target retry.

QEMU diagnostic015 successor | 2026-09-30: diagnostic014 was consumed without
resuming the paused guest because the pinned debugger rejected a command. Its
partial result and released process groups remain retained. Offline preflight now
passes against exact virtual ELF
b76fdd3dd942dbebc3b47143530ae772c54b83fd35f15a7cfbf1aa6a36a7ce36,
including supported debugger syntax, exported checkpoint symbols, 384-byte record
layout and verified Xtensa argument register. Permit exactly one successor
`just virtual-emulator diagnose-heap --manifest <retained-build014>/virtual-package.json --evidence-dir <repo>/scratch/virtual-emulator/diagnose015-health`.
The prior local-only effects, 30-second emulator/25-second debugger bounds,
privacy, no limit increases, cleanup and stop conditions apply unchanged. This
examines the frozen pre-extension guest; it cannot qualify the current broader
source or establish the physical failure cause. No further checkpointed target
retry is authorized by this continuation.

Corrective local qualification continuation | 2026-09-30: diagnostic015 measured
9280 stack bytes available at the pre-controller checkpoint. Its own144-byte
frame and the32-byte observer frame end before construction, giving9456 bytes
available at the constructor callsite. The exact frozen guest's resolved required
trust-key validation path alone needs10752 bytes (before additional curve leaves):
a1296-byte deficit. This is a demonstrated virtual caller-frame insufficiency,
not proof of the physical status001 cause or a complete callgraph upper bound.

Split the shared scenario initializer from the larger Start/status execution frame
and return an owned boxed controller; preserve the16KiB configured stack, SDK
extra512 bytes, all safety/authority/accounting limits and observer order. Verify
host corpus equality and build a separately marked corrected virtual package with
fresh source/ELF/config bindings. Offline audit must show the initialization frame
and constructor path fit with the existing margin before target execution. Then
permit one fresh synthetic `healthy-lifecycle` seed1 QEMU run only, with status and
allocation probes, bounded to30 seconds, in a new private evidence root. No
checkpoint diagnostic, new limit, physical discovery/effect or old grant replay.
Stop on recurrence or missing release/proof; save a precise blocker and continue
only independent host checks. Ordinary full qualification is still not authorized
by a passing component probe; all mandatory profiles and rollout criteria remain.

Evidence review correction | 2026-09-30: require natural host completion separately
from resource release; verify scenario/seed/source/package/executable bindings,
embed compiler inputs rather than attest to a later checkout, and verify copied
frozen manifest/config digests. New host run-v3 records preserve earlier v1/v2
results. Resolve compiler-stamp build setup/API errors with focused offline checks
before broad qualification. No previous result or seal may be rewritten.

Corrected target outcome | 2026-09-30: fresh guest build017 pairs the production
ELF27594c9f92e84e907d1f08acd87ea6a1c0644168a45a83c8dcfbbeeacf282b22;
virtual ELF579fdd34473fe17245f36d7e026fbd68e1041b0a7408a1cb3f958d4b030eb98e.
Further separation retires board construction and trust-validation temporaries
before controller assembly. The ordinary wrapper-inclusive selected native path
needs10752 bytes plus992 of observed leaf/iteration allowance:11744 total, leaving
4640 against the unchanged16384-byte configuration. This is a selected lower-bound
check; the complete call closure remains unproved.

The one authorized `run017-health` target attempt then failed: boot, task join,
internal allocation/release and allocator observations passed, but an unexpected
panic/reset prevented the scenario and actual stack-margin results. Host process
group release passed. The retained core's checksum and full ELF identity verify;
SDK fake frames are absent. The meaningful fault is LoadStorePIFAddrError in SDK
find_containing_heap, reached through heap_caps_free and a Rust Vec drop during
Noise mix_hash/Initiator completion inside the authenticated Start path. The
constructor progressed; the new failure's cause and actual margin remain unproved.
Raw core/debugger/registers remain private under run017-health/decoded-core. Stop further target execution under the recurrence rule;
retain exact ELF/logs/flash and any partial core. Constructor margin alone cannot
qualify target execution or establish the remaining cause. No device access occurred.

Preflash observation outcome | 2026-09-30: `just preflash-validate` saved a sealed
working-tree observation with42 passed/21 unsupported host cases across63 rows;
all63 target corpus cases were withheld because required host integration is
unsupported. Five exact production native audits passed. The development source
was dirty, full qualification was false, no package was frozen and no flash occurred.
The original report remains immutable; unavailable comparison coverage is typed
unsupported by the successor validator, not a demonstrated semantic mismatch.

Noise diagnosis continuation | 2026-10-01 | Approved retained-core and minimal probe

- [x] Independently verify retained run017 checksum/full ELF and pinned SDK task
  ABI; inspect Vec deallocation layout, stack bounds and captured heap data in a
  new private analysis root. Preserve raw dumps and offline001 findings unchanged.
- [x] Implement a host/target Noise-only probe using the actual production
  NoiseInitiator and the existing synthetic responder/transport shape; isolate
  controller/board/work/grants. Verify authentication, frame bounds, MAC/truncation
  rejection and phase ordering locally; retain a specific red-capable signal.
- [ ] Add allocation-free phase callbacks for SDK heap integrity and actual stack
  observations; keep diagnostics local to the marked virtual target. Build and
  audit exact ELF/caller frames and the existing2KiB margin before execution.
- [ ] Publish and push the complete repo-owned command/contract and verification
  receipts before a fresh bounded emulator attempt. Use fresh private evidence,
  exact package/source/config/model/emulator/validator bindings and verified release.
- [ ] Compare the minimized target result with retained full-scenario failure;
  state ranked falsifiable hypotheses before any additional diagnostic variant.
  Apply only an evidence-backed lifetime/ownership correction, add a regression,
  audit and then recheck the original composed boundary through a separate gate.
- [ ] Run affected Bazel and ordered Cargo checks, native audits, reference,
  redaction, standards, Markdown and diff checks. Commit/push truthful outcomes;
  leave this parent task and accepted-share unresolved unless all criteria pass.

Effect policy: offline inspection and synthetic virtual effects only, no physical
USB/detection/network/device effects, no signer or real credentials, no grants,
mining, work/nonce/submission or ASIC activity in the minimal probe. The synthetic
Noise handshake is solely an in-memory byte exchange. Maintain16KiB main stack,
SDK extra512 bytes, original internal/PSRAM routing/reserve and all authority,
accounting/safety limits. No original seal repair or core clearing. Raw memory,
registers/debugger output stay in ignored0700 roots/0600 files under ADR-0030;
share only redacted findings and hashes. Parity remains90/95.

noise-probe-baseline-enabled: false

Baseline execution gate enabled for exactly one clean/pushed Noise-only baseline. Admission requires a fresh exact
marked package, verified offline stack/resource bounds and a tested repo-owned
command. Once published, permit one Noise-only valid-handshake seed1 baseline,
QEMU at most30 seconds and any debugger at most25 seconds, single owned process
family, no automatic unchanged retry. Always collect partial facts/core on panic
and terminate/reap children/listeners before sealing. Missing checksum/ELF, stale
source, unexpected effects, invalid input/phase proof or unproved release stops
this stage. A differing or green minimal result diagnoses context dependence;
it is not full lifecycle or hardware qualification. Subsequent variants require
specific hypotheses and separately published scope, not another broad Start.

Offline revalidation | 2026-10-01: offline001 and independent offline002 both
verify checksum/full ELF. Main SP is10168 bytes below the SDK TCB base; observed
deduplicated native entry frames total25824 bytes. Fixture caller8464 and
constructor7136 overlap Noise completion6080 and outer frames. This proves a
virtual callsite stack contract violation. The96-byte Vec pointer is aligned,
in configured PSRAM and consistent across frames; its payload/header and heap
registry descriptor are absent. The direct registry node is unaligned/outside
configured memory. Earliest overwrite, double-free and physical cause are unproved.
See `docs/testing/20261001-noise-panic-diagnosis.md`; raw findings remain private.

Baseline publication | 2026-10-01: repo-owned command
`just virtual-emulator noise --manifest <fresh-current-clean-marked-package>/virtual-package.json --audit <fresh-exact-ELF-native-audit>.json --evidence-dir <repo>/scratch/virtual-noise-diagnostic/baseline001 --seed 1 --mode valid`.
One-use claim: ignored `scratch/virtual-noise-diagnostic/baseline001.claim.json`,
created exclusively before effect and never reset/reused. Only this command/seed/
mode is enabled. Require clean pushed code, package source equal to current HEAD,
source_dirty false, full ELF/image/config hashes, current compiled-source closure,
fixed16KiB stack/SDK routing, live current auditor+objdump replay and every required
selected native path within14336 bytes before admission. No changing declared
artifact identities to reuse a stale report. Retain all failed/unused prior roots.

Offline build001 observed Noise-only completion path12704, frame path7696 against
14336 available. Twelve native audit and23 runner regressions pass; seven actual
Noise host regressions pass. Broader crypto/indirect-call closure remains unproved,
so baseline must additionally prove all14 phase checkpoints, heap integrity,
configured stack observations and actual SDK low-water at least2048. The dirty
build001 is diagnostic development evidence, not the enabled execution package;
a fresh clean marked build and exact audit are required after this publication.
Result must bind validator/source/package/auditor/emulator and verify process/FD
release before finalization; failures/partial core survive independently. A
collection cutoff never becomes fixture natural completion. No full lifecycle,
physical diagnosis or parity qualification is implied by Noise-only success.

Baseline001 result and re-plan | 2026-10-01: the one-use claim was consumed at
pushed df639c99, clean virtual ELF80875ff9aaef090bc3f9016693c60de2fc99161771e66f9c87c2959b7750a94d,
compiled sourcebfeff883011f1d720af6c4deb018c61f7ce465f086fe8a71711e5b1131d97d62.
The minimal target panicked before a result; process/FD release passed, the virtual
core partition was empty. The first LoadStorePIFAddrError is in TLSF integrity
inspection; the second LoadProhibited is in panic/core-storage preparation, not
another baseline invocation. Preserve both partial roots and the consumed claim.

Correction from exact native caller instructions: optimized DWARF line attribution
suggested BeforeCompletion, but the actual call loads phase111AfterCompletion,
then advances to112. Valid certificate verification had returned before the
failing heap check. Do not infer that completion was never entered. The valid
certificate/strauss/odd-table path requires17264 bytes before additional branches,
exceeding even nominal16896 allocation. The old selected mix_hash fit receipt
omitted this required valid branch and cannot admit another full Noise run. No
current task bounds/checkpoint RAM were retained, so first overwrite is still
unseen; retain the distinction from the earlier run017 proved stack overrun.

Staged emulator bisection | 2026-10-01 | Explicit owner strategy change

- [x] Implement typed runtime-prefix diagnostics with checkpoint, boundary-reached
  and released conclusions. Prefixes101..110 stop before the known oversized
  completion branch and must never credit full authentication/frame/mining success.
- [x] Preserve one compiled campaign image/seed/config for runtime-prefix comparisons;
  record exact native caller frames and their delta from baseline6368. The new
  prefix helper is a separately compiled artifact, not an unchanged baseline
  frame. Compile-time code removal is a separate
  corroboration artifact with changed frame measurements, never automatic fix proof.
- [x] Strengthen native audit to bind the exact selected cutoff and constructor/act-two
  crypto-family paths. Report unresolved external and indirect edges explicitly;
  require independent live bounds and heap facts instead of claiming an upper bound. Full-completion admission must also include
  certificate verification and nested crypto, blocking the known17264-byte path.
- [x] Add a private debugger owner that pauses at reached-prefix, completed release
  or first panic/assert before secondary panic capture. Save bounded phase RAM,
  task stack bounds/SP/high-water and heap-integrity facts without invoking target
  allocation functions from GDB. Verify installed debugger grammar offline first.
- [ ] Publish/push command, tests and one-use per-cutoff inventory before effects;
  binary-search the earliest bad prefix with fresh roots and release every owner.
- [ ] Apply only a measured lifetime/ownership correction and real-boundary
  regression, rebuild/audit, then independently admit the corrected full boundary.
  Never raise stack/heap/safety limits or treat a shortened prefix as qualification.

noise-prefix-bisect-enabled: false

Effect contract: emulator-only prefix code bisection, no physical discovery/USB,
credentials, grants, work/mining/ASIC, original replay, erase or parity promotion.
Use same actual Noise initiator/responder/time100/lifetime3600/seed1 and pinned
SDK5.5.4/QEMU backend. Keep16KiB main/configured2KiB margin, SDK extra512 and
allocation routing/reserve unchanged. Allowed runtime cutoffs101,102,103,105,106,
107,109,110;110 is the last pre-completion cutoff. Known bad111 is historical
feedback only and cannot execute until a separately audited correction exists.
Each cutoff consumes one exclusive claim bound to frozen source/ELF/config and
native cutoff audit; max8 distinct prefix trials, no unchanged repeat. Select
halves adaptively from recorded outcomes; distinguish boundary reached while
owners live from cleanup/release failure. Missing facts remain unknown and halt
binary-search inference rather than becoming a healthy prefix. Per trial bound
QEMU35 seconds, debugger25 seconds, output2MiB, private new0700 root/0600 files.
Always terminate/reap QEMU/GDB descendants and close listeners before finalizing.
A cutoff image/diagnostic result is explicitly partial and physical-ineligible.
Stop on source/identity drift, unexpected effects, unproved ownership release or
any requirement to enter the known over-budget branch. Further trials after a
verified boundary-changing correction use a new explicit successor inventory;
no elapsed human-response timeout or per-trial confirmation is required.

Ranked predictions after the red-capable minimal baseline: (1) overlapping virtual
caller/crypto stack frames are strongest; retiring constructor/caller temporaries
must lower measured native peak and restore phase111 integrity at unchanged
limits. (2) earlier heap damage would already fail a pre-completion prefix or
its initial checkpoint. (3) an ownership/emulator allocator defect would persist
after stack bounds/margin and healthy pre-completion heap are independently
proved; no current evidence proves double-free or a Noise algorithm fault.

Commands: build the clean campaign package with `just virtual-emulator build`
and the pairing manifest, then generate each cutoff receipt through
`bazel run //scripts:virtual_noise_prefix_audit`. Execute only
`just virtual-emulator noise-prefix --manifest scratch/virtual-noise-diagnostic/build005-clean-prefix/virtual-package.json --audit AUDIT --stop CUTOFF --seed 1 --evidence-dir ROOT`.
The exact build/audit invocations and placeholder bindings are published in
`docs/testing/20261001-noise-panic-diagnosis.md`. `AUDIT` and `ROOT` are fresh
private paths for that admitted cutoff; all artifact identities are checked
before the one-use claim and effects. Start at 106 only if its native audit
passes, otherwise select an earlier admitted prefix.

Pre-effect verification | 2026-10-02: ordered Cargo format, Clippy, all-target
build and all-feature tests pass; nine affected Bazel suites pass. Exact native
prefix audits for 101/106/110 pass their selected-path budgets. The campaign
helper frame is 2,704 bytes, 3,664 below the separate baseline helper. This is a
changed diagnostic artifact, not a corrected full-handshake claim. The retained
110 crypto-family ECDH path is 13,600 bytes; unresolved external/indirect paths
remain explicit and live margin/heap proof is mandatory. Offline GDB SDK ABI and
command grammar pass against full ELF
`1ca18eea8596e0f4eb33c5e74e5db8c065138cd71231f2bd06d97cf1468aeaa9`.
Compiler-folded marker aliases now require mutually exclusive flag/argument
conditions. Missing Rust DWARF object types are verified from exact ELF symbol
sizes; no live function calls supply those facts. Standards, reference,
redaction, scoped Markdown and diff checks pass. One debugger subagent stopped
with a provider cybersecurity-content flag; its completed evidence was preserved,
and local artifact validation resolved the independent preflight type failure.
No prefix execution occurred before publication. Rebuild from this clean pushed
source and recheck exact package/audit/debugger identities before claiming 106.

Campaign001 result | 2026-10-02: published at 8817a8a3 and consumed cutoff106.
Clean virtual ELF `0d4d9400391793fa4285cfc2db4edde3d8cf3baaed814216efb4a23a52b3c5d0`
completed all six guest prefix checks with no observed failure, but debugger
collection timed out and supplied no independent snapshots. Prefix and cleanup
snapshot conclusions remain unknown. QEMU/GDB process groups, log descriptors
and the loopback listener were released. The one-use claim and all evidence are
immutable. Native breakpoint placement was at the hook entry before register
window setup; this is a diagnostic collection gap, not proof of a target failure.
Do not repeat or promote campaign001.

Checkpoint-only continuation | 2026-10-02 | Explicit owner constraint

- [x] Replace active prefix execution with ordinary application checkpoint
  collection. No debugger attachment, memory capture, core acquisition,
  extraction, inspection, decoding or examination of panic output.
- [x] Project only complete `VIRTUAL_U205` application records into retained
  stdout; discard other stdout and stderr without interpreting it. Bound all
  raw process bytes and partial-line buffering; fail visibly on overflow.
- [x] Keep source/ELF/config/native-audit bindings, existing stack/heap limits,
  one-use cutoff claims, unconditional host cleanup and subset-only results.
  Separate guest facts from unsupported independent task-bound observations.
- [x] Verify real process projection and release plus affected scenario/CLI
  suites; run ordered Cargo, standards/reference/redaction/Markdown/diff checks.
- [x] Publish and push this successor contract before its first trial; collect
  106 then select the remaining half using measured application records. Missing
  completion/checkpoint output stops diagnosis without reading crash material.

noise-prefix-telemetry-enabled: false

Successor effect contract: campaign002 replaces debugger-driven collection only.
Use `just virtual-emulator noise-prefix --manifest scratch/virtual-noise-diagnostic/build006-clean-telemetry/virtual-package.json --audit AUDIT --stop CUTOFF --seed 1 --evidence-dir ROOT`.
Build/package from the clean pushed source and generate each cutoff audit with
`bazel run //scripts:virtual_noise_prefix_audit`; exact manifest identities and
all selected-path limits must pass before claiming an effect. `AUDIT` and `ROOT`
are fresh ignored private paths for each admitted cutoff. Keep SDK5.5.4, pinned
QEMU, the same synthetic credentials/time/seed, main16KiB, margin2KiB and existing
memory routing/reserves. Only101,102,103,105,106,107,109,110 are allowed; no full
certificate completion, Start, grants, fixture/mining/ASIC, physical discovery,
USB, flashing, restart, real credentials or original replay. Campaign002 permits
one newly instrumented106 and at most eight distinct cutoffs total, each bound
to one fixed image/source/config/validator and an exclusive durable claim.
Campaign001 claims are never cleared or reused. QEMU35seconds, total output2MiB,
partial line64KiB; terminate/reap every descendant and release descriptors before
finalization. Retain only ordinary structured application facts and host release
records. Missing, invalid or unsafe checkpoint facts stop further inference;
unknown is never a healthy prefix. No core partition or debugger material is
read after execution. Independent live task-bound proof remains unsupported and
prevents full qualification; guest checks alone may support this explicitly
partial diagnostic comparison. Seals stay immutable, accepted-share stays
unresolved, physical-effect gates stay disabled and parity remains90/95.

Pre-effect review: real synthetic process projection/release regressions pass
35/35, ordinary telemetry runner tests pass8/8, and six affected Bazel suites
pass. Ordered Cargo format/Clippy/build/tests pass. Standards, pinned-reference,
redaction, scoped Markdown and diff checks pass. The new runner is smaller,
removes the debugger dependency from active collection, and never reads a
post-run partition. Explicit v2 result semantics preserve partial guest evidence
without claiming independent task bounds or full qualification. New record
parsing rejects malformed/non-object output before inference. Current commands
and coverage limitations are in
`docs/testing/20261002-checkpoint-only-bisection.md`. Publish this reviewed source
before the first campaign002 effect and keep earlier outcomes immutable.

Campaign002 measured outcome | 2026-10-02: source17461e37, clean virtual ELF
`6d07737269d24d3749adad65150d8285a2d7ce948469a318965f23236f9e758b`,
SDK configuration`cbe2ab13dc61885a77a0052ebee912ce0494393c41d0c3390cae293dbaabd6e1`.
Both106 and110 pass all six application prefix checks with healthy recorded heap
integrity and post-drop release. Minimum observed main free stack:106=9,672 bytes;
110=2,296 bytes, above the unchanged2,048-byte requirement. Verified host process,
descendant, descriptor and writer release passed for both. Target stderr files
are empty; only projected ordinary records were retained (2,342/3,174 bytes).
No debugger, memory/core acquisition, decoding, post-run partition inspection or
examination of panic data was used. Claims and records remain immutable. Close
campaign002 rather than spending further unchanged cutoffs. These are diagnostic
subset passes with independent task bounds unsupported and full qualification
false. They narrow the next source investigation to completion/live caller-frame
overlap; changed prefix frames prevent claiming an unchanged-baseline bisection
or a demonstrated exact failing statement.

Completion lifetime correction | 2026-10-02 | Source and build artifacts only

- [x] Extract preparation into a non-inlined helper whose native frame ends
  before certificate completion. Keep actual codecs, trust, deterministic inputs,
  clock and existing failure categories/phase ordering unchanged.
- [x] Use fallibly reserved owner storage and the existing production completion
  seam to reduce result-construction scratch; retain unconditional post-owner
  release observation on every error. Add a meaningful reservation failure test.
- [x] Verify real host handshake/frame regressions, independent trust rejection,
  exact target native paths through construction, ECDH/signing, valid certificate
  verification and nested crypto, serialization and cleanup. Report unresolved
  paths explicitly; never increase stack, heap, authority or safety limits.
- [ ] Keep every execution gate disabled while building/auditing the correction.
  Publish a separate checkpoint-only full-boundary diagnostic contract only if
  its native resource admission passes. No core/panic/debugger data is permitted
  in this continuation. Missing normal records stop inference.

Current blocker: complete-boundary resource proof and a newly admitted corrected
run do not exist yet. The known original full helper remains inadmissible. This
source-only correction is a measured hypothesis, not a physical firmware repair
or virtual-board qualification. Accepted-share remains unresolved; do not archive.

Correction source verification | 2026-10-02: ordered Cargo format/Clippy/build
and all-feature tests pass (2,581 passed, 0 failed), including all 11 Noise probe
tests with real owner-reservation failure and unreserved-slot regressions. Eleven
affected Bazel suites, standards, reference, redaction and diff checks pass. No
local Markdown formatter is installed; none was added. Host behavior only: the
corrected target's native resource paths remain unaudited, so items 3 and 4 stay
open and the development build under `build007-lifetime` is not admitted.

Responder frame correction | 2026-10-02 | Dev builds only, never executed

- [x] Measure the `85999542` correction with the longest resolved crypto-family
  descent, calibrated against campaign002 cutoff110 (static ECDH 14,272 bytes vs
  runtime ~14,088 used). Completion with certificate verification fell from
  16,160 to 12,160 bytes, but responder ECDH/sign rose to 15,104 bytes, over the
  unchanged 14,336-byte budget: the first correction moved preparation over margin.
- [x] Split preparation into sibling non-inlined frames: `prepare_initiator`,
  `respond`, `construct_responder` and `step_responder`, consuming the act-two
  result in place. The responder is already boxed upstream; no new allocation.
  Phase order, seeds, trust and fault behavior are unchanged.
- [x] Replace the full-probe native auditor with v2: required paths follow the
  corrected helpers and extend each crypto boundary by its longest resolved
  descent, including a named certificate-verification path, encrypted frame and
  outcome emission. Indirect, outside-family, cycle and drop-glue edges remain
  reported, not bounded; `complete_callgraph_bound` stays false.
- [x] Build a clean package from the pushed source and bind a v2 receipt to it.

Dev measurement (`build011-step-in-place`, dirty source, not admitted): all eight
v2 paths fit. Tightest static headroom: responder ECDH/sign 13,552 bytes (784),
completion and certificate verification 12,288 bytes (2,048). Static descents are
lower bounds; only a separately published checkpoint-only run can measure runtime
margin. Gates stay disabled.

Clean v2 receipt | 2026-10-02: source `7b2fe9e6` (clean), compiled source
`88ae2a06275ec63a50663526ca1092f5423a29763a67d6c11ebebf58a2340cbf`, virtual ELF
`73b3f2ccded8958dc47ecf7325c70389a3bce5546a13231f523b470ef49f3dec`, SDK
configuration `10f5335da267cef5bdcd383671ca20a7d41b13072b96cca00aed04a24ded7bda`,
auditor `ce7fd5da2b31e05cdbad163fedd617e46277bca04bb34bc14ae803a740002385`,
receipt `b01492292e8355218a6b7236fd82a76b1d82945f11b22eeae51c04cf45515383`.
All eight paths fit, with figures identical to the dev measurement. Read-only
`admitNoisePackage` recomputes and accepts the native proof. This satisfies the
static native resource admission only. Drop glue, indirect and outside-family
edges remain unbounded, and runtime margin, heap integrity and release are
unmeasured. No emulator, debugger or hardware effect was run. The receipt lives
under ignored `scratch/virtual-noise-diagnostic/build012-clean-split`.

Next: item 4 may now publish a separate checkpoint-only full-boundary contract
for this exact package. It needs full-probe checkpoint telemetry, a one-use claim,
a fresh protected root and an explicit runtime-margin and heap verdict before any
effect gate is enabled. Until then, full qualification stays false and
accepted-share stays unresolved. Do not archive.

Full-boundary checkpoint contract (campaign003) | 2026-10-02 | Explicit owner approval

- [x] Add `noise-checkpoint`, a full-probe runner that keeps only projected
  `VIRTUAL_U205` application records. It does not use a debugger, panic text,
  core decoding or post-run partition reads. Package admission moves to
  `noise-admission.mjs`, so the runner never loads crash tooling. The validator
  identity binds every visible emulator and core-dump script.
- [x] Publish and push this contract with both gates enabled, then build the
  package from that exact commit and bind a fresh v2 receipt to it.
- [x] Run exactly one claimed execution, record the outcome, disable both gates,
  then commit and push.

noise-full-checkpoint-enabled: false

Objective: one emulated valid seed-1 handshake, certificate completion and
encrypted frame round trip on the corrected image. Judge only application
records: boot identity, authenticated round trip, owner release, 14-phase heap
integrity, and at least 2,048 bytes of free stack at every phase.

Commands, run from a clean tree equal to `origin/main`:

```sh
just virtual-emulator build --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --evidence-dir scratch/virtual-noise-diagnostic/build013-full-checkpoint
node scripts/virtual-emulator/noise-stack-audit.mjs --elf ELF --sdkconfig CONFIG --compiled-source-sha256 SHA --output scratch/virtual-noise-diagnostic/build013-full-checkpoint/full-audit-v2.json
just virtual-emulator noise-checkpoint --manifest scratch/virtual-noise-diagnostic/build013-full-checkpoint/virtual-package.json --audit scratch/virtual-noise-diagnostic/build013-full-checkpoint/full-audit-v2.json --seed 1 --evidence-dir scratch/virtual-noise-diagnostic/full-checkpoint001
```

ELF, CONFIG and SHA must come from the build013 package. The receipt must pass
before any effect. The evidence parent stays mode 0700, and the
`full-checkpoint001` child must be absent immediately before launch. Wrapper
stdout and stderr go to separate mode-0600 sibling files.

Allowed effects: one bounded emulator process with a 60-second collection
window, owned and reaped by `runPrivate`. Prohibited: hardware, USB, network,
debugger, memory or core capture, reading target stderr or panic text, partition
inspection, and raising any stack, heap or safety limit.

Retry bound: one execution. The claim
`scratch/virtual-noise-diagnostic/full-checkpoint001.claim.json` is written
exclusively immediately before the effect. A failure before that claim consumes
nothing and may be repaired and rerun with a fresh root. Once claimed, an
unchanged retry is prohibited, and any successor needs a new contract and a
verified fix.

Stop conditions: a failed check, missing or invalid records, an unproven release,
or a source or validator change. Missing normal records stop inference without
reading crash material. Evidence and privacy: raw artifacts stay under ignored
mode-0700 roots; commits record only digests, check outcomes and numeric
observations. Passing would prove only this emulated diagnostic. Independent
task bounds stay unsupported, and full-board qualification, hardware, parity
and accepted-share remain unchanged.

Campaign003 outcome | 2026-10-02: the contract was published at `a024948a`. The
package built from that commit has compiled source
`88ae2a06275ec63a50663526ca1092f5423a29763a67d6c11ebebf58a2340cbf`, virtual ELF
`1bc1cfd7c3d5828a2670b271134192fef5c97d7f1f2a6b8d4b2cd8d3326886df`, v2 receipt
`6bd3a035bc4f0335084af59f388bdb2cfe41213ad96922118f8b33bc032e8094` and
validator `b5820382ceaa02e6a7ea37066c92f4f7f08f5eeb1aa6da06cc33783d010ec52e`.
The single claimed execution passed all five checks: boot identity, an
authenticated handshake with certificate completion and a 32-byte frame round
trip, owner release, heap integrity at all 14 phases (101–114), and the
unchanged stack margin. Minimum free main stack was 3,016 bytes, first reached
at phase 109 (responder ECDH and sign), or 13,368 bytes used against the
13,552-byte static path. Completion and the frame stayed shallower. Internal
free heap ranged from 340,723 to 346,403 bytes, with a constant 188,416-byte
largest block. The emulator was released at the collection cutoff, no writer
leases remained, and target stderr was empty. The projected application records
were 3,701 bytes. Result digest:
`8447cc4e865de46429d8a0f608d887500764c2a7df1317647dda30deeaa86a61`. The claim is
consumed, so this run cannot be replayed. Both gates are disabled again.

This supports the frame-lifetime hypothesis in emulation only. The earlier
baseline's heap-integrity fault after completion is absent from the corrected
image. That does not show which statement was originally faulty, and it is not
a device repair. Independent task bounds remain unsupported, and full-board
qualification, hardware, parity (90/95) and accepted-share are unchanged.
Remaining parent-task blockers are listed under Verification below; do not
archive.

Verification: ordered Cargo format/Clippy/build/tests,21 affected Bazel targets,
55 model tests,19 encrypted-profile tests,9 scenario tests, compiler-closure
stale-source/build regressions, process/listener and evidence regressions, five
production native audits, reference/redaction/standards/Markdown/diff checks.
Final verification receipts and public summary accompany the committed milestone.
Completion review: not complete; do not archive. Explicit remaining blockers are
corrected-target panic and missing actual margin/call-closure proof, strict live
fixture/header/nonce vector, real allocation interception, production Gate/page/
HTTP/framed-I/O integration, consolidated production lifecycle/ownership profiles,
full qualified host/target corpus and mandatory matching-report rollout. Missing
calibration remains an explicit fidelity non-claim. Accepted-share stays active,
all physical-effect gates stay disabled and parity remains90/95.

## Future

### task-parity-bap-live-accessory-verification | 2026-10-06 | Verify BAP over a USB-to-UART adapter

Status: Future, owner-gated. The owner has USB-to-UART adapters for
development (recorded in `AGENTS.md` under Direct UART And Pin-Manipulation
Authorization) and asked to revisit this later. Do not select it
automatically: it starts only when the owner explicitly activates it, and the
direct-UART rule still requires fresh explicit authorization before any
connection guidance or hardware use.

Scope: promote `BAP-001` and `BAP-002` from `implemented` with live evidence
for the Ultra 205 accessory port, using a host-side adapter in place of an
accessory.

- [ ] With the owner, identify the Ultra 205 BAP connector, its pinout and
      logic level from the pinned reference and board documentation, and the
      owner's adapter model; write connection guidance for the board and the
      macOS host.
- [ ] Add a repo-owned host BAP client for request, subscribe, unsubscribe and
      set, with privacy rules for the Wi-Fi `ssid` and `password` parameters.
- [ ] Write a complete hardware contract covering electrical safety, allowed
      settings, restoration of changed settings, evidence and stop conditions.
- [ ] Run it, review independently and promote the BAP rows if the evidence
      holds.

Authorization: none until owner activation and explicit direct-UART
authorization.
Verification: pending. Completion review: pending.

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

### task-str005-noise-cooperative-cancellation | 2026-09-16 | Explore bounded and cancellable Noise cryptography

Status: Deferred by the owner; future backlog outside current parity work.
Do not automatically select this dependency-engineering work while that scope
deferral remains in force. A future scope decision is required to activate it.

Origin: `task-str005-noise-runtime-readiness` and the
[source/API readiness audit](docs/hardware/str005-noise-runtime-readiness.md).
The pinned crypto calls lack supported in-call cancellation and a maximum
ElligatorSwift search length. No five-second device overrun was measured.
The five-second cleanup guarantee is a prospective qualification requirement,
not established upstream parity behavior.

Scope: Assess a supported upstream solution first. If none suffices, evaluate
a narrow, reviewed dependency patch or maintained fork and its maintenance,
cryptographic-review and provenance costs before committing to that approach.
No fork or upstream contribution is commissioned by this backlog entry.

- [ ] Reassess available upstream APIs and the required cancellation guarantee;
      document whether a supported solution avoids a maintained fork.
- [ ] If activated, bound scalar-generation retries and make the native
      ElligatorSwift search cancellable/bounded without biased fallback keys,
      partial-success results or changed wire/authentication semantics.
- [ ] Propagate cancellation through act-one creation and act-two authentication;
      preserve first failure and prove actual worker/socket cleanup and disposal
      of owned secret buffers.
- [ ] Verify real-operation cancellation, deadline boundaries, interoperability,
      invalid inputs and failure cleanup; qualify indivisible native operation
      timing, stacks and memory alongside ordinary runtime owners.
- [ ] Review dependency source/provenance and publish exact tested pins before
      handing software readiness back to `task-str005-noise-runtime-readiness`.

Boundaries: Software investigation only after activation; no implicit dependency
upgrade, fork publication, device access, flashing, mining or parity promotion.
Do not edit downloaded registry sources. The frozen successor contract and all
historical evidence remain unchanged. If a different qualification strategy is
chosen, it requires a separate reviewed contract-amendment task. The linked v2
amendment supplies that decision for current Noise qualification; this backlog
entry itself grants no effect authority.

Verification: Pending future activation. Existing characterization tests and
readiness reports establish the gap, not a completed cancellation implementation.

Scope update | 2026-09-16: The [v2 parity-scope amendment](docs/hardware/str005-noise-parity-scope-amendment.md) removes this work
from the runtime/fixture/live readiness dependencies. Keep it deferred as a
stronger future cancellation/latency guarantee. It does not block v2 Noise or
later parity work; any future adoption needs its own verified contract.

Completion review: Pending future activation. No implementation or fork has
been commissioned.

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
