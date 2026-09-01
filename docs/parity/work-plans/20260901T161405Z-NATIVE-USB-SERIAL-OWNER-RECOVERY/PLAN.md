# Native USB Serial/JTAG Owner Recovery

- Run ID: `20260901T161405Z-NATIVE-USB-SERIAL-OWNER-RECOVERY`
- Source base: `fc20a59ffb03eb77277093ce25c4fbf2984dc4af`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-native-usb-rom-exit-discriminator-205`
- Supersedes: `20260831T190744Z-NATIVE-USB-ROM-EXIT-DISCRIMINATOR`

## Summary

Continue `task-native-usb-rom-exit-discriminator-205`. Preserve the earlier
plan and unused `attempt-001` as historical context.

Correct detection so shared USB-Serial-JTAG is not automatically treated as
ROM. First attempt bounded receive-only application authentication. Use one
built-in BOOT/RESET sequence only if passive evidence cannot distinguish a
silent application from ROM.

## Interfaces and implementation

- Add `just native-usb-owner-recovery preflight|observe|recover|finalize`.
- Use protected root `scratch/native-usb-owner-recovery/attempt-001` and public
  projection
  `docs/parity/evidence/native-usb-owner-recovery/owner-projection-001.json`.
- Use a sealed state machine:
  `prepared -> passive_observation -> application_authenticated | rom_admitted | manual_required -> application_exit_sent -> application_authenticated -> complete`.
- Make ordinary `just detect-ultra205` inspection-only for Serial/JTAG:
  - report `serial_jtag_runtime`, `execution_owner=unknown`, and
    `rom_admitted=false`;
  - do not send bootloader synchronization traffic;
  - retain `--retain-rom` as the explicit board-info-required ROM admission
    mode.
- `observe` retains the physical-device lease and:
  - accepts the Worker descriptor as application ownership;
  - observes Serial/JTAG receive-only for 25 seconds;
  - requires two consistent recovery-006 runtime attestations with the same
    session/ordinal, increasing uptime, and exact source/reference/ELF
    identity;
  - after silence or insufficient samples, performs one read-only
    `board-info --before no-reset --after no-reset`;
  - seals `rom_admitted` on success or `manual_required` when synchronization
    fails without identity/enumeration drift.
- Malformed, inconsistent, mixed-session, or package-mismatched markers are
  terminal and cannot be bypassed with a reset.
- `recover`:
  - from `rom_admitted`, proceeds without human action;
  - from `manual_required`, displays a no-timeout instruction to hold BOOT,
    press and release RESET, then release BOOT;
  - reacquires the same physical connector and requires one admitted ESP32-S3
    ROM downloader;
  - reads the exact force-download bit once and accepts either value because
    manual strapping does not imply that bit is set;
  - runs contained esptool with
    `--before no_reset --after hard_reset --no-stub run`;
  - observes for 30 seconds and requires exact application evidence.
- No branch may repeat observation, ROM admission, manual reset, force-bit
  read, or application exit.

## Evidence and hardware sequence

- Public evidence contains only safe digests, initial transport,
  passive-marker category, execution owner, ROM-entry path (`none`,
  `already_rom`, or `manual_boot_reset`), force-bit category, reset Adapter,
  bounded counts/timings, physical-identity match, no-write/no-network facts,
  cleanup, terminal category, and redaction status.
- Raw serial, USB identity, ports, board information, register data, process
  data, timestamps, and device identifiers remain mode `0600` under a mode
  `0700` root.
- After separate verified plan and implementation commits are pushed:
  1. Rebuild and verify the exact clean package without flashing it.
  2. Run inspection-only detection and effect-free preflight.
  3. Consume `observe` once.
  4. If it authenticates recovery-006, skip all button/reset work.
  5. If it returns `manual_required`, ask the user for the built-in button
     sequence and then consume `recover` once.
  6. Run effect-free `finalize`, verify cleanup and zero owned processes,
     write `RESULT.md`, and archive only the ROM-exit discriminator task.
- Exact application authentication makes configuration-AP Stage 2 eligible
  under its existing separate task. AP association, settings restoration,
  networking, and restart remain excluded here.

## Tests and verification

- Test inspection-only detection, explicit ROM admission, and prevention of
  bootloader traffic during ordinary Serial/JTAG inspection.
- Test two-sample attestation admission, silence, insufficient samples,
  malformed data, mixed sessions, non-monotonic uptime, identity mismatch, and
  Worker descriptor admission.
- Test state non-repetition, manual eligibility only after silence/failed ROM
  probe, same-device reacquisition, ROM ambiguity, force-bit categories, exact
  esptool exit command, and terminal precedence.
- Test no-timeout button checkpoint, cancellation, process cleanup, protected
  modes, runfiles, and explicit public allowlisting.
- Before each commit or hardware action, run ordered Cargo gates, Bright
  Builds, all Bazel tests, firmware build/package, native-USB ownership
  verification, parity/progress, redaction, reference cleanliness, whitespace,
  sensitive-value scan, and final diff review.

## Assumptions

- Recovery-006 remains installed and its accepted `nvs_match` state remains
  immutable.
- Recovery-006 emits runtime attestations every ten seconds when its
  application is executing and ready.
- Manual BOOT/RESET is authorized only as the conditional last-resort branch.
- Firmware/NVS writes, flash, erase, OTA, network access, mining, hardware
  control, electrical interfaces, durability, and parity promotion remain
  prohibited.
- Repo-local native-USB guidance and Bright Builds architecture, verification,
  testing, and Rust standards govern implementation.
