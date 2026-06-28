---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 8-2026-06-28T21-51-32
generated_at: 2026-06-28T21:54:06.223Z
---

# Phase 8: Parity Evidence And Ultra 205 Release Gate - Context

**Gathered:** 2026-06-28
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Phase 8 closes the V1 evidence governance and Ultra 205 release gate. It should prove or explicitly defer final V1 parity claims using the existing checklist, release documents, package manifest, release-gate tooling, and Ultra 205 evidence workflow. It must not expand V1 into non-205 boards, new ASIC families, Stratum v2, BAP completeness, an Angular UI rewrite, or unplanned hardware-control behavior.

</domain>

<decisions>
## Implementation Decisions

### Evidence Governance And Claim Policy

- **D-01:** Treat `docs/parity/checklist.md` as the release audit ledger. Every V1 parity surface must keep observable behavior, reference breadcrumb, Rust-owned target, status, evidence, and notes aligned before release readiness is claimed.
- **D-02:** Preserve the project meaning of `verified`: evidence-backed parity, not implementation completion. Package, compile, unit, golden, and API compare evidence can support appropriate rows, but live hardware, release-sensitive, OTA, rollback, recovery, and interrupted-update claims need their required evidence class before `verified`.
- **D-03:** Keep safety-critical and hardware-control rows below `verified` unless they have `hardware-smoke` or `hardware-regression` evidence. This includes voltage, fan, thermal, power, ASIC initialization, self-test hardware, runtime input, and runtime display surfaces.
- **D-04:** Add or tighten automated parity guards where release claims could otherwise drift, rather than relying on prose review alone.

### Ultra 205 Release Evidence Workflow

- **D-05:** Use the repo-local Ultra 205 hardware workflow before live hardware checks: run `just detect-ultra205`, continue only when exactly one likely ESP USB serial port is found and `espflash board-info --chip esp32s3 --port <port> --non-interactive` succeeds, then record that detector output in evidence.
- **D-06:** Phase 8 should prioritize the deferred live release surfaces from Phase 7: HTTP reachability, `/`, `/assets/app.css.gz`, missing static redirect behavior, `/recovery`, valid firmware OTA, invalid OTA rejection, OTAWWW gap response, rollback or boot-validation evidence, large erase recovery, failed update recovery, and interrupted-update recovery.
- **D-07:** Destructive or fault-injection checks are allowed only through phase-gated procedures that document the recovery path and required evidence. Do not run ad hoc erase, rollback, interrupted-update, voltage/fan/mining stress, or raw write commands outside those documented procedures.
- **D-08:** If the board is not reachable over HTTP or a destructive recovery path cannot be made explicit, record the evidence as pending or not run with a concrete blocker. Do not promote the corresponding checklist row.

### Release Gate And Documentation Closure

- **D-09:** Extend the existing `tools/parity` release-gate/checklist validation path instead of creating a second release-readiness tool. `just parity` and `bazel run //tools/parity:report -- release-gate` should remain the canonical command evidence.
- **D-10:** Release readiness should be derived from the package manifest, license inventory, provenance manifest, release operator guide, parity checklist, and evidence records together. A final release summary may be added, but it must cite concrete artifacts and command results rather than restating goals.
- **D-11:** Package artifacts must be tied to source commit, reference commit, checksums, tool versions, and artifact paths before publication. Generated firmware images remain GPL-risk-reviewed release artifacts until provenance and license review are explicitly complete.
- **D-12:** Keep `docs/release/ultra-205.md`, `docs/release/license-inventory.md`, `docs/release/provenance-manifest.md`, and `docs/parity/evidence/phase-07-ota-filesystem-release.md` as the starting release evidence set; update them only where Phase 8 produces new evidence or closes known gaps.

### Deferred Scope And Gap Handling

