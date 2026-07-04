---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 23-2026-07-04T22-53-37
generated_at: 2026-07-04T22:55:05.649Z
---

# Phase 23: Redacted Operator Evidence Workflow - Context

**Gathered:** 2026-07-04
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Phase 23 creates the repo-owned, detector-gated operator workflow for bounded v1.1 production-mining evidence. It must define and automate the path from Ultra 205 detection through package or flash, local runtime-only credential input, bounded mining command execution, telemetry capture, safe-stop capture, redaction, evidence review, and final evidence-root conclusion.

This phase is primarily a workflow, documentation, evidence-packaging, and redaction-hardening phase. It does not implement the trusted BM1366 production work path, real Stratum socket adapter, live ASIC-derived share submission, accepted/rejected share proof, runtime telemetry promotion, full active voltage/fan/thermal/fault/self-test closure, OTA/recovery evidence, non-205 board support, Stratum v2, display/input parity, BAP, or unbounded stress mining. Those remain later v1.1 or future-milestone surfaces unless Phase 23 discovers a blocker that prevents safe redacted evidence capture.
</domain>

<decisions>
## Implementation Decisions

### Operator Evidence Flow

- **D-01:** Provide one documented repo-owned flow for board `205` that covers detection, package or flash, local credential input, bounded mining evidence, telemetry capture, safe-stop capture, redaction, redaction review, and evidence review.
- **D-02:** The flow must start with `just detect-ultra205` and stop when detection is absent, ambiguous, not board `205`, or board-info fails. No stale port, stale `DEVICE_URL`, network scan, mDNS, ARP, router, or unrelated evidence may substitute for the detector gate.
- **D-03:** Prefer a `just`-reachable command surface or repo-owned script wrapper over ad hoc instructions. If new hardware command shapes are needed, extend the repo validator and tests instead of bypassing allow-manifest style checks.
- **D-04:** The operator flow may accept local `wifi-credentials.json` and `pool-credentials*.json` files as runtime inputs only. The committed command summaries must record only category labels such as `pool_config: local-owner-supplied`, not the raw file contents or owner-identifying values.

### Single Redacted Evidence Root

- **D-05:** Phase 23 must define one v1.1 redacted evidence root shape for committed artifacts. Required slots are package, detector, board-info, command, log, API, WebSocket, share-outcome, redaction-review, safe-stop, and conclusion artifacts.
- **D-06:** Artifact slots that cannot be truthfully populated yet must still be represented with an explicit blocked, pending, or deferred outcome and the exact reason. In particular, share-outcome proof can remain a non-claim until Phase 25 observes a real accepted or rejected pool response to live ASIC-derived work.
- **D-07:** The evidence root must name board `205`, source commit, reference commit, package or firmware identity, detector evidence, command category, redaction status, observed behavior, safe-stop status, conclusion, and exact non-claims.
- **D-08:** Raw local artifacts may exist only in ignored/local paths when needed for diagnosis. The committed evidence root must contain redacted files or category summaries only and must declare `raw_artifacts_committed: no`.

### Redaction Contract

- **D-09:** Redaction must be designed as a first-class deliverable, not as a final manual cleanup step. Every new committed artifact type needs either automatic sanitization, an explicit review checklist, or both.
- **D-10:** Committed logs, command summaries, API captures, WebSocket captures, retained logs, NVS/settings values, Stratum fields, share payloads, safe-stop summaries, and conclusions must redact pool URLs, ports, users, workers, owner addresses, passwords, tokens, targets, extranonces, share payloads, socket errors, device URLs, IP addresses, MAC addresses, Wi-Fi values, NVS secrets, API tokens, and raw BM1366 frames.
- **D-11:** Add deterministic redaction tests or checks for the categories that existing Phase 17/21 tooling only partially covers, especially Stratum target values, extranonces, share payloads, socket errors, NVS/settings values, worker/address-like usernames, device URLs, IPs, MACs, Wi-Fi values, and pool credential fields.
- **D-12:** Operator-visible firmware and host logs should emit lifecycle/status markers with `redacted=true` or equivalent redaction-safe labels instead of raw Stratum messages, raw pool values, raw share payloads, raw socket errors, or raw ASIC frames.

