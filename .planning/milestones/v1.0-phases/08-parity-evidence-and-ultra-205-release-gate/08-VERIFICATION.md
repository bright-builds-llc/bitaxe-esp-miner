---
phase: 08-parity-evidence-and-ultra-205-release-gate
verified: 2026-06-29T00:40:06Z
status: passed
score: "23/23 must-haves verified"
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 8-2026-06-28T21-51-32
generated_at: 2026-06-29T00:40:06Z
lifecycle_validated: true
overrides_applied: 0
---

# Phase 8: Parity Evidence And Ultra 205 Release Gate Verification Report

**Phase Goal:** Ultra 205 V1 parity claims are evidence-backed, scoped, and ready for release without expanding into deferred boards, protocols, accessories, or UI rewrites.
**Verified:** 2026-06-29T00:40:06Z
**Status:** passed
**Re-verification:** No - initial verification

## Goal Achievement

Phase 8 achieved the evidence-gate goal conservatively. The code and docs now prevent release/checklist false positives, record package and Ultra 205 detector/factory-flash/serial evidence, keep `DEVICE_URL status: blocked - no reachable DEVICE_URL` visible, and leave live HTTP/static/recovery/OTA/rollback/destructive rows below `verified`.

The missing live `DEVICE_URL` evidence is not treated as goal failure because the phase did not claim live release parity. It is explicit residual release risk, and the affected checklist rows remain `implemented`, `deferred`, or below verified.

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Every V1 parity surface records observable behavior, reference breadcrumb, Rust implementation pointer, status, evidence, and notes. | VERIFIED | `docs/parity/checklist.md` validates through `just parity` with `validation_errors: none`; plan artifact verification passed for the checklist. |
| 2 | `verified` means evidence-backed parity, and safety-critical rows require hardware-smoke or hardware-regression evidence. | VERIFIED | `cargo test -p bitaxe-parity --all-features` passed 39 tests, including verified-row, safety-critical, and release/OTA evidence guards. |
| 3 | Non-205 boards and ASICs remain unverified or deferred unless each has its own evidence set. | VERIFIED | `CFG-002`, `ASIC-008`, `ASIC-009`, `ASIC-010`, `STR-005`, `BAP-*`, all-board, and UI expansion remain deferred/not-started or below verified; guard tests reject Ultra 205 evidence inheritance. |
| 4 | Rust modules porting reference behavior have module or behavior breadcrumbs without line-by-line translation comments. | VERIFIED | Breadcrumb audit found reference breadcrumbs across Rust-owned crates/firmware/tools; release summary records the audit. |
| 5 | Release readiness is derived from unit, golden, API-compare, hardware-smoke, hardware-regression, or soak evidence instead of implementation status alone. | VERIFIED | `just parity` passed, release summary cites command evidence and residual risks, and rows with package/workflow-only evidence stay below verified when live evidence is missing. |
| 6 | Live HTTP/static/recovery/OTA/rollback/erase/failure evidence must exist before release parity is claimed. | VERIFIED | Evidence ledger records `DEVICE_URL status: blocked - no reachable DEVICE_URL`; affected surfaces are `not run` and no release parity is claimed. |
| 7 | Verified checklist rows fail validation when their evidence class does not support the claim. | VERIFIED | Parity tests include pending-evidence, safety-critical, release-image, firmware OTA, filesystem, and OTAWWW verified-row rejection cases. |
| 8 | Release-sensitive rows cannot be verified from package, unit, workflow, or API-compare evidence alone. | VERIFIED | `FS-001`, `OTA-001`, `REL-001`, `REL-002`, and `REL-003` remain `implemented` with blocked live evidence, not `verified`. |
| 9 | Non-205, deferred ASIC, deferred protocol, BAP, all-board image, and UI rewrite rows cannot inherit Ultra 205 evidence. | VERIFIED | `deferred_scope_verified_rows_reject_ultra205_evidence` passed; checklist statuses remain conservative. |
| 10 | Manifest-backed release-gate validation checks the supplied Ultra 205 package manifest. | VERIFIED | `bazel run //tools/parity:report -- release-gate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` returned `release_gate: passed`. |
| 11 | Phase 8 has a concrete evidence destination before live hardware actions begin. | VERIFIED | `docs/parity/evidence/phase-08-ultra-205-release-gate.md` exists and contains package, detector, flash, serial, and blocker sections. |
| 12 | `just detect-ultra205` runs before flash, monitor, HTTP, OTA, or recovery probes. | VERIFIED | Evidence ledger records exactly one Ultra 205 port, `/dev/cu.usbmodem1101`, and ESP32-S3 board-info before live actions. |
| 13 | A reachable device URL is established and sanitized, or a concrete blocker is recorded. | VERIFIED | Evidence ledger records `DEVICE_URL status: blocked - no reachable DEVICE_URL`; no private URL is committed. |
| 14 | No checklist row is promoted when hardware detection or `DEVICE_URL` discovery fails. | VERIFIED | Checklist keeps live HTTP/OTA/release rows below verified; `just parity` reports no validation errors. |
| 15 | Live static, recovery, OTA, OTAWWW, failed-update recovery, and boot-validation evidence runs only after detection and `DEVICE_URL` discovery succeed. | VERIFIED | Live sections are marked `not run - no reachable DEVICE_URL`; no evidence is overclaimed. |
| 16 | If the board is not reachable over HTTP, every live HTTP/OTA/recovery surface records a concrete not-run blocker. | VERIFIED | Evidence ledger and checklist notes repeat the concrete `DEVICE_URL` blocker for `FS-001`, `OTA-001`, `OTA-002`, and `REL-*`. |
| 17 | Destructive or fault-injection evidence runs only after recovery path, factory checksum, exact commands, expected observations, and stop criteria are documented. | VERIFIED | Large-erase/destructive section documents prerequisites and remains blocked; no destructive run is claimed. |
| 18 | OTAWWW remains an explicit REL-03 gap unless whole-www and interrupted-update hardware-regression evidence exists. | VERIFIED | `OTA-002` remains `deferred`; release docs keep the fail-closed `Wrong API input` operator response explicit. |
| 19 | Every current-release checklist row records observable behavior, reference breadcrumb, Rust-owned target, status, evidence, and notes. | VERIFIED | Checklist parses successfully and `just parity` returns `validation_errors: none`. |
| 20 | `verified` remains evidence-backed and safety/release-sensitive rows stay below verified when evidence is pending. | VERIFIED | Affected release rows remain `implemented` or `deferred`; no row was promoted during Phase 8. |
| 21 | Non-205 boards, deferred ASICs, deferred protocol expansion, BAP, all-board images, and Angular UI rewrite remain deferred or not-started. | VERIFIED | Checklist status review confirms deferred/not-started scope was not promoted. |
| 22 | Reference breadcrumbs are audited at Rust module or behavior boundaries without line-by-line translation comments. | VERIFIED | Release summary records the breadcrumb audit; no source diff exists in guarded areas after review fixes. |
| 23 | Release readiness summary cites concrete commands, package manifest data, evidence files, row status changes, and residual risks. | VERIFIED | `docs/parity/evidence/phase-08-ultra-205-release-summary.md` cites `just test`, `just package`, `just parity`, release-gate manifest, blocker status, and residual risks. |

