---
phase: 35-detector-gated-correlated-evidence-and-exact-parity-promotion
verified: "2026-07-23T04:22:38Z"
status: passed
score: 7/7 requirements satisfied
requirement_score: 7/7
roadmap_truth_score: 5/5
plan_score: 4/4
generated_by: codex
verification_mode: fresh_goal_backward
lifecycle_mode: yolo
phase_lifecycle_id: 35-2026-07-17T17-00-37
generated_at: "2026-07-23T04:22:38Z"
lifecycle_validated: true
reviewed_source_commit: a51fee794ea5d2d1a7b139d2795611625d6f357a
overrides_applied: 0
---

# Phase 35 Verification Report

**Status:** `passed`

**Hardware source:** `a51fee794ea5d2d1a7b139d2795611625d6f357a`

**Admitted root:** `0401e7b485df2d1ccfc67e63845f98b6217816a184901bf0595d03af3219757d`

## Verdict

Phase 35 achieves its goal. Attempt 31 proves one exact-current-package,
detector-gated, two-epoch Ultra 205 chain from package admission through Boot A,
confirmed hostname mutation, exactly-once normal reboot, authoritative Boot B,
restoration, cleanup, inventory, redaction, and atomic evidence admission.

## Requirements

| Requirement | Status | Evidence |
| --- | --- | --- |
| CFG-12 | Satisfied | Boot A and Boot B share the persisted setting digest; the same device reports a changed session, exact next ordinal, and software reset. |
| EVD-10 | Satisfied | Internal detector and board-info admission precede credential access and every flash or request effect. |
| EVD-11 | Satisfied | The admitted root binds exact source, reference, package, executable, factory image, runtime identity, and board category. |
| EVD-12 | Satisfied | Both epochs contain coherent API, WebSocket, retained-log, session, revision, duration, and setting facts. |
| EVD-13 | Satisfied | The device-session quorum proves the same physical device, exact build, one restart request, changed boot session, `N → N+1`, `software_cpu`, and persisted postcondition. |
| EVD-14 | Satisfied | Restoration, process cleanup, inventory, lifecycle, no-actuation, reference, current-head, digest, and full admitted-tree redaction checks pass. |
| EVD-15 | Satisfied | Exactly four dedicated passive board-205 rows promote; 87 rows remain fingerprint-identical and eleven scopes are typed non-promotions. |

## Publication Audit

The admitted generation contains exactly:

- `.phase35-generation-manifest.json`
- `admitted.json`
- `decision-matrix.json`
- `projection.json`

The manifest digests recompute for the projection, matrix, and canonical
checklist. No generated checklist copy exists. The canonical checklist diff
contains only the four allowlisted rows.

## Verification Commands

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- Focused device-session, flash, Phase 35 HTTP/supervisor/promotion/parity, and
  Phase 30 Bazel suites
- `just verify-reference`
- `just parity`
- Exact Phase 35 lifecycle validation
- `just verify-redaction`
- `just phase35-evidence preflight-only=true`
- Manifest digest, checklist-row, and `git diff --check` audits

## Non-Claims

Active control, self-test effects, watchdog intervention, mining/Stratum/ASIC,
archived Phase 28.1.1, credentials, direct UART/pins, OTA/recovery, other boards,
lifecycle-only proof, and broader/unmapped claims remain unverified.