### Reuse And Integration

- **D-13:** Reuse the strongest existing patterns from Phase 17 live API/WebSocket capture, Phase 21 mining evidence scripts and redaction tests, Phase 22 claim-ladder and blocker governance, `tools/flash` commit-redacted evidence, and `tools/parity` allow/evidence validators.
- **D-14:** The preferred implementation shape is a thin script or CLI shell around typed redaction/evidence decisions where practical. Pure redaction classifiers, evidence-root inventory validation, and claim/outcome modeling should live in tested Rust, Node, or shell-testable helpers rather than buried in long inline command strings.
- **D-15:** Preserve Phase 22 exact-claim language. Phase 23 can prove the operator workflow and redaction root, but it must not promote trusted BM1366 production work, live Stratum socket success, or accepted/rejected share outcomes unless those later-phase artifacts exist.
- **D-16:** Update parity and release/operator docs only to the exact level supported by Phase 23 artifacts. Evidence-governance rows may advance when the redacted workflow and review pass; production mining behavior rows remain pending or non-claimed until later phases.

### Verification Gate

- **D-17:** Verification must include tests for changed redaction/evidence helpers, script behavior, parity/evidence-root validation, `just parity`, `just verify-reference`, and lifecycle validation for this phase.
- **D-18:** If hardware is used, it must follow the Ultra 205 detector gate and evidence rules. If local credentials are used, the run must never print, summarize, commit, or copy secret values into evidence.
- **D-19:** The final phase verification may pass with blocked or pending hardware outcome slots only if those slots are explicitly marked as later-phase non-claims and the workflow, redaction, and evidence-root contract are otherwise proven.

### Claude's Discretion

Claude may choose exact file names, command names, script boundaries, validator names, artifact suffixes, test fixture values, and whether to extend existing Phase 21 helpers or create Phase 23-specific wrappers. Those choices must preserve detector gating, runtime-only secrets, redaction-safe committed artifacts, exact non-claim governance, rerunnable scripts, repo-native verification, and functional-core/imperative-shell separation.
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope And Requirements

- `.planning/ROADMAP.md` - Phase 23 goal, dependency on Phase 22, requirements, and success criteria.
- `.planning/REQUIREMENTS.md` - EVD-07, STR-10, REL-09, CFG-07, EVD-09, and v1.1 traceability.
- `.planning/PROJECT.md` - v1.1 Ultra 205 trusted production mining scope, active constraints, parity evidence policy, and out-of-scope boundaries.
- `.planning/STATE.md` - Carried v1.1 decisions, Phase 22 completion state, and current blockers.
- `AGENTS.md` - Ultra 205 detector gate, local credential handling, `DEVICE_URL` derivation limits, redaction rules, hardware evidence requirements, and frontmatter separator rule.
- `AGENTS.bright-builds.md` - Bright Builds workflow, standards routing, and verification expectations.
- `standards/core/architecture.md` - Functional core / imperative shell and boundary parsing.
- `standards/core/code-shape.md` - Early returns, script rerun-safety, foreign-code-in-strings guidance, and size triggers.
- `standards/core/testing.md` - Unit test structure and pure logic coverage expectations.
- `standards/core/verification.md` - Sync and repo-native verification before commit.
- `standards/languages/rust.md` - Rust module, invariant, optional naming, and verification guidance.

### v1.1 Research And Pitfalls

- `.planning/research/FEATURES.md` - Owner-ready operator workflow, redacted pool credential handling, evidence-governed claim promotion, and anti-features.
- `.planning/research/PITFALLS.md` - Pitfalls for redaction leakage, stale `DEVICE_URL`, allow-manifest bypass, safe-stop ambiguity, NVS credential handling, and controlled-evidence overclaiming.

### Upstream Phase Handoff

