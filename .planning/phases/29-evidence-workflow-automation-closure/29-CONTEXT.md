---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 29-2026-07-13T00-19-45
generated_at: 2026-07-13T00:19:45.615Z
---

# Phase 29: Evidence Workflow Automation Closure - Context

**Gathered:** 2026-07-12
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Phase 29 automates the existing Phase 25, Phase 27, and Phase 28 evidence lifecycle. Phase 25 and Phase 27 workflows must finish with strict operator-evidence validation, and a new Phase 28 workflow must consolidate Phase 27 artifacts into the complete Phase 23 evidence-slot inventory without manual assembly.

This phase closes EVD-07, EVD-08, EVD-09, and REL-09 workflow gaps. It does not diagnose live nonce production, create new hardware or safety claims, inspect raw credentials, or promote STR-09, CFG-07, or ASIC-11 to `verified`; those promotion outcomes remain Phase 30 work.

</domain>

<decisions>
## Implementation Decisions

### Canonical evidence-root model

- **D-01:** Keep the Phase 23 eleven-slot inventory authoritative: package, detector, board-info, command, log, API, WebSocket, share-outcome, redaction-review, safe-stop, and conclusion.
- **D-02:** Replace path/content guessing with explicit typed Phase 23, Phase 25, Phase 27, and Phase 28 workflow profiles.
- **D-03:** Drive root generation and validation from one shared schema descriptor so required slots, phase identity, allowed outcomes, and redaction rules cannot drift between builder and validator.
- **D-04:** Normalize every completed Phase 25 and Phase 27 workflow into the canonical inventory. A slot without phase-native observation must be an explicit typed `blocked` or `deferred` result with a stable reason; generated slots must never imply success.

### Phase 25 and Phase 27 finalization

- **D-05:** Each wrapper must have one explicit top-level terminal finalizer after workflow setup. Do not rely on duplicated branch calls or an `EXIT` trap.
- **D-06:** Finalization order is fixed: finish the complete slot inventory, write or verify the redaction review, run `mining-allow` where applicable, then run `operator-evidence --require-redaction-passed` exactly once and last.
- **D-07:** Accumulate workflow and validator statuses without short-circuiting. A validator failure makes the command fail, and a passing validator must never mask an earlier detector, prerequisite, capture, safe-stop, or workflow failure.
- **D-08:** Detector and prerequisite failures must still produce a complete, valid blocked root before returning nonzero. Blocked API/WebSocket slots retain the established stale-target and network-discovery prohibitions.

### Phase 28 consolidation and reruns

- **D-09:** `phase28-evidence` takes an explicit Phase 27 source root and an explicit dedicated Phase 28 destination root. Source and destination must be distinct and non-nested.
- **D-10:** Consolidation reads allowlisted category fields from committed Phase 27 artifacts and emits stable relative cross-links. It must not copy raw runtime directories, logs, credentials, endpoints, device identifiers, or other private local evidence.
- **D-11:** Generate all eleven slots in a sibling staging root, run redaction and operator-evidence validation there, then atomically replace the generator-owned destination. On failure, retain the previous valid destination unchanged.
- **D-12:** Identical source inputs should produce byte-identical generated content. Unknown destination files, contradictory source categories, unknown outcome tokens, or missing mandatory source artifacts fail closed.
- **D-13:** Preserve the Phase 27 closed outcome set: `accepted`, `rejected`, or `blocked_safe_prerequisite`. Accepted or rejected is valid only with the exact required ASIC-correlation and safe-stop evidence; blocked stays blocked with inherited non-claims.

### Documentation and regression contract

- **D-14:** Add one `just phase28-evidence` command backed by Bazel and a repo-owned wrapper; document automatic validation for Phase 25, Phase 27, and Phase 28 in the Ultra 205 operator flow.
- **D-15:** Regression tests must prove full slot inventories, exact validator command and ordering, exactly-once invocation, failure propagation, deterministic reruns, atomic destination preservation, cross-link-only output, all three share outcomes, and continued rejection of overbroad parity promotion.
- **D-16:** Hardware behavior remains unchanged. Any hardware-mode verification must use `just detect-ultra205`, board `205`, runtime-only local credential paths, at least 360 seconds for real capture, and redacted commit-ready evidence.

### the agent's Discretion

- Exact Rust type, module, helper, shell-function, Bazel target, and generated-file names.
- Whether the shared schema descriptor lives in the existing operator-evidence module or a focused child module, provided generation and validation consume the same typed contract.
- Stable blocked-reason strings and generated Markdown wording, provided they are deterministic, redaction-safe, and preserve exact non-claims.

</decisions>

\<canonical_refs>

## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope and governance

- `.planning/ROADMAP.md` — Phase 29 goal, dependency, success criteria, and Phase 30 boundary.
- `.planning/REQUIREMENTS.md` — EVD-07, EVD-08, EVD-09, and REL-09 definitions and traceability.
- `.planning/v1.1-MILESTONE-AUDIT.md` — Original workflow integration and manual-consolidation gap diagnosis.
- `AGENTS.md` — hardware-first evidence workflow, detector gate, credential handling, redaction, timeout, and direct-UART restrictions.
- `AGENTS.bright-builds.md` — repository workflow and high-signal engineering rules.
- `standards/core/architecture.md` — functional core and imperative shell boundary.
- `standards/core/code-shape.md` — shallow control flow and rerunnable script guidance.
- `standards/core/testing.md` — focused behavior tests and Arrange/Act/Assert expectations.
- `standards/core/verification.md` — repo-native verification and clean pre-commit gate.
- `standards/languages/rust.md` — Rust module, type, testing, and verification guidance.

