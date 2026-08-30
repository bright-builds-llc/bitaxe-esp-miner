# Native USB recovery and single-transition diagnostic

- Run ID: `20260830T142327Z-NATIVE-USB-RECOVERY-TRANSITION`
- Source base: `12017547a669126e8faeec4ad026137170a1ffeb`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-native-usb-recovery-transition-205`
- Parent blocker: `task-native-usb-ownership-handoff`

## Objective

Restore the connected Ultra 205 to recovery-006 before any diagnostic work,
then run exactly one no-write Worker-to-ROM-to-Worker transition using the
acknowledged native-USB handoff diagnostics. The run must distinguish commit
delivery, physical/profile observation, ROM admission, application
reappearance, restoration, and cleanup without treating a flash write as
transition proof.

This child does not resume the 20-cycle durability campaign and does not close
the parent task. Mining, ASIC work, fan or voltage effects, external pools,
direct UART, pins, pads, probes, headers, jumpers, soldering, test points,
eFuses, fault injection, erase, arbitrary writes, OTA-only recovery, other
boards, other devices, and parity promotion remain excluded.

## Interfaces

Add these repo-owned human Interfaces:

- `just native-usb-transition-recovery preflight|start|finalize`;
- `just verify-native-usb-transition`.

Callers provide the immutable plan, exact clean package manifest,
recovery-006 restore bundle, Wi-Fi credentials, fresh ordinal, protected root,
and redaction flag. They never select USB personalities, reset strategy,
subprocesses, physical-identity fields, or raw evidence schemas.

The recovery Interface owns task-local roots beneath
`scratch/native-usb-transition`:

- primary recovery: `recovery-002`;
- conditional diagnostic bootstrap: `bootstrap-001`;
- single transition: `diagnostic-001`;
- contingency recovery: `recovery-003`.

The public result path is
`docs/parity/evidence/native-usb-transition/transition-projection-001.json`.
Private roots are mode `0700`; sensitive files are mode `0600`. The committed
recovery readiness projection remains public mode `0644`.

## Recovery-first contract

`preflight` is effect-free and creates no task-local private root. It must
validate:

- clean, pushed, exact source and reference identity;
- the immutable plan digest and active task binding;
- the canonical manifest and every artifact digest;
- recovery-006 bundle, source lineage, validator receipt, and public readiness
  projection;
- mode and non-symlink requirements for credentials, bundle, receipts,
  managed NVS Python, and contained managed `esptool.py`;
- exact `restore-installed --admission-only` acceptance for action
  `native_usb_recovery`, ordinal `2`, plan, bundle, authorization, and root;
- no existing task-local primary recovery root and zero repo-owned USB child
  processes.

After preflight passes, the user may perform one built-in-button recovery with
no human-response timeout: hold BOOT, press and release RESET, then release
BOOT. Direct UART and every pin/pad/header path remain prohibited.

Fresh profile-aware detection must find exactly one physical Ultra 205 as
`rom_downloader` and pass `board-info`. `start` then restores recovery-006
only. It must prove exact installed identity/settings, `mineonboot=false`, an
inactive `paused` or `safe_blocked` state, zero hash rate and accepted/rejected
shares, complete USB cleanup, and zero owned processes. No diagnostic package
may be installed in the primary recovery action.

Any incomplete primary restoration runs no diagnostic action. A fresh
recovery-only root may be used only after a verified progress-changing fix;
the same authoritative restoration signature after its targeted fix is
terminal.

## No-write transition verifier

Add a `tools/flash` subcommand behind `just verify-native-usb-transition`.
It accepts board 205, one admitted Worker port, the exact package manifest, a
fresh protected root, and `--redact-evidence`. It performs no flash, NVS,
settings, network, mining, or hardware-control write.

Through the existing `UsbOwnership` Module it must:

1. acquire one retained physical-device lease in `worker_runtime`;
2. require the ready receipt;
3. clear DTR while retaining CDC and require the committed receipt;
4. record the bounded closed profile trace;
5. require the ESP32-S3 D-/D+ disconnect and observed BUS_RESET path to produce
   `serial_jtag_runtime` on the same physical connector;
6. run `espflash board-info --chip esp32s3 --non-interactive --before no-reset
   --after hard-reset` without loading or writing an image;
7. admit the board-info response as `rom_downloader` using the pre-reset
   Serial/JTAG inspection;
8. reacquire the same connector as `worker_runtime` after the hard reset;
9. close the lease and prove zero holders and owned processes.

The public projection is built from an explicit allowlist and contains only:

- source, reference, plan, evaluator, manifest, and safe artifact digests;
- `ready_received`, `committed_received`, `bus_reset_observed`;
- bounded counts for `absent`, `same_worker`, `same_serial_jtag`,
  `same_unknown`, and `physical_mismatch`;
- ROM board-info admission, application reappearance, restoration, cleanup,
  and redaction booleans;
- closed terminal category and bounded timings.

Raw ports, addresses, endpoints, credentials, USB serials, location IDs,
physical/enumeration digests, descriptors, transcripts, timestamps, device
identifiers, and board-info identity values remain protected only.

Closed failures include the existing native-USB vocabulary plus
`handoff_commit_timeout`, `bus_reset_timeout`, `same_worker_after_commit`,
`rom_admission_failed`, `application_reappearance_timeout`,
`physical_identity_drift`, `recovery_required`, and cleanup failures. Earliest
failure precedence is immutable.

## Diagnostic install and single hardware attempt

After primary recovery succeeds and the plan and implementation commits are
independently verified and pushed, build the canonical package from the exact
clean implementation commit.

Fresh detection determines the recovered profile:

- `serial_jtag_runtime` or `rom_downloader`: install the exact diagnostic
  package through normal `UsbOwnership` ROM admission without manual buttons;
- `worker_runtime`: the recovered image's handoff is not qualified, so one
  separately recorded built-in BOOT/RESET diagnostic bootstrap is authorized;
- `unknown`, zero devices, multiple devices, physical drift, failed
  `board-info`, or foreign holder: stop before installation.

After installation, prove exact-package `worker_runtime`, then run
`verify-native-usb-transition` exactly once at `diagnostic-001`. Do not write
an image during the verifier and do not run a second diagnostic ordinal.

Signature-bounded outcomes:

| Closed result | Required action |
| --- | --- |
| `handoff_commit_timeout` | Stop at CDC commit delivery; do not change PHY code |
| committed plus `same_worker` | Stop at restart/force-download ownership |
| `same_serial_jtag` plus failed board-info | Stop at ROM admission |
| `absent` | Stop at enumeration/disconnect observation |
| `physical_mismatch` | Stop at macOS physical-identity join |
| ROM admitted but Worker missing | Stop at application reappearance |
| Complete transition | Record one-cycle qualification; do not start durability |

## Final restoration and contingency

Every diagnostic branch must end at exact recovery-006. Prefer the newly
proved automatic path only after a complete transition. If the diagnostic
fails before that path is qualified, one contingency built-in BOOT/RESET entry
is authorized solely for `recovery-003`; it may restore recovery-006 and do
nothing else.

`finalize` performs no device effect. It accepts only a completed primary
recovery, at most one diagnostic result, an exact final recovery result, and
complete cleanup. It publishes a public projection only when redaction and
every identity relationship pass independent validation. A failed diagnostic
may publish its closed discriminator only after exact final restoration.

## Test and verification plan

- Red-to-green tests for parser shape, effect-free preflight, fresh roots,
  exact task/plan/package binding, recovery authorization, and no-write source
  ownership.
- Pure reducer tests for ready/committed ordering, detach-before-commit,
  committed-before-detach, BUS_RESET timeout, same-Worker observation,
  Serial/JTAG admission, physical mismatch, ROM admission, and Worker
  reappearance.
- Fresh-process macOS tests for private root modes, profile trace creation,
  process groups, holder checks, and runfiles resolution.
- Source and command tests proving the verifier contains no `write-bin`,
  `write_flash`, erase, NVS, credential-read, mining, or control path.
- Evaluator identity binds every reachable parser, projector, reducer,
  validator, launcher, and source inventory.
- Before each commit or hardware effect: ordered Cargo format, strict Clippy,
  all-target/all-feature build, all-feature tests, Bright Builds, focused USB
  tests, all Bazel tests, normal and rollback firmware links, canonical
  package, native-USB ownership, parity/progress, redaction, reference
  cleanliness, whitespace, and final diff review.

## Assumptions and standards

- Recovery-006 is the mandatory safe baseline and remains more important than
  diagnostic convenience.
- The exact implementation package supersedes `ea58797f` only as the software
  carrier for this diagnostic; it does not claim hardware qualification.
- One successful transition qualifies only the diagnostic seam, not routine
  flashing or durability.
- The design follows repo-local task, hardware, privacy, recovery, and native
  USB guidance plus Bright Builds architecture, code-shape, verification,
  testing, and Rust standards.