- `.planning/phases/21-live-mining-and-soak-evidence/21-CONTEXT.md` - Detector gates, runtime-only pool inputs, evidence ladder, telemetry correlation, safe-stop, and final verification expectations.
- `.planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md` - Claim ladder, typed prerequisite contract, blocker reasons, evidence boundaries, and exact non-claim handling.
- `.planning/phases/22-claim-ladder-and-safety-preconditions/22-03-SUMMARY.md` - Phase 22 closure handoff into redacted operator evidence.
- `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/evidence-contract.md` - Evidence ladder and required artifact categories to adapt into a single Phase 23 root.
- `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/redaction-review.md` - Existing mining evidence redaction review pattern.
- `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/summary.md` - Controlled no-share closure, residual non-claims, and final evidence matrix.
- `docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/claim-ladder.md` - v1.0 and v1.1 claim tiers and production-mining non-claim boundaries.
- `docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/blocker-reasons.md` - Stable redaction-safe blocker strings.
- `docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/redaction-review.md` - Deterministic redaction-review structure.

### Existing Workflow And Redaction Implementation

- `Justfile` - Human command surface including `detect-ultra205`, `package`, `flash-monitor`, `monitor`, `parity`, `verify-reference`, `build`, and `test`.
- `scripts/detect-ultra205.sh` - Required detector and board-info preflight.
- `scripts/phase21-live-mining-evidence.sh` - Closest mining evidence wrapper for smoke, bounded soak, pool credentials, API/WebSocket capture, redaction, and safe-stop markers.
- `scripts/phase21-live-mining-evidence-test.sh` - Existing redaction tests for Phase 21 wrapper outputs.
- `scripts/phase21-live-mining-package.sh` - Controlled live-mining package builder and compile-time acknowledgement gate.
- `scripts/phase21-pool-credentials-json.mjs` - Local JSON pool credential parser for runtime-only inputs.
- `scripts/phase21-pool-input-bridge.sh` - Pool settings bridge that applies local runtime inputs without committing secrets.
- `scripts/phase17-live-http-api-smoke.sh` - API capture and stream redaction pattern.
- `scripts/phase17-websocket-capture.mjs` - Bounded WebSocket capture helper.
- `tools/flash/src/main.rs` - Commit-redacted flash evidence, serial log sanitizer, command evidence JSON, and trusted artifact handling.
- `tools/parity/src/mining_allow.rs` - Mining allow-manifest validation, prohibited command tokens, safe-state markers, and redaction reviewer gate.
- `tools/parity/src/release_evidence.rs` - Release evidence redaction requirements and flash evidence validation.
- `tools/parity/src/main.rs` - Checklist validation, report command, and verified-row guardrails.
- `firmware/bitaxe/src/controlled_mining_runtime.rs` - Redacted controlled runtime markers and safe-stop status markers.
- `firmware/bitaxe/src/mining_evidence_mode.rs` - Fail-closed live-mining acknowledgement gate.
- `pool-credentials.json.example` - Committed shape only; real files stay local and ignored.

### Operator And Release Policy

- `docs/release/ultra-205.md` - Operator flash, credential, evidence, and redaction guidance.
- `docs/parity/checklist.md` - Current parity evidence semantics and exact row promotion rules.
- `docs/adr/0001-device-user-parity.md` - Observable device-user parity scope.
- `docs/adr/0005-read-only-reference-implementation.md` - Read-only upstream reference policy.
- `docs/adr/0006-parity-checklist-as-audit-evidence.md` - Checklist evidence policy.
- `docs/adr/0012-parity-verification-evidence.md` - Evidence class and verified status semantics.
- `docs/adr/0013-mit-first-with-gpl-guardrails.md` - GPL provenance guardrails.
- `docs/adr/0014-pivot-to-ultra-205-bm1366-first-parity.md` - Ultra 205 BM1366 first parity target and deferred non-205 scope.
- `PROVENANCE.md` - Source attribution, GPL, fixture, dependency-license, and release-review policy.
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `tools/flash/src/main.rs` already supports `redact-evidence=true`, commit-redacted serial logs, and command evidence JSON.
- `scripts/phase21-live-mining-evidence.sh` already wraps mining-smoke and bounded-soak style runs, pool credential inputs, API/WebSocket capture, redaction, and safe-stop markers.
- `scripts/phase21-live-mining-evidence-test.sh` already verifies that selected pool credential and API body values do not survive in redacted outputs.
- `scripts/phase21-pool-credentials-json.mjs` and `scripts/phase21-pool-input-bridge.sh` already keep local pool JSON as runtime input and emit redacted bridge/status outputs.
- `scripts/phase17-live-http-api-smoke.sh` and `scripts/phase17-websocket-capture.mjs` provide mature API/WebSocket capture and redaction patterns.
- `tools/parity/src/mining_allow.rs` already checks detector, board-info, package identity, prohibited command tokens, abort conditions, safe-state markers, live-pool/soak categories, and redaction reviewer presence.
- `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/redaction-review.md`, Phase 21 redaction review, and Phase 22 redaction review provide review templates to reuse.
- `firmware/bitaxe/src/controlled_mining_runtime.rs` already emits redacted lifecycle/status markers instead of raw Stratum or share data.

