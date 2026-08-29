# Native USB ownership and buttonless flashing

- Run ID: `20260829T175047Z-NATIVE-USB-OWNERSHIP`
- Source base: `a0337f013b2b1db1956843d56ccdd2d003f493d7`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-native-usb-ownership-handoff`
- Blocks: `task-str005-noise-auth-205` recovery and
  `task-bwg007-real-worker-restoration`

## Objective

Keep the accepted TinyUSB Worker vendor-control and CDC-evidence topology while
restoring a tested buttonless path from every application USB profile to the
ESP32-S3 ROM downloader. Put profile classification, physical continuity,
handoff, flashing, runtime observation, recovery, and cleanup behind one deep
host `UsbOwnership` Module and one boot-lifetime firmware `UsbRuntime` Module.

Use one canonical firmware image. Normal safe boots expose the TinyUSB Worker
profile. Consume-once diagnostic/recovery boots and unconfirmed boot-safe
baselines retain USB-Serial-JTAG. OTA remains optional and cannot replace the
ROM recovery path.

This work does not promote STR-005, BWG, or any parity row. Mining, ASIC work,
fan/voltage effects, external pools, direct UART, headers, pins, probes,
soldering, test points, eFuses, fault injection, erase, and unrelated settings
changes remain excluded.

## Host Interface and profiles

Preserve the human Interface `just detect-ultra205`, `just flash`,
`just monitor`, `just flash-monitor`, and existing recovery commands. Internally
route them through one `run_usb_operation(request) -> outcome` Interface with
`Inspect`, `Flash`, `Observe`, and `Recover` intents.

Model `WorkerRuntime`, `SerialJtagRuntime`, and `RomDownloader` profiles.
Separate stable physical identity from profile identity and enumeration
identity. Retain one macOS host lease across every runtime/disappearance/ROM
transition. Runtime inspection may accept a known application profile without
pretending `board-info` is available; flash and recovery require one admitted
ROM profile and successful `board-info` before any write.

`espflash` must never receive synchronization bytes on a WorkerRuntime CDC
node. Monitoring never arms handoff and selects the receive-only adapter for
the current runtime profile.

Closed failures include `runtime_profile_unknown`, `handoff_unsupported`,
`handoff_rejected_unsafe_state`, `handoff_ready_timeout`,
`handoff_transition_timeout`, `bootloader_ambiguous`,
`physical_identity_drift`, `bootloader_sync_failed`,
`application_reappearance_timeout`, `recovery_required`, and the established
flash/cleanup categories. Preserve earliest-failure precedence.

## Firmware maintenance handoff

Extract TinyUSB installation and descriptor ownership into `UsbRuntime`; BWG
Worker remains an Adapter behind that seam. Preserve ADR-0018's rule that CDC
data input never becomes a command. Maintenance uses CDC class-control events:

1. Exact 1200-baud line coding with DTR asserted arms maintenance.
2. TinyUSB callbacks enqueue events and perform no reset or safe-stop work.
3. The Rust owner rejects active Worker effects, completes Production Mining
   Session safe stop, and emits a fixed CDC readiness receipt.
4. The host observes readiness and clears DTR.
5. The owner disarms Worker ingress, switches the internal PHY to
   USB-Serial-JTAG, registers the force-download shutdown handler, and restarts
   into ROM.

Wrong ordering, duplicate events, timeout, disconnect, active effects, or
incomplete safe stop disarm without reboot. Port only the minimum handoff logic
from Espressif's maintained implementation with source breadcrumbs. Replay
only redaction-safe boot/build/runtime markers through TinyUSB CDC; boot-time
diagnostics remain on USB-Serial-JTAG.

After handoff the host waits for the same physical connector to appear as
RomDownloader, runs `espflash --before no-reset-no-sync`, and reacquires the
expected runtime profile.

## One-time bootstrap and recovery

The user explicitly authorizes one manual recovery using only the board's
built-in BOOT and RESET buttons. Before hardware, commit/push the exact plan and
implementation, build the canonical package, create a protected bootstrap
intent, and release every host resource. Then instruct the user without a
human-response timeout to hold BOOT, press/release RESET, and release BOOT.

Fresh profile-aware detection must find one RomDownloader and pass board-info.
Flash the exact handoff-enabled package, prove WorkerRuntime, exercise automatic
handoff without buttons, then run the existing STR-005 recovery-001 contract
through UsbOwnership. Acceptance requires exact recovery-006 identity/settings,
`mineonboot=false`, inactive zero-work/share state, USB cleanup, and zero owned
processes. Diagnostic-001 is never rerun.

Before declaring routine flashing buttonless, run
`just verify-native-usb-ownership` for 20 automatic
runtime-to-ROM-to-flash-to-runtime cycles and stop at the first repeated or
identity-drift signature.

## Guardrails and verification

Add ADR-0020 and `docs/hardware/native-usb-ownership.md`; link them from
ADRs 0015/0018 and update `docs/hardware/esp-device-session.md`. Add a short
always-loaded AGENTS pointer with the invariant that every canonical
application profile retains a qualified buttonless ROM path. Manual BOOT/RESET
is bootstrap or last-resort recovery, not the normal workflow.

Append one active lesson for the observed visible-CDC/non-flashable regression.
Add Bazel-backed `just verify-native-usb-ownership` to prove one TinyUSB owner,
diagnostic USB-Serial-JTAG retention, maintenance handoff, profile-aware host
routing, and documentation links.

Use red-to-green vertical slices through UsbOwnership outcomes, UsbRuntime
events, macOS USB adapters, and the guardrail command. Cover profile planning,
earliest failure, 1200-baud/DTR order, timeout, duplicate, disconnect,
active-effect/safe-stop rejection, physical/profile/enumeration joins,
ambiguous transitions, direct ROM flashing, diagnostic SerialJTAG flashing,
runtime CDC protection, fresh-process runfiles/process cleanup, and HIL.

Before each commit or hardware effect run ordered Rust formatting, strict
Clippy, all-target/all-feature build, all-feature tests, Bright Builds, all
Bazel tests, canonical package, parity/progress, redaction, reference
cleanliness, whitespace, and final diff review. Initial production support is
macOS; Linux and Windows fail closed until their adapters are qualified.
