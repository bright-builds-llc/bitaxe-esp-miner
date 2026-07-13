---
phase: 08-parity-evidence-and-ultra-205-release-gate
phase_number: 08
status: verified
asvs_level: 1
block_on: high
threats_total: 18
threats_closed: 18
threats_open: 0
unregistered_flags: 0
generated_by: gsd-security-auditor
generated_at: 2026-06-29T02:05:35Z
---

# Phase 08 Security Audit

Phase 08 security verification checked only the threats declared in the Phase 08 plan threat registers and the summary threat flags. Implementation files, `reference/esp-miner`, and unrelated docs were treated as read-only.

## Threat Verification

| Threat ID | Category | Disposition | Status | Evidence |
| --- | --- | --- | --- | --- |
| T-08-01-01 | Repudiation | mitigate | CLOSED | `tools/parity/src/main.rs:504` dispatches verified release/OTA/filesystem rows to guard logic; `tools/parity/src/main.rs:591` requires REL-08 terms for `REL-003`; tests at `tools/parity/src/main.rs:1062` and `tools/parity/src/main.rs:1083` cover the required evidence failures. |
| T-08-01-02 | Tampering | mitigate | CLOSED | `tools/parity/src/release_gate.rs:218` parses manifest JSON with `serde_json::Value`; `tools/parity/src/release_gate.rs:228` applies schema/string/path/artifact checks; `tools/parity/src/release_gate.rs:427` requires 64-character lowercase SHA-256 values; tests at `tools/parity/src/release_gate.rs:787`, `tools/parity/src/release_gate.rs:843`, and `tools/parity/src/release_gate.rs:882` cover manifest failures. |
| T-08-01-03 | Repudiation | mitigate | CLOSED | `tools/parity/src/main.rs:620` rejects deferred/non-205 verified rows that reuse Ultra 205 evidence; `tools/parity/src/main.rs:631` names `CFG-002`, `ASIC-008`, `ASIC-009`, `ASIC-010`, `STR-005`, BAP, all-board, and Angular scope; test coverage starts at `tools/parity/src/main.rs:1169`. |
| T-08-01-04 | Information Disclosure | accept | CLOSED | Accepted risk is documented below in the accepted risks log. The accepting plan states this guard-only plan added no raw hardware logs or secrets and later evidence plans enforce redaction. |
| T-08-02-01 | Spoofing | mitigate | CLOSED | `scripts/detect-ultra205.sh:85` requires exactly one likely ESP USB candidate; `scripts/detect-ultra205.sh:104` runs `espflash board-info --chip esp32s3`; `docs/parity/evidence/phase-08-ultra-205-release-gate.md:22` records the detector pass and ESP32-S3 board-info. |
| T-08-02-02 | Information Disclosure | mitigate | CLOSED | `docs/parity/evidence/phase-08-ultra-205-release-gate.md:59` records `DEVICE_URL status: blocked - no reachable DEVICE_URL`; `docs/parity/evidence/phase-08-ultra-205-release-gate.md:61` records sanitized URL evidence for the no-URL branch; `docs/parity/evidence/phase-08-ultra-205-release-gate.md:65` says no exact private URL was committed; `docs/parity/evidence/phase-08-ultra-205-release-gate.md:185` through `:189` record secret redaction status. |
| T-08-02-03 | Denial of Service | mitigate | CLOSED | `tools/flash/src/main.rs:494` builds/reads the package manifest when no image override is supplied; `tools/flash/src/main.rs:521` prefers the `factory_merged_image` artifact; `tools/flash/src/main.rs:464` writes the factory image at `0x0`; `docs/parity/evidence/phase-08-ultra-205-release-gate.md:32` records package manifest and factory image paths. |
| T-08-02-04 | Repudiation | mitigate | CLOSED | `docs/parity/evidence/phase-08-ultra-205-release-gate.md:55` records the DEVICE_URL discovery section; `docs/parity/evidence/phase-08-ultra-205-release-gate.md:59` records the blocked status; `docs/parity/evidence/phase-08-ultra-205-release-gate.md:60` and `:66` record the concrete no-URL blocker before live probes. |
| T-08-03-01 | Elevation of Privilege | mitigate | CLOSED | `crates/bitaxe-api/src/update_plan.rs:101` plans update requests through the existing access gate; `crates/bitaxe-api/src/update_plan.rs:107` rejects denied access before route work; `firmware/bitaxe/src/http_api.rs:398` exercises `/api/system/OTA` through `plan_update_request`; `firmware/bitaxe/src/http_api.rs:403` only proceeds on `AcceptFirmwareOta`. |
| T-08-03-02 | Denial of Service | mitigate | CLOSED | `docs/parity/evidence/phase-08-ultra-205-release-gate.md:137` documents the destructive procedure gate; `docs/parity/evidence/phase-08-ultra-205-release-gate.md:143` through `:149` record manifest, factory artifact, recovery commands, erase command, expected observations, and stop criteria; `docs/parity/evidence/phase-08-ultra-205-release-gate.md:150` records the blocked gate result. |
| T-08-03-03 | Repudiation | mitigate | CLOSED | `docs/parity/evidence/phase-08-ultra-205-release-gate.md:79` through `:124` record firmware OTA, invalid image, failed update recovery, and boot-validation as not run with the concrete DEVICE_URL blocker; `docs/release/ultra-205.md:115` and `docs/release/ultra-205.md:227` state upload success is not rollback or boot-validation proof. |
| T-08-03-04 | Information Disclosure | mitigate | CLOSED | `docs/parity/evidence/phase-08-ultra-205-release-gate.md:181` through `:190` records that exact private URLs, Wi-Fi credentials, pool credentials, private endpoints, and NVS secret values were not committed. `docs/parity/evidence/phase-08-ultra-205-release-summary.md:120` through `:123` repeats the sanitized release-summary posture. |
| T-08-03-05 | Tampering | mitigate | CLOSED | Static/recovery planning stays on existing routes in `crates/bitaxe-api/src/static_plan.rs:127` through `:168`; firmware registers `/recovery` and the static wildcard only in `firmware/bitaxe/src/static_files.rs:29` through `:52`; Phase 8 evidence records no live static probes due the DEVICE_URL blocker at `docs/parity/evidence/phase-08-ultra-205-release-gate.md:68` through `:77`; source/reference diff was recorded clean at `docs/parity/evidence/phase-08-ultra-205-release-summary.md:92`. |
| T-08-04-01 | Repudiation | mitigate | CLOSED | `docs/parity/checklist.md:137` through `:142` keeps `FS-001`, `OTA-001`, `REL-001`, `REL-002`, and `REL-003` below `verified` with blockers; `tools/parity/src/main.rs:518` through `:617` enforces exact evidence terms for verified rows; `docs/parity/evidence/phase-08-ultra-205-release-summary.md:69` through `:83` records no row promotion. |
| T-08-04-02 | Tampering | mitigate | CLOSED | `docs/release/provenance-manifest.md:82` through `:90` ties artifact review rows to manifest-present artifacts; `docs/parity/evidence/phase-08-ultra-205-release-summary.md:51` through `:59` lists manifest artifacts and SHA-256 values; `tools/parity/src/release_gate.rs:352` through `:380` requires named Ultra 205 artifacts, and `tools/parity/src/release_gate.rs:427` through `:448` validates checksums. |
| T-08-04-03 | Information Disclosure | mitigate | CLOSED | `docs/parity/evidence/phase-08-ultra-205-release-summary.md:120` through `:123` states no private URLs, Wi-Fi credentials, pool credentials, private endpoints, or NVS secret values are recorded; `docs/parity/evidence/phase-08-ultra-205-release-gate.md:181` through `:190` records the evidence redaction review. |
| T-08-04-04 | Repudiation | mitigate | CLOSED | `docs/parity/evidence/phase-08-ultra-205-release-summary.md:97` through `:111` explicitly keeps `CFG-002`, `ASIC-008`, `ASIC-009`, `ASIC-010`, `STR-005`, `BAP-001`, `BAP-002`, Angular, and all-board scope deferred, not-started, pending, or out of V1 scope. |
| T-08-04-05 | Tampering | mitigate | CLOSED | `docs/parity/evidence/phase-08-ultra-205-release-summary.md:85` through `:95` records breadcrumb audit as audit-only and `git diff -- crates firmware tools reference/esp-miner --exit-code` as passed; this audit reran that command with exit code 0 before writing this file. |

