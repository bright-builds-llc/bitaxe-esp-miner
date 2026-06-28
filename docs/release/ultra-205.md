# Ultra 205 Release Operator Guide

This guide is the Phase 7 operator contract for Bitaxe Ultra 205 release
artifacts. It records the command and evidence structure before later plans add
live package, OTA, recovery, rollback, and hardware proof.

## Artifact List

Expected release artifacts:

- `esp-miner.bin` - firmware OTA image for `/api/system/OTA`.
- `www.bin` - AxeOS static image for `/api/system/OTAWWW` and `/recovery` once
  the static-update evidence gate is closed.
- Merged factory/recovery image - USB flash artifact containing bootloader,
  partition table, app image, and filesystem image offsets.
- Package manifest - release artifact metadata, paths, offsets, checksums,
  source commit, reference commit, tool versions, and install notes.
- `docs/release/cargo-about.html` - Cargo dependency license report.
- `docs/release/license-inventory.md` - non-Cargo and release artifact license
  inventory.
- `docs/release/provenance-manifest.md` - source/reference/static/recovery
  provenance manifest.

## Local Build And Package

Use the repo command surface:

```bash
just build
just package
```

`just package` must create the Ultra 205 package artifacts and manifest without
requiring manual artifact discovery. Treat successful packaging as package
evidence only; it does not prove live OTA, recovery, rollback, or interrupted
static update behavior.

## USB Flash

Use the Ultra 205 board selector:

```bash
just flash board=205
```

Pass `port=...` only when automatic port discovery is missing or ambiguous.
Factory image flashing is the recovery baseline when the web interface or
filesystem state is unknown.

## Monitor

Use:

```bash
just monitor
```

Monitor logs should record firmware identity, board/ASIC target, partition or
image identity, filesystem status, update status, and recovery-relevant errors
when those adapters exist.

## Flash And Monitor

Use:

```bash
just flash-monitor board=205
```

This is the preferred single command for evidence captures that need both a USB
flash and boot log. Evidence should include the command, board, port, firmware
commit, reference commit, package manifest path, and observed result.

## Firmware OTA

The AxeOS firmware upload path targets:

- File name: `esp-miner.bin`
- Route: `/api/system/OTA`
- Visible surface: AxeOS `Update Firmware`

Firmware OTA evidence must separately record accepted uploads, rejected uploads,
AP/APSTA rejection, progress/status labels, validation or activation failures,
reboot scheduling, boot validation, and rollback behavior before release parity
is claimed.

## AxeOS OTAWWW Status

The AxeOS static update path targets:

- File name: `www.bin`
- Route: `/api/system/OTAWWW`
- Visible surface: AxeOS `Update AxeOS`

OTAWWW static update is not available for verified parity until the D-16
evidence gate is closed. Until then, `/api/system/OTAWWW` must stay fail-closed
or below verified status, and release notes must describe REL-03 as an explicit
gap with owner, evidence state, release impact, and follow-up path.

## Recovery Page

The recovery route is:

- Route: `/recovery`
- Recovery upload file: `www.bin`
- Recovery surface: `AxeOS Recovery`

Use `/recovery` when normal AxeOS static assets are unavailable, corrupt, or
not mounted. The recovery workflow must preserve the `www.bin` file name, the
60 second wait warning, the instruction not to restart before a response, and
the explicit restart step after the upload response.

## Large Erase

Large erase procedures are destructive recovery actions. Use them only when the
package/recovery evidence identifies the device as recoverable through USB
factory flashing. Record the erase command, board, port, package manifest,
source commit, reference commit, and post-erase flash/boot outcome.

## Failed Firmware Update

For a failed firmware OTA, capture:

- Request target `/api/system/OTA`.
- File name and checksum for `esp-miner.bin`.
- Public response and internal update status.
- Validation, activation, or write error logs.
- Running partition after reboot or failed activation.
- Recovery path used to return the Ultra 205 to an operable state.

Do not treat a failed upload rejection as rollback proof unless boot validation
or rollback evidence is recorded.

## Interrupted Static Update

For interrupted static update evidence, capture:

- Request target `/api/system/OTAWWW`.
- File name and checksum for `www.bin`.
- Point of interruption and observed device state.
- Whether `/`, representative static assets, and `/recovery` are reachable.
- The recovery procedure that restores AxeOS.

If this evidence is not present, OTAWWW remains an explicit REL-03 gap for
release claims.

## Rollback And Boot Validation

Firmware OTA evidence must distinguish:

- Upload accepted.
- Next app partition selected.
- Device rebooted.
- New app booted.
- Boot validation completed.
- Rollback path observed or ruled out with evidence.

Successful `/api/system/OTA` response text is not enough to claim rollback or
boot-validation parity.

## Evidence Required Before Verified Claims

Before claiming verified release or update parity, evidence must exist for:

- Package artifact generation and manifest checksums.
- Live firmware OTA success and rejection on Ultra 205.
- Live recovery-page access on Ultra 205.
- Rollback or boot-validation behavior.
- Failed firmware update recovery.
- Interrupted static update behavior, or an explicit REL-03 gap.
- License inventory and provenance review covering Cargo and non-Cargo inputs.

Use status wording such as `not run - hardware verification pending` when live
Ultra 205 evidence has not been recorded.

## Release Caveats

Phase 7 release candidates may carry explicit gaps, but they must not imply that
package evidence proves live OTA, AxeOS OTAWWW, `/recovery`, rollback, large
erase, failed update, or interrupted update behavior. Keep AxeOS names, route
names, and artifact names stable: `AxeOS`, `esp-miner.bin`, `www.bin`,
`/api/system/OTA`, `/api/system/OTAWWW`, and `/recovery`.