### Locked evidence contracts

- `.planning/phases/23-redacted-operator-evidence-workflow/23-CONTEXT.md` — canonical slot inventory, runtime-only credentials, redaction, and exact-claim rules.
- `.planning/phases/25-live-stratum-runtime-and-safe-stop/25-CONTEXT.md` — safe-stop, evidence-root extension, and exact promotion boundary.
- `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md` — evidence-class and blocker-as-proof rejection rules.
- `.planning/phases/27-live-hardware-asic-and-stratum-bridge/27-CONTEXT.md` — detector-gated wrapper and closed share-outcome categories.
- `.planning/phases/28-hardware-evidence-and-checklist-promotion/28-CONTEXT.md` — cross-link-only consolidation and conservative promotion rules.
- `.planning/phases/28.1.1-bm1366-nonce-production-wire-parity/28.1.1-CONTEXT.md` — Phase 29 automation versus Phase 30 promotion boundary.
- `docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/evidence-contract.md` — authoritative eleven-slot evidence-root contract.
- `docs/parity/evidence/phase-28-hardware-evidence-and-checklist-promotion/evidence-contract.md` — Phase 28 source cross-links and consolidation metadata.

### Implementation and operator surfaces

- `Justfile` — existing Phase 23, Phase 25, and Phase 27 operator commands.
- `scripts/BUILD.bazel` — evidence wrapper and regression-test targets.
- `scripts/phase23-redacted-operator-evidence.sh` — complete-root and end-of-run operator-validation reference implementation.
- `scripts/phase23-redacted-operator-evidence-test.sh` — deterministic full-root regression pattern.
- `scripts/phase25-live-stratum-evidence.sh` — Phase 25 workflow to normalize and finalize.
- `scripts/phase25-live-stratum-evidence-test.sh` — Phase 25 blocked/hardware workflow tests.
- `scripts/phase27-live-hardware-bridge-evidence.sh` — Phase 27 workflow to normalize and finalize.
- `scripts/phase27-live-hardware-bridge-evidence-test.sh` — Phase 27 blocked/hardware workflow tests.
- `tools/parity/src/operator_evidence.rs` — evidence-root inventory, phase policy, metadata, and redaction validation.
- `tools/parity/src/main.rs` — operator-evidence CLI and parity regression integration.
- `tools/parity/src/mining_allow.rs` — mining evidence allow-manifest guard.
- `docs/release/ultra-205.md` — REL-09 operator workflow documentation.
- `docs/parity/checklist.md` — exact-claim checklist and overbroad-promotion guard surface.

\</canonical_refs>

\<code_context>

## Existing Code Insights

### Reusable Assets

- `scripts/phase23-redacted-operator-evidence.sh` already creates the complete inventory and invokes strict operator validation at workflow end.
- `tools/parity/src/operator_evidence.rs` already owns slot loading, metadata checks, redaction enforcement, target-source blockers, sentinel rejection, and Phase 28 checks.
- Phase 25 and Phase 27 wrappers already own their detector, package, capture, safe-stop, outcome, and mining-allow behavior; Phase 29 should normalize and finalize them, not replace their runtime logic.
- Existing Phase 23/25/27 shell tests provide deterministic fakes for wrapper behavior without real hardware.

### Established Patterns

- `just` is the human command surface and Bazel owns executable/test targets.
- Shell remains the thin orchestration layer; deterministic evidence classification and invariants belong in tested typed code where practical.
- Committed evidence uses category labels and cross-links. Raw or private local artifacts remain ignored and are never copied into promoted roots.
- Evidence validation is fail-closed, and blocked results are truthful non-claims rather than missing files or inferred success.

### Integration Points

- Add the Phase 28 alias in `Justfile` and wrapper/test targets in `scripts/BUILD.bazel`.
- Extend Phase 25 and Phase 27 terminal control flow around one finalizer each.
- Extend the operator-evidence model and CLI with explicit workflow profiles and shared generation/validation schema.
- Update `docs/release/ultra-205.md` and preserve `docs/parity/checklist.md` guard behavior.

\</code_context>

<specifics>
## Specific Ideas

- Prefer atomic managed-root regeneration over in-place mutation so an interrupted or invalid consolidation cannot leave a mixed-generation Phase 28 root.
- Treat phase identity as explicit typed input rather than an inference from operator-controlled paths or optional marker content.
- The operator validator should be the last workflow check, while its success remains subordinate to any earlier workflow failure.

</specifics>

<deferred>
## Deferred Ideas

- STR-09, CFG-07, and ASIC-11 verified promotion remains Phase 30 work.
- Live nonce/share production diagnosis remains in the Phase 28.1.1.x chain.
- New hardware, active-safety, recovery, non-205, Stratum v2, UI/BAP, and unbounded-stress claims remain outside this phase.

</deferred>

______________________________________________________________________

*Phase: 29-evidence-workflow-automation-closure*
*Context gathered: 2026-07-12*
