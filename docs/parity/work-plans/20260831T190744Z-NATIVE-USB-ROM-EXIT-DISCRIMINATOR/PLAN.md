# Native USB ROM-exit discriminator and execution-owner proof

- Run ID: `20260831T190744Z-NATIVE-USB-ROM-EXIT-DISCRIMINATOR`
- Source base: `c02798ce70a830089137a222bcb5fa9dc99277cd`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-native-usb-rom-exit-discriminator-205`
- Predecessor: `task-native-usb-config-ap-recovery-205`

## Objective

Resolve the terminal ambiguity between ESP32-S3 ROM download mode and an
application intentionally retaining USB-Serial-JTAG. Correct the host ROM-exit
strategy, model transport profile separately from execution owner, add
boot-lifetime application/profile evidence, and run one no-write discriminator
against the installed recovery-006 image.

This child supersedes no historical plan and does not reopen either predecessor.
It consumes the predecessor's immutable accepted `nvs_match` only as admission
evidence. Configuration-AP association, settings restoration, and all Stage 2
work remain prohibited until this child proves application ownership.

## Interfaces and model

- Add `just native-usb-rom-exit preflight|start|finalize`.
- Use protected root `scratch/native-usb-rom-exit/attempt-001` and public
  projection
  `docs/parity/evidence/native-usb-rom-exit/rom-exit-projection-001.json`.
- Preserve `UsbProfile` as the physical USB transport/personality observation,
  but add an independent execution-owner vocabulary:
  `unknown`, `rom`, and `application`.
- USB descriptors may prove `worker_runtime` or the shared
  `serial_jtag_transport`; they may not by themselves prove ROM or application
  ownership.
- A successful exact ESP32-S3 `board-info` exchange proves `rom` only for that
  observation epoch. A validated boot-profile or runtime-attestation marker
  proves `application` for its boot session. Worker descriptors may prove
  application ownership because only the installed application exposes that
  project-owned VID/PID/product tuple.
- Keep one deep `UsbOwnership` Interface: callers request inspect, ROM exit,
  observe, or recover intents and never choose reset lines, force-bit handling,
  child programs, or execution-owner joins directly.

## Corrected ROM exit

- The firmware handoff may continue setting `RTC_CNTL_FORCE_DOWNLOAD_BOOT` to
  enter ROM; this child does not alter the accepted Worker-to-ROM sequence.
- The host exit Adapter must use the contained, non-symlinked managed
  `esptool.py` and the exact ESP32-S3 application-exit command with
  `--before no_reset`, `--after hard_reset`, `--no-stub`, and `run`.
- Reject `run` combined with `--after no_reset`: Espressif defines that option
  as retaining the serial bootloader, and the protected predecessor log proved
  the exact `Staying in bootloader` result.
- Bind the pinned esptool behavior that clears
  `RTC_CNTL_FORCE_DOWNLOAD_BOOT` before ESP32-S3 hard reset. Do not substitute
  espflash RTS reset, watchdog reset, direct register writes, or another tool
  without a later plan change.
- Before exit, read only the exact RTC option register through contained
  esptool and privately reduce it to `force_download_bit_set`. Raw addresses,
  register values, commands, and device identity remain protected.
- Pre-arm the same-device profile watcher before the exit command. Record
  disappearance, enumeration change, transport observations, process cleanup,
  and the earliest failure without assuming that USB-Serial-JTAG implies ROM.

## Boot-lifetime application evidence

- Add a pure redaction-safe marker model for:
  - selected transport: `worker_runtime` or `serial_jtag_runtime`;
  - reason: `worker_started`, `diagnostic_owner`, or
    `boot_baseline_unconfirmed`;
  - baseline: `confirmed`, `unconfirmed`, or `diagnostic`;
  - current boot session/ordinal and safe build identity digest.
- Firmware registers the marker after the final USB-owner decision and the
  existing boot-lifetime observer emits it immediately and every existing boot
  evidence interval. Production must not depend on Wi-Fi, HTTP, Stratum,
  mining, ASIC progress, or a late service session for replay.
- The host parser accepts only exact complete markers, rejects inconsistent
  duplicates, and joins them to the current source/build and retained physical
  device. Raw serial material remains private.
- Existing recovery-006 may satisfy the hardware discriminator with its current
  recurring runtime-attestation marker. The new boot-profile marker is software
  hardening for the canonical package; this plan does not authorize flashing it.

## Hardware contract

After separate plan and implementation commits are fully verified and pushed:

1. Build the canonical exact-HEAD package for provenance only; do not flash it.
2. Run fresh `just detect-ultra205 --retain-rom` and require one admitted Ultra
   205 with successful ESP32-S3 board information and no foreign holder.
3. Run the effect-free `native-usb-rom-exit preflight`; it creates no root and
   performs no USB command.
4. Run `start` once. It may read the exact force-download bit, execute the exact
   corrected ROM-exit command, observe the same physical connector, and passively
   capture at most 30 seconds of application evidence.
5. Accept only:
   - `worker_runtime` with the same physical identity; or
   - `serial_jtag_runtime` plus a current exact recovery-006 runtime-attestation
     or boot-profile marker.
6. If only undifferentiated Serial/JTAG remains, publish
   `execution_owner_unknown`, prove cleanup, and stop. Do not probe ROM again,
   reset again, flash, scan, associate, or reuse the root.

One start invocation and one corrected reset are allowed. A repeated
authoritative signature is terminal. The user's standing authorization covers
this exact progress-backed contract; no manual button action is included.

## Evidence and failures

The public projection is allowlisted and contains only safe source/evaluator
digests, force-bit boolean, selected reset Adapter, transport profile,
execution owner, application-marker status, bounded profile counts/timings,
physical-identity match, enumeration-change, no-write/no-network facts,
cleanup, terminal category, and `redaction_status`.

Ports, MACs, register addresses/values, reset commands, board-info, serial
bytes, boot-session nonces, device identities, process data, credentials,
network values, and timestamps remain mode-`0600` below the mode-`0700` root.

Closed failures include `force_bit_read_failed`, `force_download_not_set`,
`rom_exit_failed`, `physical_identity_drift`, `transport_unknown`,
`execution_owner_unknown`, `application_identity_mismatch`,
`application_reappearance_timeout`, and cleanup failures. Earliest failure
precedence is immutable.

## Tests and delivery

- Red-to-green public-Interface tests for descriptor-only unknown ownership,
  board-info ROM admission, Worker application admission, Serial/JTAG marker
  admission, inconsistent marker rejection, and transport/owner separation.
- Command tests proving exact managed esptool use, exact force-bit read bounds,
  hard-reset exit, rejection of `run + no_reset`, and absence of flash/NVS/erase
  operations.
- Firmware tests for all marker variants, immediate plus periodic replay,
  boot-lifetime ownership, complete closed vocabulary, and independence from
  network/mining owners.
- Fresh-process tests for private modes, runfiles, process groups, same-device
  joins, root consume-once behavior, cleanup, and public allowlisting.
- Update the native-USB ownership guard and documentation so future changes
  cannot collapse execution owner back into transport profile.
- Before each commit or hardware effect, run ordered Cargo formatting, strict
  Clippy, all-target/all-feature build, all-feature tests, Bright Builds,
  focused USB/firmware/workflow tests, all Bazel tests, firmware links/package,
  native-USB ownership, parity/progress, redaction, reference cleanliness,
  whitespace, sensitive-value scan, and final diff review.
- Commit and push the immutable plan separately. Commit and push one verified
  implementation commit before hardware. On accepted hardware evidence, write
  `RESULT.md`, archive only this child task, and leave predecessor tasks and
  parity statuses unchanged. On a terminal discriminator, keep this task active
  and blocked with its closed completion review.

## Exclusions

No firmware/NVS/settings/theme write, flash, erase, OTA, Wi-Fi scan or
association, HTTP request, hostname/mDNS/ARP/router/subnet discovery, mining,
ASIC work, fan/voltage/power control, manual BOOT/RESET, direct UART,
pins/pads/headers/probes, other device, durability cycle, recovery mutation,
or parity promotion is authorized.
