---
phase: 19
slug: recovery-regression-and-otawww-evidence
status: verified
threats_open: 0
asvs_level: 1
created: 2026-07-03T20:41:34Z
---

# Phase 19 - Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

## Trust Boundaries

| Boundary | Description | Data Crossing |
| --- | --- | --- |
| CLI input to Phase 19 wrapper | Untrusted manifest paths, image paths, output directories, target provenance, USB port, and allow flags enter `scripts/phase19-recovery-otawww-evidence.sh`. | Local paths, target URL provenance, device action intent |
| Phase 19 wrapper to Phase 16 helper | Phase 19 delegates live recovery and destructive/fault-injection actions to `scripts/phase16-recovery-regression.sh`. | Detector-gated hardware action requests |
| USB and HTTP evidence to committed docs | Serial, detector, board-info, HTTP, curl, and target artifacts may contain private network or secret values. | Device/network identifiers, request/response snippets, credential-adjacent metadata |
| Package and flash identity to release evidence | Generated package manifests and flash-monitor evidence become committed release evidence. | Source/reference commits, board ID, package artifacts |
| Phase evidence to release docs/checklist | Final docs and checklist entries derive user-facing release claims from committed artifacts. | Verified, pending, blocked, deferred, and non-claim statuses |

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
| --- | --- | --- | --- | --- | --- |
| 19-01/T-19-01 | Elevation of Privilege / Tampering | Wrapper target handling | mitigate | Wrapper accepts only origin-only direct targets or trusted board `205` flash-monitor evidence; writes `network_scan: disabled`; evidence checked in `target-lock.json` and wrapper tests. | closed |
| 19-01/T-19-02 | Tampering / Repudiation | Package and flash identity inputs | mitigate | Manifest path input, source/reference fields, and release-gate validation are preserved in package evidence and final summary. | closed |
| 19-01/T-19-03 | Denial of Service | Failed-update, large-erase, interrupted-upload flags | mitigate | Default no-allow behavior records pending evidence; live actions delegate to Phase 16 detector and board-info gates. | closed |
| 19-01/T-19-04 | Denial of Service / Tampering | OTAWWW whole-www claim | mitigate | Wave 0 and final evidence record OTAWWW as REL-03 gap; no whole-www proof is claimed from `www.bin`, route presence, static serving, or `Wrong API input`. | closed |
| 19-01/T-19-05 | Information Disclosure | Evidence artifacts | mitigate | Redaction review enumerates committed artifact classes and records final `redaction_status: passed`. | closed |
| 19-01/T-19-06 | Repudiation / Denial of Service | Raw destructive commands | mitigate | Wrapper tests prove default no-allow behavior and allow-flag delegation; no Phase 19 direct erase, rollback, raw write, or interrupted-upload implementation is used for committed evidence. | closed |
| 19-02/T-19-01 | Elevation of Privilege / Tampering | Flash-monitor and target provenance | mitigate | Hardware evidence is detector-gated; `serial-boot.md` records board `205`, trusted flash-monitor output, and redacted committed evidence. | closed |
| 19-02/T-19-02 | Tampering / Repudiation | Package identity | mitigate | `just package` and release-gate evidence are captured against the copied manifest, with source/reference commits recorded in package and serial evidence. | closed |
| 19-02/T-19-03 | Denial of Service | Flash-monitor execution | mitigate | Uses repo-owned `just flash-monitor`; recovery/destructive actions are explicitly non-claims for this plan. | closed |
| 19-02/T-19-04 | Denial of Service / Tampering | Static administration UI | accept | Plan scope is package/static identity only; OTAWWW remains documented as a release gap and non-claim. | closed |
| 19-02/T-19-05 | Information Disclosure | Target lock and serial artifacts | mitigate | Committed artifacts use `redact-evidence=true`, raw developer evidence stays under `target/`, and `target-lock.json` is blocked with `network_scan: disabled`. | closed |
| 19-02/T-19-06 | Repudiation / Denial of Service | Raw hardware commands | mitigate | Plan evidence uses repo-owned detector and flash-monitor commands only; no raw erase, rollback, write, or interrupted-upload command is cited. | closed |
| 19-03/T-19-01 | Elevation of Privilege / Tampering | Failed-update and interrupted upload requests | mitigate | No target is inferred or scanned; target provenance remains blocked and live HTTP/recovery actions stay pending. | closed |
| 19-03/T-19-02 | Tampering / Repudiation | Package identity used by recovery helper | mitigate | Recovery evidence uses the copied Plan 02 manifest and records pending/no-allow status because no live helper gate ran. | closed |
| 19-03/T-19-03 | Denial of Service | Interrupted upload and large erase | mitigate | Recovery logs show `PHASE19_ALLOW_*` gates absent, failed-update/large-erase/interrupted-update pending, and no erase/restore/monitor recovery sequence run. | closed |
| 19-03/T-19-04 | Denial of Service / Tampering | Static administration UI after update failure | mitigate | Post-failure HTTP/static/recovery promotion is withheld; release docs and verification keep this behavior below verified. | closed |
| 19-03/T-19-05 | Information Disclosure | Recovery, curl, detector, and monitor logs | mitigate | Final redaction review covers recovery artifacts and records no raw target, request/response body, or secret-bearing values in committed evidence. | closed |
| 19-03/T-19-06 | Repudiation / Denial of Service | Raw destructive commands | mitigate | Phase 19 wrapper delegates allowed recovery paths to Phase 16; Plan 03 committed only safe no-allow transcripts and pending logs. | closed |
| 19-04/T-19-01 | Elevation of Privilege / Tampering | OTAWWW HTTP route evidence | mitigate | OTAWWW route behavior is documented as gap evidence only; existing route/access behavior is not promoted to update proof. | closed |
| 19-04/T-19-02 | Tampering / Repudiation | Final package and evidence traceability | mitigate | Final summary, release docs, checklist, requirements, validation, and verification cite copied manifest, commits, release gate, lifecycle validation, and artifact paths. | closed |
| 19-04/T-19-03 | Denial of Service | Partial/interrupted update evidence | mitigate | Missing destructive/fault-injection evidence remains blocked, pending, below verified, or non-claimed in summary, release docs, checklist, and verification. | closed |
| 19-04/T-19-04 | Denial of Service / Tampering | Whole-www partition overwrite | mitigate | OTAWWW remains a REL-03 gap unless size checks, chunked erase/write behavior, recovery access, and interrupted-update hardware-regression evidence exist. | closed |
| 19-04/T-19-05 | Information Disclosure | Final evidence set | mitigate | `redaction-review.md` is passed after scanning target, network, credential, request/response, detector, board-info, serial, and recovery artifacts. | closed |
| 19-04/T-19-06 | Repudiation / Denial of Service | Destructive commands in final docs | mitigate | `19-VERIFICATION.md` inventories the hardware/network commands actually used and marks blocked/not-run actions explicitly. | closed |

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
| --- | --- | --- | --- | --- |
| R-19-01 | 19-02/T-19-04 | Phase 19 Plan 02 intentionally captures package/static identity only. Whole-`www` OTAWWW update behavior remains an explicit REL-03 gap with owner, blocker, operator impact, and follow-up path. | Phase 19 plan disposition, verified by agent | 2026-07-03 |

## Verification Evidence

| Check | Evidence |
| --- | --- |
| Wrapper syntax and regression tests | `bash -n scripts/phase19-recovery-otawww-evidence.sh scripts/phase19-recovery-otawww-evidence-test.sh` and `bazel test //scripts:phase19_recovery_otawww_evidence_test //scripts:phase16_recovery_regression_test` passed. |
| Target provenance and flash identity | Python JSON assertions passed for `target-lock.json` (`network_scan: disabled`, blocked target, no device URL) and `serial-boot/flash-command-evidence.json` (board `205`, trusted output, commit-redacted capture). |
| Redaction and non-claim evidence | `redaction-review.md` records `redaction_status: passed`; `summary.md`, `otawww.md`, `docs/release/ultra-205.md`, and `docs/parity/checklist.md` preserve OTAWWW/recovery gaps below verified. |
| Lifecycle and UAT status | `19-VERIFICATION.md` records lifecycle validation as valid; `19-UAT.md` records four agent-verified UAT passes with no issues. |

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
| --- | --- | --- | --- | --- |
| 2026-07-03 | 24 | 24 | 0 | Codex agent via `$gsd-secure-phase 19` |

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-07-03
