---
phase: 35-detector-gated-correlated-evidence-and-exact-parity-promotion
plan: "04"
subsystem: parity
tags: [hardware, evidence, device-session, admission]
requires:
  - phase: 35-03
    provides: exact typed promotion and atomic publication
provides:
  - one admitted exact-current-package two-epoch Ultra 205 evidence root
  - deterministic receive-only device-session reboot proof
  - exact four-row passive board-205 parity promotion
affects: [milestone-v1.2-audit]
key-files:
  created:
    - docs/parity/evidence/phase-35-detector-gated-correlated-evidence-and-exact-parity-promotion/projection.json
    - docs/parity/evidence/phase-35-detector-gated-correlated-evidence-and-exact-parity-promotion/decision-matrix.json
    - docs/parity/evidence/phase-35-detector-gated-correlated-evidence-and-exact-parity-promotion/admitted.json
  modified:
    - docs/parity/checklist.md
    - scripts/phase35-promotion-contract-test.sh
key-decisions:
  - "Bootloader effects use espflash; runtime observation uses the OS-native receive-only device-session adapter; HTTP proves application postconditions."
  - "The admitted generation contains only projection, matrix, verdict, and manifest; the canonical checklist is fingerprinted and atomically updated without duplication."
  - "Only four dedicated passive board-205 rows promote; eleven excluded scopes remain typed non-promotions."
requirements-completed: [CFG-12, EVD-10, EVD-11, EVD-12, EVD-13, EVD-14, EVD-15]
duration: 4d
completed: 2026-07-23
generated_by: codex
lifecycle_mode: yolo
phase_lifecycle_id: 35-2026-07-17T17-00-37
generated_at: 2026-07-23T04:22:38Z
---

# Phase 35 Plan 04: Exact Hardware Evidence and Promotion Summary

Attempt 31 admitted one exact-current-package, detector-gated, two-epoch Ultra
205 evidence root and promoted exactly four dedicated passive board-205 parity
rows after restoration, cleanup, redaction, and atomic publication passed.

## Accomplishments

- Replaced the unqualified runtime monitor with a deterministic macOS
  receive-only device-session adapter and an authoritative HTTP reboot quorum.
- Proved same-device reboot attribution through exact build identity, changed
  boot session, `N → N+1` ordinal, `software_cpu`, and persisted hostname after
  exactly one restart request.
- Preserved private operational evidence while admitting only typed facts and
  public provenance.
- Narrowed atomic publication to four commit-safe artifacts and retained the
  canonical checklist as the sole checklist truth.
- Promoted only `V12-HOSTNAME-205`, `V12-PACKAGE-IDENTITY-205`,
  `V12-OPERATOR-SNAPSHOT-205`, and `V12-RUNTIME-HEALTH-205`; preserved 87 other
  row fingerprints and eleven explicit non-promotions.

## Hardware Result

- **Attempt:** 31
- **Hardware source:** `a51fee794ea5d2d1a7b139d2795611625d6f357a`
- **Board category:** `205`
- **Evidence root:** `0401e7b485df2d1ccfc67e63845f98b6217816a184901bf0595d03af3219757d`
- **Terminal outcome:** `admitted`
- **Restoration and cleanup:** passed

Attempts 1 through 30 remain immutable non-promotion history and were not
spliced into the admitted generation.

## Verification

- Ordered Cargo format, Clippy, all-target build, and all-feature tests passed.
- Device-session, flash, HTTP, supervisor, parity, promotion, and Phase 30
  non-promotion Bazel suites passed.
- Reference, parity, lifecycle, redaction, manifest digest, exact row diff, and
  admitted-tree audits passed.
- The exact-current-HEAD Phase 35 preflight passed before Attempt 31.

## Non-Claims

The evidence does not promote active control, self-test effects, watchdog
intervention, mining/Stratum/ASIC, archived Phase 28.1.1, credentials, direct
UART/pins, OTA/recovery, other boards, lifecycle-only proof, or broader/unmapped
rows.

## Next Step

Run the v1.2 milestone audit without broadening the admitted Phase 35 claims.
