# Ultra 205 Release Operator Guide

This guide is the Phase 7 operator contract for Bitaxe Ultra 205 release
candidates. It gives the commands, artifacts, evidence gates, and recovery
paths a developer needs before using package, flash, OTA, recovery, rollback,
or erase workflows on connected hardware.

Do not treat package generation as live OTA, rollback, recovery, large erase,
failed update, or interrupted update verification. Phase 8 owns that live
network and recovery evidence gate; until it runs, use explicit Phase 8
deferral language and keep affected parity rows below `verified`.

## Phase 8 Evidence Status

Phase 8 release evidence is recorded in
`docs/parity/evidence/phase-08-ultra-205-release-gate.md` and summarized in
`docs/parity/evidence/phase-08-ultra-205-release-summary.md`.

Current conclusion: package, manifest, Ultra 205 detector, factory flash, and
serial boot evidence are recorded, but live HTTP, static, recovery, firmware
OTA, invalid image rejection, rollback, failed update recovery, large erase, and
interrupted-update checks remain blocked by `DEVICE_URL status: blocked - no
reachable DEVICE_URL`.

Keep `FS-001`, `OTA-001`, `REL-001`, `REL-002`, and `REL-003` below
`verified` until a later evidence record includes the required live observations.
Keep the OTAWWW REL-03 gap and public response `Wrong API input` until
whole-`www` hardware-regression and interrupted-update evidence exists.

## Build And Package

Run the canonical command surface from the repository root:

```bash
just build
just package
```

`just package` writes the Ultra 205 package manifest at:

```text
bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json
```

The package manifest must name the release artifacts directly:

- `bitaxe-ultra205.elf` - USB flashing default image for `tools/flash`.
- `esp-miner.bin` - firmware OTA image for `/api/system/OTA`.
- `www.bin` - static filesystem image built from the Rust-owned `www` tree.
- `bitaxe-ultra205-factory.bin` - merged factory/recovery image.
- `otadata-initial.bin` - OTA data initial image or erased-flash fallback.

Record the package manifest path, source commit, reference commit, artifact
SHA-256 values, ESP-IDF version, Rust target, and package command output before
using the artifacts in release evidence.

## USB Flash

Flash the Ultra 205 with an explicit serial port:

```bash
just flash board=205 port=<port>
```

Use the factory image path from
`bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` when recovery requires
a full USB flash baseline. Record the exact port, package manifest, source
commit, reference commit, and observed flash result.

## Monitor

Monitor an already-flashed device with:

```bash
just monitor port=<port>
```

Logs should record firmware identity, board/ASIC target, reset reason, package
or partition identity when available, SPIFFS status, OTA status, rollback boot
validation status, and recovery-relevant errors.

## Flash And Monitor Evidence Capture

For the Phase 9 wrapper-owned USB flash and boot-log capture, run:

```bash
just flash-monitor board=205 port=<port> evidence-dir=docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening
```

The optional timeout override is:

```bash
just flash-monitor board=205 port=<port> evidence-dir=docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening capture-timeout-seconds=25
```

The JSON source of truth is
`docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening/flash-command-evidence.json`.
The wrapper-owned serial log is
`docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening/flash-monitor.log`.

Passing capture statuses are `completed` and
`timed_out_after_trusted_output`. Any other status means evidence capture failed and is not trusted.
The JSON record must name the board, port, source commit,
reference commit, package manifest path, flash image path, exact flash command,
exact monitor command, monitor log path, capture mode, capture status, timeout,
trusted-output flag, and conclusion.

This serial evidence proves only the wrapper-owned flash-monitor boot-log path.
It does not verify live HTTP, static, recovery, firmware OTA, invalid image
rejection, rollback, failed update recovery, large erase, interrupted update, or
OTAWWW behavior.

### Flash-Monitor Recovery

If evidence capture fails, use repo-owned recovery and diagnostic steps:

```bash
just detect-ultra205
just flash-monitor board=205 port=<port> evidence-dir=<path>
just monitor port=<port>
```

Use `just monitor port=<port>` as a diagnostic-only follow-up after the wrapper
failure has been recorded. raw `espflash monitor` output must not be used as the trusted Phase 9 proof.

## Firmware OTA

Firmware OTA uses the AxeOS firmware update route:

- Upload target: `/api/system/OTA`
- Expected upload file name: `esp-miner.bin`
- Success response: `Firmware update complete, rebooting now!`
- Visible AxeOS surface: `Update Firmware`

Before claiming firmware OTA verified, capture these facts on Ultra 205:

1. Upload accepted for `esp-miner.bin`.
2. Invalid image rejection or validation failure behavior.
3. AP/APSTA rejection behavior.
4. Progress/status labels and final public response.
5. Selected next app partition.
6. Reboot scheduling and post-reboot firmware identity.
7. Rollback and boot-validation evidence.
8. Recovery procedure if boot validation fails.

