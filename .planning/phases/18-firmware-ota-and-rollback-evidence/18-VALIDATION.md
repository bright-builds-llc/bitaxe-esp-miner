---
phase: 18
slug: firmware-ota-and-rollback-evidence
status: ready
nyquist_compliant: true
wave_0_complete: false
created: 2026-07-03
---

# Phase 18 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
| --- | --- |
| **Framework** | Bazel `sh_test` for helper scripts and Bazel `rust_test` for Rust crates/tools |
| **Config file** | `MODULE.bazel`, `Cargo.toml`, `Justfile`, `scripts/BUILD.bazel`, `tools/parity/BUILD.bazel`, `firmware/bitaxe/BUILD.bazel` |
| **Quick run command** | `bazel test //scripts:phase13_firmware_ota_smoke_test //tools/parity:tests` when helper/parity behavior changes; `just parity` for docs/checklist-only changes |
| **Full suite command** | `just test && just package && just parity && just verify-reference`, plus manifest-backed `release-gate` for the copied Phase 18 manifest and every hardware/network command actually used |
| **Estimated runtime** | ~300 seconds plus hardware/network command time |

## Sampling Rate

- **After every task commit:** Run targeted helper/parity checks for changed code or `just parity` for docs/checklist changes.
- **After every plan wave:** Run `just test` for code/helper changes, then `just parity` and `just verify-reference`.
- **Before phase verification:** Full suite must be green or the phase must record exact hardware/network blockers without promotion.
- **Max feedback latency:** 300 seconds for local checks; hardware commands may exceed this only when evidence capture is active and bounded by the plan.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 18-01-01 | 01 | 1 | REL-02, REL-08, EVD-05 | T-18-02, T-18-03 | Phase 18 wrapper validates manifest-listed firmware, invalid fixture handling, target provenance input, and blocked-safe outputs. | shell unit | `bazel test //scripts:phase18_firmware_ota_evidence_test` | no | pending |
| 18-01-02 | 01 | 1 | REL-07, EVD-05 | T-18-04, T-18-06 | Evidence contract and pending redaction review define claim boundaries before live artifacts exist. | docs/parity validation | `rg -n "firmware_ota_status|invalid_rejection_status|boot_validation_status|rollback_status|network_scan" docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence` | no | pending |
| 18-02-01 | 02 | 2 | REL-02, REL-07, EVD-05 | T-18-01, T-18-06 | Current package and release-gate identity bind source commit, reference commit, manifest, and firmware image before hardware use. | package/release gate | `just package && just release-gate` | partial | pending |
| 18-02-02 | 02 | 2 | REL-02, REL-08, REL-07, EVD-05 | T-18-01, T-18-05 | Detector, flash-monitor identity, and target lock either prove the selected board/target or record exact blocked evidence. | gated hardware smoke | `just detect-ultra205` then plan-owned `just flash-monitor board=205 ...` when detection passes | no | pending |
| 18-03-01 | 03 | 3 | REL-02, REL-08, EVD-05 | T-18-02, T-18-03, T-18-05 | Firmware OTA helper either captures invalid rejection, valid upload, post-reboot identity, and boot-validation markers, or records the exact blocker. | gated hardware OTA | `scripts/phase18-firmware-ota-evidence.sh ...` after package/detector/target gates | no | pending |
| 18-03-02 | 03 | 3 | REL-02, REL-08, REL-07, EVD-05 | T-18-03, T-18-06 | Firmware OTA ledger separates invalid rejection, valid OTA, boot validation, rollback, and destructive rollback non-claims. | ledger/parity validation | `just parity && rg -n "invalid_rejection_boundary|rollback_status|destructive_rollback_status" docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota.md` | no | pending |
| 18-04-01 | 04 | 4 | REL-02, REL-08, REL-07, EVD-05 | T-18-04, T-18-06 | Redaction review and final summary confirm committed Phase 18 evidence contains no secrets or private endpoints. | redaction audit | `rg -n -i "ssid|wifi|password|pool|token|device_url|nvs|stratum|https?://|wss?://|([0-9]{1,3}\\.){3}[0-9]{1,3}|([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}|secret" docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence` | no | pending |
| 18-04-02 | 04 | 4 | REL-02, REL-08, REL-07, EVD-05 | T-18-03, T-18-06 | Release docs, checklist, requirements, and verification artifact promote only evidence-backed claims and preserve blockers/non-claims. | final docs/parity verification | `just parity && just verify-reference` plus lifecycle verification | no | pending |