## Accepted Risks Log

| Threat ID | Accepted Risk | Scope | Owner | Review Trigger | Status |
| --- | --- | --- | --- | --- | --- |
| T-08-01-04 | Plan 08-01 added parity/release-gate guard logic only and did not add raw hardware logs, private URLs, credentials, private endpoints, NVS secret values, network endpoints, auth paths, firmware effects, schema migrations, or filesystem write paths. | Evidence validation guard-only code in `tools/parity`; later Phase 8 evidence plans own secret redaction. | Phase 08 release evidence owner | Reopen if guard-only plans begin committing raw hardware logs or secret-bearing evidence. | CLOSED |

## Summary Threat Flags

| Source | Flag | Mapping | Result |
| --- | --- | --- | --- |
| 08-01 summary | None beyond planned trust boundaries; manifest validator expands the existing local manifest-to-release-gate trust boundary and adds no endpoint, auth path, firmware effect, schema migration, or filesystem write path. | T-08-01-02 | Informational |
| 08-02 summary | None beyond documented trust boundaries; evidence files only and no endpoint, auth path, file-access adapter, firmware effect, or schema change. | T-08-02-02, T-08-02-04 | Informational |
| 08-03 summary | None beyond planned trust boundaries; documentation evidence only and no endpoint, auth path, file-access adapter, firmware effect, or schema change. | T-08-03-02, T-08-03-04 | Informational |
| 08-04 summary | No `## Threat Flags` section present. Summary records DEVICE_URL remains blocked, no unsupported promotion, source/reference trees unchanged by final plan, and final gates passed. | T-08-04-01, T-08-04-04, T-08-04-05 | Informational |