The success response proves only that the upload path reached the reboot step.
It does not prove rollback, boot validation, or return-to-operable-state parity
without matching hardware logs.

## AxeOS Static Update Gap

The upstream AxeOS static update route is:

- Upload target: `/api/system/OTAWWW`
- Expected upload file name: `www.bin`
- Visible AxeOS surface: `Update AxeOS`
- Current public gap response: `Wrong API input`

AxeOS update is not available in this release candidate. Use just package to create www.bin and flash the factory image, or use /recovery only after the documented evidence gate is complete.

This is the D-16 OTAWWW gap. Do not claim static update parity from `www.bin`
package generation alone. REL-03 remains an explicit release gap until Phase 8
evidence proves whole-`www` partition write behavior, recovery access, and
interrupted-update recovery on Ultra 205.

## Static And Recovery Smoke

Static filesystem smoke must cover:

- `/` serving the packaged AxeOS-compatible static entry point.
- Representative gzip smoke path `/assets/app.css.gz`.
- Missing static asset redirect behavior.
- API route coexistence for `/api/*`, `/api/ws`, `/api/ws/live`,
  `/api/system/OTA`, and `/api/system/OTAWWW`.
- `/recovery` availability when normal static assets are unavailable or
  damaged.

Static smoke is live firmware evidence only when the record names the connected
Ultra 205 board, serial port, firmware commit, reference commit, package
manifest path, and observed HTTP responses.

## Recovery Page

Use the recovery route when AxeOS static assets are missing, corrupt, not
mounted, or intentionally being restored:

- Route: `/recovery`
- Upload file: `www.bin`
- Recovery surface: `AxeOS Recovery`

Recovery evidence must capture the `/recovery` page load, upload attempt,
public response, response body, restart step, post-restart static route state,
and whether the device returned to an operable state. Do not mark recovery
verified until the hardware-smoke record contains these observations.

## Large Erase

Large Erase is destructive. Use it only when the device can be recovered
through USB factory flashing with the current package manifest.

Safe procedure:

1. Record the board, port, source commit, reference commit, and package
   manifest path.
2. Confirm `bitaxe-ultra205-factory.bin` is present in
   `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`.
3. Record the exact erase command and tool version before running it.
4. Flash the factory image with `just flash board=205 port=<port>`.
5. Monitor with `just monitor port=<port>`.
6. Record post-erase boot, filesystem, recovery, and API reachability.

Do not describe Large Erase as verified unless that full sequence is captured
in hardware evidence.

## Failed Firmware Update

For a failed firmware update, capture:

- Request target `/api/system/OTA`.
- File name and checksum for `esp-miner.bin`.
- Whether the failure is invalid image rejection, upload interruption, write
  error, validation failure, activation failure, or post-reboot validation.
- Public response and internal update status.
- Running partition after reboot or failed activation.
- Rollback logs when a pending image is marked valid or invalid.
- Recovery procedure used to return the Ultra 205 to an operable state.

Do not treat a rejected upload as rollback proof. Rollback requires bootloader
or boot-validation evidence from the actual post-update state.

## Interrupted Static Update

For interrupted static update evidence, capture:

- Request target `/api/system/OTAWWW`.
- File name and checksum for `www.bin`.
- Point of interruption and observed device state.
- Whether `/`, `/assets/app.css.gz`, missing static redirect behavior, and
  `/recovery` remain reachable.
- Recovery procedure that restores AxeOS or the factory image.
- Final conclusion.

Until the Phase 8 record exists, interrupted static update remains deferred and
OTAWWW remains the REL-03 gap.

## Rollback And Boot Validation

Rollback evidence must distinguish:

- Upload accepted.
- Next app partition selected.
- Device rebooted.
- New app booted.
- Boot validation marked the pending app valid.
- Boot validation marked a bad pending app invalid and rebooted.
- Rollback path observed or ruled out by captured logs.

Successful `/api/system/OTA` response text is not enough to claim Rollback or
boot-validation parity.

## Evidence Required Before Verified Claims

Before claiming verified release or update parity, evidence must exist for:

- Package artifact generation and manifest checksums.
- Live firmware OTA success and invalid-image rejection on Ultra 205.
- Live static filesystem and `/recovery` smoke on Ultra 205.
- Rollback and boot-validation behavior.
- Failed firmware update recovery.
- Interrupted static update behavior, or the explicit REL-03 gap.
- License inventory and provenance review covering Cargo and non-Cargo inputs.

Use `docs/parity/evidence/phase-07-ota-filesystem-release.md` for the Phase 7
rollup and
`docs/parity/evidence/phase-07-ultra-205-ota-hardware-smoke.md` for the manual
Ultra 205 OTA/recovery smoke capture.