**Score:** 23/23 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `tools/parity/src/main.rs` | Checklist validation and conservative verified-row guards | VERIFIED | Artifact verification passed; tests cover false-positive blocker text, release-sensitive rows, and deferred scope. |
| `tools/parity/src/release_gate.rs` | Release-gate and manifest validation | VERIFIED | Artifact verification passed; tests reject wrong-board metadata, missing artifacts, bad SHA-256, and unresolved artifact review. |
| `docs/parity/evidence/phase-08-ultra-205-release-gate.md` | Phase 8 package, detector, hardware, blocker, and destructive-gate evidence | VERIFIED | Artifact verification passed; `DEVICE_URL` blocker and not-run live surfaces are explicit. |
| `docs/parity/checklist.md` | Final parity checklist | VERIFIED | Artifact verification passed; `just parity` returned no validation errors. |
| `docs/parity/evidence/phase-08-ultra-205-release-summary.md` | Release readiness summary | VERIFIED | Artifact verification passed; records commands, manifest evidence, no row promotion, and residual risks. |
| `docs/release/provenance-manifest.md` | Release provenance | VERIFIED | Artifact verification passed; package artifact review is closed and publication remains gated. |
| `docs/release/license-inventory.md` | Release license inventory | VERIFIED | Artifact verification passed; release artifact rows remain publication-gated. |
| `docs/release/ultra-205.md` | Ultra 205 operator release notes | VERIFIED | Artifact verification passed; OTAWWW gap and unsupported live surfaces remain explicit. |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `docs/parity/checklist.md` | `tools/parity/src/main.rs` | `validate_rows` | WIRED | Key-link verification passed; `just parity` consumes checklist rows. |
| `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` | `tools/parity/src/release_gate.rs` | `validate_manifest_if_provided` | WIRED | Manifest-backed release-gate command passed. |
| `scripts/detect-ultra205.sh` | evidence ledger | `port=` | WIRED | Evidence records `/dev/cu.usbmodem1101` and ESP32-S3 board-info. |
| evidence ledger | live HTTP/OTA sections | `DEVICE_URL status` | WIRED | Blocker text is propagated to each not-run live surface. |
| evidence ledger | release guide | `large erase` and recovery gate | WIRED | Large erase remains documented and blocked pending recovery prerequisites. |
| evidence ledger | parity validator | `interrupted-update` | WIRED | OTAWWW/interrupted-update requirement remains below verified without hardware-regression evidence. |
| checklist | evidence ledger | Phase 8 evidence path | WIRED | Release rows cite `phase-08-ultra-205-release-gate.md`. |
| release summary | package manifest | `bitaxe-ultra205-package.json` | WIRED | Summary cites manifest-backed gate and package evidence. |
| `tools/parity/src/release_gate.rs` | provenance manifest | release artifact review checks | WIRED | Release gate passed with supplied manifest and provenance docs. |
| release docs | checklist/release summary | residual risk notes | WIRED | Release docs do not overclaim live HTTP/OTA/recovery support. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| `tools/parity/src/main.rs` | parsed checklist rows | `docs/parity/checklist.md` | Yes | FLOWING - `just parity` parsed real rows and returned `validation_errors: none`. |
| `tools/parity/src/release_gate.rs` | release-gate documents and package manifest | release docs plus `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` | Yes | FLOWING - manifest-backed release gate returned `release_gate: passed`. |
| `docs/parity/evidence/phase-08-ultra-205-release-summary.md` | command/evidence status | Phase 8 evidence ledger, checklist, release docs | Yes | FLOWING - summary cites actual command status and blocked live evidence instead of static success claims. |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Parity validator rejects unsupported verified claims and wrong promotion paths. | `cargo test -p bitaxe-parity --all-features` | 39 passed; 0 failed. | PASS |
| Checklist validates with conservative statuses. | `just parity` | `validation_errors: none`. | PASS |
| Release gate validates supplied Ultra 205 package manifest. | `bazel run //tools/parity:report -- release-gate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` | `release_gate: passed`. | PASS |
| Guarded source/reference tree has no uncommitted diff. | `git diff -- crates firmware tools reference/esp-miner --exit-code` | Exit code 0. | PASS |
| Artifact declarations in Phase 8 plans are present and substantive. | `gsd-tools verify artifacts` for all Phase 8 plans | All passed. | PASS |
| Phase 8 key links are wired. | `gsd-tools verify key-links` for all Phase 8 plans | All passed. | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| REL-08 | 08-01, 08-02, 08-03, 08-04 | Release parity requires evidence for rollback, recovery, erase, failed update, interrupted update, and release readiness. | SATISFIED | Release parity is not claimed; blocker and not-run live surfaces are explicit, and release gate rejects unsupported verified claims. |
| EVD-01 | 08-01, 08-04 | Checklist rows include behavior, breadcrumb, target, status, evidence, and notes. | SATISFIED | Checklist validates through `just parity`; plan artifact check passed. |
| EVD-02 | 08-01, 08-04 | `verified` is evidence-backed and safety-critical rows require hardware evidence. | SATISFIED | 39 parity tests passed; safety/release rows stay below verified when live evidence is missing. |
| EVD-03 | 08-01, 08-04 | Non-205/deferred scope cannot inherit Ultra 205 evidence. | SATISFIED | Deferred/not-started rows remain conservative; guard test passed. |
| EVD-04 | 08-04 | Reference breadcrumbs are present without line-by-line translation comments. | SATISFIED | Release summary records breadcrumb audit; guarded source diff is clean. |
| EVD-05 | 08-01, 08-02, 08-03, 08-04 | Release readiness uses layered evidence and documents residual risk. | SATISFIED | Summary cites unit/package/parity/release-gate commands, manifest data, evidence files, row status changes, and `DEVICE_URL` residual risk. |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| None blocking | n/a | Stub/placeholder/overclaim/secret scan | None | No over-promoted rows, private URLs, credentials, modified reference tree, or blocker-text false positives found. |

### Human Verification Required

None for Phase 8 goal achievement. Future release parity promotion still requires a reachable `DEVICE_URL` and live HTTP/static/recovery/OTA/rollback/destructive evidence, but Phase 8 correctly records that evidence as blocked and does not claim it.

### Gaps Summary

No blocking gaps found. The only material residual risk is the intentionally unclaimed live Ultra 205 HTTP/OTA/recovery evidence caused by `DEVICE_URL status: blocked - no reachable DEVICE_URL`. That risk is visible in the evidence ledger, checklist, release summary, and release docs, and the release gate prevents it from becoming a verified release parity claim.

_Verified: 2026-06-29T00:40:06Z_
_Verifier: the agent (gsd-verifier)_
