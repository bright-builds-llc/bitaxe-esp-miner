# Phase 16 Current-Commit Release Evidence Completion

## Scope

This directory is the evidence root for Phase 16 current-commit release
evidence on Ultra 205 board `205`. Artifacts here may support release,
HTTP/static/recovery, OTA, rollback, erase, failed-update, or
interrupted-update claims only when they belong to the same current package
manifest and pass redaction review.

Historical Phase 13, Phase 14, and Phase 15 artifacts remain useful context,
but they are not current-commit release proof for this phase.

## Required Command Order

1. `just package`
2. `bazel run //tools/parity:report -- release-gate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`
3. `bazel run //tools/parity:report -- release-evidence --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --evidence-root docs/parity/evidence/phase-16-current-commit-release-evidence-completion`
4. `just detect-ultra205`
5. `just flash-monitor board=205 port=<detected-port> manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json evidence-dir=docs/parity/evidence/phase-16-current-commit-release-evidence-completion/serial-boot`
6. Run explicit `DEVICE_URL` HTTP/static/recovery probes only after the current package identity gate passes.
7. Run firmware OTA and recovery regression helpers only after detector, board-info, manifest, image, allow-flag, recovery, and redaction prerequisites are satisfied.
8. Complete `redaction-review.md` before citing generated logs or response bodies in release docs or checklist rows.

## Current-Commit Identity Gate

The current package manifest must have `source_commit` equal to the local
`git rev-parse HEAD` value at validation time. Flash-monitor evidence must
match that same `source_commit`, the manifest `reference_commit`, trusted
wrapper output, and an observed firmware marker equal to the full commit or a
prefix of at least 12 characters.

All flash evidence JSON, monitor logs, and route evidence cited for Phase 16
must be under this directory. Evidence outside this root cannot support Phase
16 checklist promotion.

## Live Evidence Packs

Expected subdirectories:

- `serial-boot/` for detector-gated current package flash-monitor evidence.
- `http-static-recovery/` for explicit `DEVICE_URL` route probes covering `/`,
  `/assets/app.css.gz`, missing static, `/recovery`, `/api/system/info`,
  `/api/ws`, `/api/ws/live`, `POST /api/system/OTA`, and
  `POST /api/system/OTAWWW`.
- `firmware-ota/` for valid OTA, invalid rejection, post-reboot identity, and
  boot-validation or rollback-state evidence.
- `recovery-regression/` for failed-update, interrupted-update, large erase,
  restore, post-action safe-state, and post-restore HTTP/static checks.

## Blocked Evidence Rules

When `DEVICE_URL` is missing, invalid, or unreachable, write blocked evidence
with `network_scan: disabled` and do not infer a target from serial logs,
local network interfaces, or cached URLs.

When the Ultra 205 detector finds zero or multiple likely ports, board-info
fails, the selected port differs from the detector port, the package manifest
is stale, the factory image is missing, or allow flags are absent, write
pending evidence and do not run flash, OTA, erase, rollback, failed-update,
interrupted-update, or raw recovery commands.

## Checklist Promotion Rules

Checklist rows may be promoted only to the evidence tier supported by the
current Phase 16 artifacts. Package, serial, HTTP/static, firmware OTA,
recovery regression, release-gate, redaction review, and lifecycle validation
are separate evidence classes.

`POST /api/system/OTAWWW` remains `otawww_rel03_status: deferred` unless a
whole-`www` update plus recovery and interrupted-update hardware regression is
captured and redaction-reviewed.

## Prohibited Scope

Phase 16 does not add non-205 board verification, additional ASIC families,
Stratum v2, BAP, all-board release images, Angular AxeOS replacement,
voltage/fan/mining stress, network scans, raw flash writes, or unbounded fault
injection. Those claims remain below verified unless a later phase produces its
own evidence and recovery gates.
