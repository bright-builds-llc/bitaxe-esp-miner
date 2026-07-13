# Phase 18: Firmware OTA And Rollback Evidence - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md. This log preserves the alternatives considered.

**Date:** 2026-07-03T14:09:29.677Z
**Phase:** 18-firmware-ota-and-rollback-evidence
**Mode:** Yolo
**Areas discussed:** Current package and device identity, Valid and invalid firmware OTA evidence, Rollback and boot-validation gate, Checklist/release docs/redaction/verification

## Current Package And Device Identity

| Option | Description | Selected |
| --- | --- | --- |
| Same-commit package gate | Run `just package`, release-gate, detector, and target provenance before OTA work. | yes |
| Reuse Phase 17 route evidence | Treat route-presence evidence as enough for OTA. | no |
| Infer target automatically | Discover the device through network scans or serial-log guessing. | no |

**User's choice:** Auto-selected same-commit package gate.
**Notes:** Phase 17 explicitly kept OTA as route-presence only. Phase 18 must either upload the manifest image or record a precise blocker.

## Valid And Invalid Firmware OTA Evidence

| Option | Description | Selected |
| --- | --- | --- |
| Phase 18 evidence wrapper | Reuse/harden the existing firmware OTA helper pattern while writing Phase 18-specific artifacts. | yes |
| Manual curl transcript | Run ad hoc uploads and summarize the terminal output. | no |
| Valid-only upload | Skip invalid rejection and only prove success response. | no |

**User's choice:** Auto-selected Phase 18 evidence wrapper.
**Notes:** Invalid image rejection remains required and is not rollback proof. Valid OTA must capture upload response, reboot identity, boot-validation or selected-partition state, and safe post-OTA behavior.

## Rollback And Boot-Validation Gate

| Option | Description | Selected |
| --- | --- | --- |
| Prefer non-destructive boot-validation proof | Use firmware retained markers and post-OTA identity before any destructive rollback/fault action. | yes |
| Force rollback fault injection | Run raw rollback invalidation or erase-like commands to prove rollback. | no |
| Treat invalid rejection as rollback | Count rejected invalid upload as rollback proof. | no |

**User's choice:** Auto-selected non-destructive boot-validation proof first.
**Notes:** Destructive rollback/fault cases require an active plan with exact commands, recovery image, restore command, allow flags, stop conditions, and redaction requirements.

## Checklist, Release Docs, Redaction, And Verification

| Option | Description | Selected |
| --- | --- | --- |
| Exact-claim promotion | Update checklist/docs only after Phase 18 artifacts exist and promote only the claims those artifacts prove. | yes |
| Goal-based promotion | Promote rows because the phase goal names OTA/rollback. | no |
| Skip redaction on local evidence | Trust local artifacts without a redaction review. | no |

**User's choice:** Auto-selected exact-claim promotion with mandatory redaction review.
**Notes:** Final verification must include repo-native checks, package/release-gate, parity, reference cleanliness, helper tests for changed helpers, hardware/network commands actually used, lifecycle validation, and `18-VERIFICATION.md` status `passed`.

## the agent's Discretion

- Exact helper names and whether Phase 18 wraps or adapts Phase 13 scripts.
- Evidence JSON field names and timeout values.
- Plan split and whether `tools/parity` needs new evidence-token guards.

## Deferred Ideas

- Recovery fault-injection, large erase, interrupted update, and OTAWWW whole-`www` update evidence belong to Phase 19 unless Phase 18 planning proves a narrow boot-validation prerequisite.
- Active safety telemetry belongs to Phase 20.
- Live mining and soak evidence belong to Phase 21.
