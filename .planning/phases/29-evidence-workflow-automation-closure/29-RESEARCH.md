---
generated_by: gsd-phase-researcher
lifecycle_mode: yolo
phase_lifecycle_id: 29-2026-07-13T00-19-45
generated_at: 2026-07-13T00:30:54.599Z
---

# Phase 29: Evidence Workflow Automation Closure - Research

## Research Summary

Phase 29 should standardize evidence roots around the existing Phase 23 eleven-slot contract, make phase identity explicit, and centralize terminal validation without changing hardware behavior. The safest implementation is a typed Rust evidence-root core consumed by thin shell wrappers: one schema describes required slots and phase-specific outcome rules, one builder emits deterministic missing/blocked slots or Phase 28 cross-links, and the existing validator consumes the same schema.

The main integration risk is not generating files; it is accidentally letting generated placeholders, path heuristics, or a passing validator convert missing evidence into a success claim. Every generated slot therefore needs a typed disposition, stable blocker reason, explicit provenance, and exact non-claims.

## Current State

### Canonical contract

- `docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/evidence-contract.md` defines eleven required slots: package, detector, board-info, command, log, API, WebSocket, share-outcome, redaction-review, safe-stop, and conclusion.
- `scripts/phase23-redacted-operator-evidence.sh` already creates that inventory and finishes with `operator-evidence --evidence-root ... --require-redaction-passed`.
- `tools/parity/src/operator_evidence.rs` owns slot loading, metadata checks, redaction enforcement, stale-target prohibitions, forbidden sentinel checks, and Phase 28 consolidation checks.
- `tools/parity/src/main.rs` exposes the `operator-evidence` command and parity regression surface.

### Phase 25 gap

- `scripts/phase25-live-stratum-evidence.sh` creates most Phase 23-compatible files but omits `conclusion.md` in its normal shape.
- Detector failure produces a partial inventory rather than a complete blocked root.
- Terminal branches invoke `mining-allow` but do not invoke strict operator-evidence validation.
- The wrapper test does not currently prove validator ordering, exactly-once invocation, or operator-validation failure propagation.

### Phase 27 gap

- `scripts/phase27-live-hardware-bridge-evidence.sh` emits detector, board-info, command, share-outcome, redaction-review, conclusion, and summary artifacts.
- Package, log, API, WebSocket, and safe-stop slots are absent from the phase-native root.
- Terminal branches invoke `mining-allow` without strict operator-evidence validation.
- The existing conclusion/outcome vocabulary is not yet a complete explicit operator-evidence profile.

### Phase 28 gap

- `docs/parity/evidence/phase-28-hardware-evidence-and-checklist-promotion/` is a manually consolidated full root and demonstrates cross-link-only promotion.
- `.planning/phases/28-hardware-evidence-and-checklist-promotion/28-01-PLAN.md` documents the prior manual consolidation algorithm and source-token preservation rules.
- No `just phase28-evidence` alias or Bazel-owned consolidation wrapper exists.
- Current operator-evidence logic recognizes Phase 28 through path/content heuristics and assumes a blocked share path in places; automation must support `accepted`, `rejected`, and `blocked_safe_prerequisite` without weakening their proof requirements.

## Recommended Architecture

### Typed evidence-root core

Add explicit types for workflow profile, slot kind, slot disposition, and closed share outcome. The profile must be supplied by the caller rather than inferred from an operator-controlled path. A single descriptor should define:

- the eleven required slot names;
- phase-specific mandatory metadata and allowed dispositions;
- allowed outcome tokens and supporting-slot requirements;
- stable blocked/deferred reason categories;
- generated versus observed provenance requirements;
- redaction-review requirements.

Use the descriptor from both builder and validator paths. This prevents a generator from emitting a shape that a separately maintained validator interprets differently.

### Thin wrapper finalization

Refactor Phase 25 and Phase 27 into one top-level terminal finalizer per script. After argument validation and workflow setup, terminal paths should accumulate status rather than exit before finalization.

The finalizer order is:

1. Complete all eleven slots using observed data or typed blocked/deferred entries.
2. Write and validate the redaction review.
3. Run `mining-allow` when a valid manifest/root makes it applicable.
4. Run `operator-evidence --require-redaction-passed` exactly once and last.
5. Return nonzero when any workflow or validator status failed.

Do not use an `EXIT` trap. Trap recursion, `set -e` behavior, and partial-root validation would make original-status preservation difficult to prove.

### Atomic Phase 28 consolidation

Add a repo-owned Phase 28 command with explicit `--phase27-root` and `--evidence-root` arguments. Reject equal or nested roots. Read only committed, allowlisted category fields from Phase 27 artifacts.

Generate a complete root into a sibling staging directory. Source-backed files contain stable relative cross-links and category labels only. A source slot absent by the Phase 27 contract becomes an explicit blocked/deferred slot; it is never fabricated as observed evidence.

Validate staging with redaction checks and strict operator-evidence validation before replacing the generator-owned destination atomically. If generation or validation fails, leave the previous valid destination unchanged. Unknown files in a managed destination fail closed so the generator does not delete operator-owned material silently.

For deterministic reruns, avoid timestamps, temporary paths, current-process identifiers, and unrelated current-HEAD values in generated content. Identical source artifacts should produce byte-identical output.

## Outcome and Claim Rules