*Status: pending, green, red, or flaky.*

Plan 18-03 intentionally keeps the live OTA helper's generated artifacts in one bounded evidence bundle. The helper emits request headers, response bodies, curl error files, invalid fixture, detector rerun, smoke log, and post-OTA monitor log from one command; splitting those generated files into separate task ownership would create false boundaries. Ledger interpretation and claim promotion remain separate in Task 18-03-02 and Plan 18-04.

## Wave 0 Requirements

- [ ] `scripts/phase18-firmware-ota-evidence.sh` and `scripts/phase18-firmware-ota-evidence-test.sh` if the planner chooses a Phase 18-specific wrapper instead of calling the Phase 13 helper directly.
- [ ] `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/` ledgers and `redaction-review.md` before checklist/docs promotion.
- [ ] Target-lock refresh or explicit target input contract for Phase 18, because `DEVICE_URL` is not guaranteed.

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
| --- | --- | --- | --- |
| Connected Ultra 205 presence | REL-02, EVD-05 | Requires physical USB hardware. | Run `just detect-ultra205`; continue only if exactly one port and board-info succeed. |
| Explicit `DEVICE_URL` provenance | REL-02, REL-07 | Requires current bench/network target knowledge and must not be inferred by scans. | Provide an origin-only URL through a plan-approved target-lock or environment path; evidence must record sanitized provenance. |
| Valid firmware OTA upload and reboot identity | REL-02, REL-08 | Requires live device, HTTP route, reboot, and post-reboot observation. | Run the plan-owned Phase 18 OTA helper after package/detector gates; verify response, reboot, identity, and boot-validation markers. |
| Destructive rollback/fault path | REL-08 | May make the device unavailable and requires recovery approval. | Run only if the active plan documents exact allow flags, recovery image, restore command, stop conditions, and redaction steps. |

## Security Validation

| Threat Ref | Threat | Required Control | Verification |
| --- | --- | --- | --- |
| T-18-01 | Wrong target receives OTA because target was inferred or stale. | Require explicit target lock or direct origin-only `DEVICE_URL`, board `205`, selected port, source/reference commits, and `network_scan: disabled`. | Inspect Phase 18 evidence ledger and target artifacts. |
| T-18-02 | Invalid firmware upload is accepted or misclassified. | Use fixed invalid fixture, rejection status, rejection body marker, and blocked evidence on unrelated responses. | Helper tests and invalid OTA evidence artifact. |
| T-18-03 | OTA response is cited as rollback proof. | Separate valid upload, invalid rejection, boot-validation, rollback, and non-claim fields. | `just parity` and checklist review. |
| T-18-04 | Secrets or private endpoints leak into evidence. | Redact response bodies, curl errors, serial logs, target locks, and docs before commit. | Redaction review plus targeted `rg` audit. |
| T-18-05 | Device becomes unavailable after OTA/fault action. | Prefer non-destructive boot validation; destructive actions require recovery image, restore command, stop conditions, and safe-state markers. | Plan gate plus evidence conclusion. |
| T-18-06 | Checklist overclaims release parity. | Promote only exact evidence tier and keep rollback/fault claims below verified unless terms and artifacts exist. | `just parity`, `tools/parity` tests when changed, and final verifier. |

## Validation Sign-Off

- [x] All planned tasks must include automated verify commands or explicit hardware gates.
- [x] Sampling continuity: no three consecutive tasks may lack automated verification.
- [x] Wave 0 requirements are identified for planner use.
- [x] No watch-mode flags are part of verification.
- [x] Feedback latency target is under 300 seconds for local checks.
- [x] `nyquist_compliant: true` set in frontmatter.

**Approval:** approved 2026-07-03