- **D-13:** Non-205 boards, deferred ASIC families, Stratum v2, BAP completeness, all-board factory image matrices, and an Angular AxeOS rewrite remain outside V1 release closure unless a later roadmap phase adds their own evidence path.
- **D-14:** OTAWWW remains an explicit REL-03 gap unless Phase 8 implements and proves whole-`www` partition update behavior, recovery access, and interrupted-update recovery on Ultra 205 with hardware-regression evidence.
- **D-15:** It is acceptable for Phase 8 to ship with explicit V1 gaps only when the checklist, release docs, evidence files, and release gate all make the gap, owner, impact, and follow-up path visible.

### the agent's Discretion

The agent may choose the exact Phase 8 plan split, evidence document names, release summary shape, test helper names, and whether validation logic lands in `tools/parity/src/main.rs` or `tools/parity/src/release_gate.rs`. Those choices must preserve the existing functional-core/imperative-shell boundary, keep upstream reference files read-only, avoid overclaiming verified status, and keep hardware evidence records free of secrets, private endpoints, pool credentials, Wi-Fi credentials, or NVS secret values.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope And Requirements

- `.planning/ROADMAP.md` - Phase 8 goal, success criteria, verification expectations, and no-expansion research boundary.
- `.planning/REQUIREMENTS.md` - REL-08 and EVD-01 through EVD-05, plus V2 deferred scope.
- `.planning/PROJECT.md` - Core value, Ultra 205 first target, evidence policy, safety constraints, and out-of-scope boundaries.
- `.planning/STATE.md` - Current progress, Phase 8 focus, accumulated evidence decisions, and remaining blockers.
- `.planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md` - Phase 7 release packaging, OTA, OTAWWW, recovery, and evidence decisions.

### Release And Evidence Documents

- `docs/parity/checklist.md` - Audit ledger and row-level status/evidence policy.
- `docs/parity/evidence/phase-07-ota-filesystem-release.md` - Phase 7 rollup that separates package, compile, serial hardware, gap, and Phase 8-deferred evidence.
- `docs/parity/evidence/phase-07-ultra-205-ota-hardware-smoke.md` - Latest Ultra 205 serial hardware record and the specific live HTTP/OTA/recovery surfaces deferred to Phase 8.
- `docs/release/ultra-205.md` - Operator guide, release evidence requirements, recovery procedures, and destructive workflow caveats.
- `docs/release/license-inventory.md` - Cargo, Bazel, ESP-IDF, flashing tool, static asset, and artifact license review inputs.
- `docs/release/provenance-manifest.md` - Source/reference/static/recovery/GPL/artifact provenance records.
- `PROVENANCE.md` - Project provenance, SPDX, GPL guardrails, and release review policy.

### Existing Rust And Firmware Integration Points

- `tools/parity/src/main.rs` - Checklist parsing, verified-row validation, release/OTA row guards, and report rendering.
- `tools/parity/src/release_gate.rs` - Release-gate validation for license inventory, provenance manifest, cargo-about output, and optional package manifest.
- `tools/parity/BUILD.bazel` - Bazel target for parity report and release-gate execution.
- `Justfile` - Human command surface for build, test, package, flash, monitor, flash-monitor, detect-ultra205, verify-reference, and parity.
- `scripts/detect-ultra205.sh` - Required hardware detection gate before autonomous Ultra 205 runs.
- `tools/flash/src/main.rs` - Flash/monitor workflow and manifest default-image reader.
- `tools/xtask/src/package_manifest.rs` - Manifest v2 and artifact metadata surface.
- `firmware/bitaxe/src/http_api.rs` - HTTP route shell, firmware OTA route, OTAWWW gap, recovery/static route registration, and API/websocket coexistence.
- `firmware/bitaxe/src/ota_update.rs` - ESP-IDF firmware OTA adapter.
- `firmware/bitaxe/src/boot_validation.rs` - ESP-IDF rollback boot-validation adapter.
- `firmware/bitaxe/src/filesystem.rs` - SPIFFS mount/status adapter.
- `firmware/bitaxe/src/static_files.rs` - Static and recovery file serving adapter.
- `crates/bitaxe-api/src/update_plan.rs` - Pure firmware OTA and OTAWWW access/status/gap decisions.
- `crates/bitaxe-api/src/static_plan.rs` - Pure static and recovery route resolution behavior.
- `crates/bitaxe-api/src/route_shell.rs` - Route ownership, access gate, and Phase 7 route manifest.