Unregistered flags: none.

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
| --- | --- | --- | --- | --- |
| 2026-06-29 | 18 | 18 | 0 | gsd-security-auditor |

### Audit Checks

| Check | Result |
| --- | --- |
| Mandatory file load | Loaded all files listed in `<files_to_read>`, plus `$HOME/.codex/agents/gsd-security-auditor.md`, `$HOME/.codex/skills/gsd-secure-phase/SKILL.md`, the secure-phase workflow, and the workflow UI reference. |
| Input state | State B confirmed: `.planning/phases/08-parity-evidence-and-ultra-205-release-gate/08-SECURITY.md` did not exist before this audit. |
| Threat register | Extracted from the four Phase 08 plan threat models and reconciled against the prompt-provided 18-threat register. |
| Mitigation grep pass | Verified declared mitigation terms and guard functions with targeted `rg` checks across parity tooling, release-gate validation, evidence ledgers, release docs, detector script, OTA/static code, and release summary. |
| Source/reference read-only check | `git diff -- crates firmware tools reference/esp-miner --exit-code` exited 0 before this file was written. |
| Implementation edit scope | No implementation files, `reference/esp-miner`, or unrelated docs were modified by this audit. Only this SECURITY artifact was created. |

## Result

Threats closed: 18/18.

Threats open: 0.

ASVS level: 1.

## Sign-Off

- [x] All threats have a disposition: mitigate or accept.
- [x] Accepted risks documented in Accepted Risks Log.
- [x] `threats_open: 0` confirmed.
- [x] `status: verified` set in frontmatter.

**Approval:** verified 2026-06-29