- Preserve the Phase 27 closed share-outcome set: `accepted`, `rejected`, and `blocked_safe_prerequisite`.
- `accepted` or `rejected` is valid only when the root contains the exact required ASIC-correlation and safe-stop support.
- `blocked_safe_prerequisite` must retain the blocker and inherited non-claims.
- A generated slot must declare its generated/cross-linked provenance and cannot satisfy an observation requirement by itself.
- Phase 29 must not change Phase 30 promotion status or infer new verified claims.
- `just parity` and existing overclaim guards remain an independent final check.

## Build and Integration Surfaces

- `tools/parity/src/operator_evidence.rs`: typed profiles, shared descriptor, deterministic builder/validator rules, and unit tests.
- `tools/parity/src/main.rs`: explicit profile/consolidation CLI integration and regression tests.
- `scripts/phase25-live-stratum-evidence.sh`: complete-root terminal finalizer.
- `scripts/phase25-live-stratum-evidence-test.sh`: Phase 25 ordering and failure tests.
- `scripts/phase27-live-hardware-bridge-evidence.sh`: complete-root terminal finalizer.
- `scripts/phase27-live-hardware-bridge-evidence-test.sh`: Phase 27 ordering and failure tests.
- New Phase 28 wrapper and test under `scripts/`.
- `scripts/BUILD.bazel`: Phase 28 wrapper/test targets and changed dependencies.
- `Justfile`: `phase28-evidence` alias.
- `docs/release/ultra-205.md`: automatic Phase 25/27 validation and Phase 28 operator command.
- `docs/parity/checklist.md`: expected to remain semantically unchanged unless a breadcrumb is required; no status promotion belongs here.

## Planning Recommendation

Use three sequential plans:

1. Typed operator-evidence profiles, shared schema, builder/consolidation core, and Rust tests.
2. Phase 25/27 terminal finalizers, Phase 28 wrapper, Bazel/Just integration, and deterministic shell regressions.
3. Operator documentation, parity/redaction regression closure, lifecycle verification, and full repo verification.

Plan 2 depends on the typed contract from Plan 1. Plan 3 depends on the completed operator commands from Plan 2. This order prevents shell code from inventing a second schema and gives documentation exact tested commands.

## Validation Architecture

### Test layers

| Layer | Target behavior | Primary checks |
| --- | --- | --- |
| Pure Rust unit tests | Profile parsing, eleven-slot schema, disposition legality, outcome support, deterministic generated content, root relationship guards | `cargo test -p bitaxe-parity --all-features` or the package name exposed by `tools/parity/Cargo.toml`; Bazel parity tests |
| Parity CLI tests | Explicit profile selection, strict redaction flag, accepted/rejected/blocked validation, overclaim rejection | `bazel test //tools/parity:parity_tests` and repo-equivalent target discovered from `tools/parity/BUILD.bazel` |
| Wrapper regression tests | Complete roots, exact invocation order/count, failure precedence, blocked detector path, atomic destination preservation | Phase 25, Phase 27, and new Phase 28 script test targets in `scripts/BUILD.bazel` |
| Operator surface | `just phase28-evidence` routes through Bazel and docs show exact fail-closed flow | Targeted Just/Bazel command inspection plus wrapper tests |
| Repository gate | Reference remains clean, parity guard passes valid checklist and rejects fixtures with unsupported promotion | `just verify-reference`, `just parity`, and targeted negative fixtures |

### Required regression cases

- Phase 25 blocked mode writes eleven slots and invokes operator validation exactly once after any applicable mining-allow check.
- Phase 27 blocked mode writes eleven slots and invokes operator validation exactly once after mining-allow.
- Detector failure writes a complete blocked root, runs operator validation, and still returns nonzero.
- Operator validation failure returns nonzero even when prior workflow work succeeded.
- Prior workflow failure remains nonzero even when both validators pass.
- Phase 28 rejects equal/nested roots, missing mandatory source artifacts, contradictory categories, unknown outcome tokens, and unknown destination files.
- Phase 28 accepted, rejected, and blocked inputs preserve exact outcomes and required non-claims.
- A failed Phase 28 generation leaves the previous valid destination byte-for-byte unchanged.
- Two runs with identical inputs produce byte-identical generated files.
- Redaction tests reject forbidden pool, credential, device, network, Wi-Fi, NVS, Stratum, share-payload, and raw-frame categories.
- Existing overbroad-promotion negative tests remain failing and `just parity` remains passing for the committed checklist.

### Phase completion gate

Before any implementation commit, follow the repo-required Rust sequence exactly:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`

Then run the affected Bazel script/parity targets, `just parity`, `just verify-reference`, Markdown/shell formatter checks when available, `git diff --check`, and GSD lifecycle validation. Hardware is not required to prove Phase 29's deterministic workflow automation; if any hardware path is exercised, it must follow the detector gate, board-205 scope, runtime-only credential handling, 360-second capture minimum, and redacted evidence rules in `AGENTS.md`.

## Research Conclusion

Phase 29 is plan-ready. The architecture should make phase identity and evidence disposition explicit, keep one schema for generation and validation, force one terminal validation path in each wrapper, and make Phase 28 consolidation deterministic and atomic. No external dependency is needed; the existing Rust, shell, Bazel, Just, and parity surfaces are sufficient.
