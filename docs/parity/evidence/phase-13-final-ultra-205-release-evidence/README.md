# Phase 13 Final Ultra 205 Release Evidence

## Evidence Contract

This directory records Phase 13 release-candidate evidence for Bitaxe Ultra 205 board `205`. Evidence in this directory must tie each trusted observation to the package manifest, source commit, reference commit, exact command, generated artifact, and redaction status used for the observation.

Package, hardware, HTTP, OTA, recovery, rollback, erase, failed-update, and interrupted-update evidence remain separate evidence classes. A later row may cite only the class that was actually captured.

## Required Inputs

- Canonical package command: `just package`.
- Canonical package manifest: `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`.
- Manifest-backed release gate: `bazel run //tools/parity:report -- release-gate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`.
- Hardware gate before USB evidence: `just detect-ultra205`.
- Board target: `205` only.
- Live network gate: a reachable `DEVICE_URL` that points at the just-flashed Ultra 205.
- Redaction review for every generated log, JSON, Markdown evidence file, and copied command output before commit.

## Blocker Handling

`DEVICE_URL status: blocked` is a valid recorded outcome when `DEVICE_URL` is missing, ambiguous, unreachable, or appears to target the wrong device, per D-06.

No checklist row may be promoted from blocker evidence, per D-17. Blocker records preserve auditability, but they do not prove live HTTP, static, recovery, OTA, rollback, erase, failed-update, interrupted-update, or OTAWWW parity.

If a release-critical artifact is absent from the package manifest, record `package_status: blocked - missing <artifact>` and stop before any downstream hardware or network evidence is trusted.

## Generated Artifacts

- `package-release-gate.md` - package manifest identity, command log, required artifact list, release-gate result, blocker status, and conclusion.
- `redaction-review.md` - Phase 13 secret redaction checklist and review status.
- Later plan artifacts may add wrapper JSON, serial logs, HTTP probe output, OTA evidence, and recovery logs under this directory only after their plan-defined gates pass.

## Non-Claims

This scaffold does not verify package release readiness, live Ultra 205 boot behavior, live HTTP/static/recovery behavior, firmware OTA behavior, rollback, large erase recovery, failed update recovery, interrupted update recovery, or OTAWWW parity.

This scaffold does not modify or verify `reference/esp-miner`; that tree remains read-only reference evidence.