### Architecture, Evidence, And Policy ADRs

- `docs/adr/0001-device-user-parity.md` - Observable device-user parity definition.
- `docs/adr/0005-read-only-reference-implementation.md` - Read-only upstream reference policy.
- `docs/adr/0006-parity-checklist-as-audit-evidence.md` - Checklist as audit evidence.
- `docs/adr/0008-reference-breadcrumb-comments.md` - Breadcrumb placement policy.
- `docs/adr/0010-axeos-api-and-asset-compatibility.md` - API/static compatibility boundary before UI rewrite.
- `docs/adr/0012-parity-verification-evidence.md` - Evidence requirements and hardware-control verification gate.
- `docs/adr/0013-mit-first-with-gpl-guardrails.md` - GPL guardrails for upstream-derived behavior and fixtures.
- `docs/adr/0014-pivot-to-ultra-205-bm1366-first-parity.md` - Ultra 205/BM1366 first parity target and deferred non-205 scope.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `tools/parity/src/main.rs`: Already rejects `verified` rows with pending evidence, safety-critical rows without hardware evidence, and release/OTA rows without the required evidence shape.
- `tools/parity/src/release_gate.rs`: Already validates required release document sections, cargo-about presence, row-level unknown follow-ups, and optional manifest presence.
- `docs/release/ultra-205.md`: Already documents the Phase 8 evidence gate and safe procedures for OTA, recovery, large erase, failed update, interrupted update, and rollback.
- `docs/parity/evidence/phase-07-ultra-205-ota-hardware-smoke.md`: Already records the latest connected-board identity, package artifacts, serial proof, and the exact live HTTP/OTA/recovery gaps.
- `scripts/detect-ultra205.sh` and `just detect-ultra205`: Existing detector path required by repo-local hardware guidance.

### Established Patterns

- Pure validation belongs in host-testable Rust modules, with firmware and hardware effects behind thin adapters.
- Evidence documents use explicit conclusions such as `deferred to Phase 8`, `not run - hardware verification pending`, or `passed for serial scope` to avoid accidental verified claims.
- Release-sensitive rows stay below `verified` until the parity tool and evidence record agree on the correct evidence class.
- GSD artifacts must use standalone `---` only for opening and closing YAML frontmatter, not body separators.

### Integration Points

- Phase 8 planning should connect `docs/parity/checklist.md`, `tools/parity` validation, release docs, package manifest output, and hardware evidence records into one release-readiness story.
- Live hardware verification should start from `just detect-ultra205` and use repo commands such as `just package`, `just flash-monitor board=205 port=<port> evidence-dir=<path>`, `just monitor port=<port>`, and HTTP/OTA probes only after a reachable device URL is established.
- Destructive recovery tests must document recovery through the current package manifest and factory image before execution.

</code_context>

<specifics>
## Specific Ideas

- Do a final checklist audit that identifies rows that can be verified, rows that must remain implemented, and rows that should be deferred with owner/impact/follow-up.
- Treat the Phase 7 no-HTTP-address finding as a first-class Phase 8 planning concern: either establish a reachable device URL safely or record why live HTTP surfaces remain pending.
- Add targeted tests for any new parity/release gate rules before updating checklist rows.
- Keep the release summary factual: commands run, package manifest used, board/port, source commit, reference commit, evidence files, rows promoted, rows deferred, and residual risk.
- Do not copy upstream AxeOS or recovery page expression just to close release evidence. Use existing Rust-owned assets unless a later review explicitly changes provenance posture.

</specifics>

<deferred>
## Deferred Ideas

- Gamma 601/BM1370 and other non-205 board verification remain future-board phases.
- Stratum v2 completeness remains future protocol scope.
- BAP accessory parity remains future accessory scope.
- Angular AxeOS UI replacement remains out of V1; V1 keeps API and asset compatibility.
- All-board factory image matrices remain future release automation after each board has evidence.

</deferred>

*Phase: 08-parity-evidence-and-ultra-205-release-gate*
*Context gathered: 2026-06-28*