### Established Patterns

- Evidence artifacts should name board `205`, port category, source commit, reference commit, package or firmware identity, exact command category, observed behavior, redaction status, safe-state markers, and conclusion.
- Later evidence tiers stop when earlier detector, package, redaction, prerequisite, or safe-state gates are missing. Blocked artifacts are preferred over bypassing guards.
- Committed evidence records category labels and exact non-claims, not secrets or private runtime values.
- Pure claim, redaction, evidence-root, and validation decisions should be testable without hardware. Hardware, serial, HTTP, WebSocket, NVS, and filesystem capture remain thin shell/adapters.
- GSD artifacts and other frontmatter-parsed Markdown must not use standalone body `---` separators after the frontmatter.

### Integration Points

- Add Phase 23 planning artifacts under `.planning/phases/23-redacted-operator-evidence-workflow/`.
- Add Phase 23 committed evidence under `docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/` when implementation produces docs, contracts, reviews, and summaries.
- Extend Phase 21 scripts or create Phase 23 wrappers for a unified evidence root, but keep hardware-capable operations behind `just detect-ultra205` and repo-owned command validation.
- Extend tests for redaction categories not fully covered today: Stratum target, extranonce, share payload, socket error, NVS/settings, device URL, IP, MAC, Wi-Fi, pool endpoint, port, worker, owner address, password, and token.
- Update `docs/parity/checklist.md` only after Phase 23 evidence can support exact EVD/STR/REL/CFG claims without overclaiming Phase 24/25 behavior.
</code_context>

<specifics>
## Specific Ideas

- Preferred artifact root: `docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/`.
- Preferred evidence slots: `package`, `detector`, `board-info`, `command`, `log`, `api`, `websocket`, `share-outcome`, `safe-stop`, `redaction-review`, and `conclusion`.
- Preferred blocked slot style: each unsupported later-phase slot should say why it is pending, which later phase owns it, and which claim is not being made.
- Preferred redaction fixture style: use synthetic representative values only, never real local credentials, owner addresses, device URLs, Wi-Fi values, tokens, endpoints, or NVS secrets.
- Preferred final gate: run targeted tests for changed helpers, `just test` or narrower repo-owned checks when appropriate, `just parity`, `just verify-reference`, redaction scan/review, reference cleanliness, and lifecycle validation.
</specifics>

<deferred>
## Deferred Ideas

- Trusted BM1366 production initialization, pool-derived work dispatch, live result parsing, stale-work invalidation, and fail-closed ASIC production errors belong to Phase 24.
- Real Stratum v1 socket lifecycle, deterministic fake-pool production tests, live accepted/rejected share outcome, watchdog under bounded production load, and bounded safe-stop runtime proof belong to Phase 25.
- API/WebSocket/statistics/scoreboard promotion and final v1.1 parity closure belong to Phase 26.
- Full active voltage/fan/thermal fault-stimulus, self-test hardware closure, OTA/recovery destructive or fault-injection evidence, runtime display/input, BAP, Stratum v2, non-205 boards, other ASIC families, and unbounded stress mining remain future work.
</deferred>

*Phase: 23-redacted-operator-evidence-workflow*
*Context gathered: 2026-07-04*
