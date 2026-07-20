---
quick_id: 260720-jwt
description: "Implement repository-wide private-first evidence policy, additive dual-artifact flash capture, Phase 35 classifier-input repair, staged/CI redaction guard, documentation, and software-only verification; no hardware or push"
mode: quick-full
status: planned
created: 2026-07-20
must_haves:
  truths:
    - "NeverPersistRaw covers passwords, tokens, credential contents, pool URLs/ports/users/workers/addresses/passwords, and NVS secrets; none reach disk, terminal output, Git, or promoted evidence."
    - "ProtectedOperational covers SSIDs, IP/MAC addresses, hostnames, device origins, USB identities/paths, PIDs, process paths, unredacted commands, settings, HTTP material, and detailed logs; these exist only in mode-0600 files under ignored mode-0700 roots."
    - "ShareableFact covers typed categories, booleans, bounded counts/durations, status classes, board categories, and outcomes; PublicProvenance covers source/reference commits and safely opaque package/artifact/evidence-root digests, never hashes of low-entropy sensitive values."
    - "Artifact lifecycle is ActivePrivate to either SealedNonPromotion or SealedEligible, then optionally AdmittedProjection or ExplicitlyPurged; process/resource cleanup never implies artifact deletion."
    - "Private classifiers consume an immutable secret-sanitized artifact before a distinct commit-redacted artifact is derived, so shareable redaction cannot destroy classifier inputs."
    - "The existing flash defaults, --redact-evidence behavior, and flash-monitor.log compatibility path remain valid while Phase 35 opts into an additive dual evidence mode."
    - "One non-echoing repository command enforces redaction against staged or CI diffs and the complete admitted-evidence tree, with only reviewed registry exceptions."
    - "Attempts 1-11 remain immutable, no hardware or credential access occurs, and attempt 12 requires a fresh authorization after exact-head preflight."
  artifacts:
    - path: "docs/parity/evidence-policy.md"
      provides: "Canonical data-class, sink, lifecycle, cleanup, admission, and exception policy"
    - path: "tools/flash/src/evidence.rs"
      provides: "Typed evidence modes, incremental secret sanitization, protected output admission, and dual-artifact derivation"
    - path: "tools/flash/src/main.rs"
      provides: "Additive --evidence-mode dual CLI and flash-monitor orchestration"
    - path: "scripts/verify-redaction.sh"
      provides: "Shared staged/CI diff and admitted-tree redaction adapter"
    - path: ".github/workflows/evidence-redaction.yml"
      provides: "Pull-request and push enforcement using the repository command"
    - path: "scripts/phase35-correlated-evidence-effects.sh"
      provides: "Phase 35 dual-mode invocation and private classifier-input routing"
  key_links:
    - from: "tools/flash/src/main.rs"
      to: "tools/flash/src/evidence.rs"
      via: "typed evidence mode and pre-disk sanitizer"
    - from: "scripts/phase35-correlated-evidence-effects.sh"
      to: "flash-monitor.classifier-input.log"
      via: "Phase 33 baseline classification before mutation"
    - from: "Justfile"
      to: "scripts/verify-redaction.sh"
      via: "just verify-redaction"
    - from: ".github/workflows/evidence-redaction.yml"
      to: "just verify-redaction"
      via: "base/head diff arguments"
---

# Quick Task 260720-jwt Plan

The user explicitly selected one scoped `gsd-quick --validate` workflow for this
cross-cutting repair. The task therefore remains one validated quick task with
three atomic commits; it does not split into roadmap phases, rewrite historical
evidence, migrate unrelated dormant pipelines, add purge tooling, run hardware,
or push.

## Task 1: Persist the repository-wide evidence contract and enforcement guard

**Files:** `docs/parity/evidence-policy.md`, `AGENTS.md`, `.codex/tasks/lessons.md`, `.codex/tasks/todo.md`, `scripts/verify-redaction.sh`, `scripts/verify-redaction-test.sh`, `scripts/redaction-exceptions.tsv`, `scripts/BUILD.bazel`, `Justfile`, `.github/workflows/evidence-redaction.yml`

**Action:**
- Define the four locked data classes and artifact lifecycle states, protected-root/file requirements, console and Git sinks, resource-cleanup versus artifact-purge distinction, retained sealed-root policy, and reviewed exception-registry contract. Keep the canonical details in the policy document and add only concise repo-local enforcement to `AGENTS.md` outside the managed block.
- Add a non-echoing `just verify-redaction` adapter that scans staged changes by default or an explicit base/head range in CI, plus the complete admitted-evidence tree. Enforce never-persist rules repository-wide and protected-operational rules in shareable sinks. Report only rule ID, category, path, and line. Allow only stable-ID, reason, exact path/category, optional-expiry registry entries; provide no inline or CLI bypass.
- Add hermetic scanner tests for staged and CI diffs, renames, synthetic fixtures, reviewed exceptions, admitted-tree leaks, malformed configuration, and non-echoing output. Wire Bazel and a dedicated pull-request/push workflow to the same command.
- Update only the existing redaction lesson and pending todo blocks as necessary; do not rewrite unrelated task/lesson history.
- Run the exact verification below, then immediately rerun the ordered mandatory Rust sequence as the final pre-commit action and commit atomically.

**Verify:** `bash -n scripts/verify-redaction.sh scripts/verify-redaction-test.sh`; `shfmt -d scripts/verify-redaction.sh scripts/verify-redaction-test.sh`; `shellcheck scripts/verify-redaction.sh scripts/verify-redaction-test.sh`; `bazel test //scripts:verify_redaction_test`; `just verify-redaction`; `git diff --check`; then immediately before the Task 1 commit run, in order, `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, `cargo test --all-features`.

**Done:** The durable policy is authoritative, agents and contributors receive concise enforcement guidance, and one tested non-echoing command plus CI prevents forbidden staged/promoted evidence without weakening private development artifacts.

## Task 2: Implement additive dual-artifact flash capture

**Files:** `tools/flash/src/main.rs`, `tools/flash/src/evidence.rs`, `tools/flash/BUILD.bazel`

**Action:**
- Add `--evidence-mode dual` while preserving the default developer mode and legacy `--redact-evidence`; reject conflicting mode arguments.
- Capture child stdout/stderr through stream-identified pipes with independent incremental bounded sanitizer state before any disk write. Carry incomplete lines across chunks without cross-stream joining, reject invalid UTF-8/overlong/malformed state as `evidence_sanitization_invalid`, and emit no raw child output to the terminal.
- In dual mode, require canonical workspace containment plus `git check-ignore` admission before any flash effect, preflight distinct nonexistent paths, reject aliases/symlinks, create mode-0600 outputs beneath the owned evidence root, and close/hash only `flash-monitor.classifier-input.log` plus its private record. Add a software-only finalizer that accepts the classified digest and only then derives `flash-monitor.log` plus its admitted record while proving the private digest is unchanged.
- Keep the private record non-commit-ready with the private role/path/digest. The finalizer creates the commit-ready admitted record without private paths; classifier failure creates neither admitted output.
- Add focused Rust and real-process tests for every data class, chunk splits, incomplete/invalid/overlong input, secret absence, operational/private retention, output admission, permissions, immutable digests, manifest compatibility, mode conflicts, and earliest-failure behavior. Preserve all legacy mode tests.
- Run the exact verification below, then immediately rerun the ordered mandatory Rust sequence as the final pre-commit action and commit atomically.

**Verify:** `bazel test //tools/flash:tests`; `git diff --check`; then immediately before the Task 2 commit run, in order, `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, `cargo test --all-features`.

**Done:** Dual capture produces only an immutable secret-sanitized classifier input and private record; explicit digest-bound finalization later produces the distinct legacy-path commit projection without exposing secrets or operational identifiers to the terminal or changing legacy callers.

## Task 3: Migrate and software-verify Phase 35

**Files:** `scripts/phase35-correlated-evidence-effects.sh`, `scripts/phase35-correlated-evidence-test.sh`, `.planning/phases/35-detector-gated-correlated-evidence-and-exact-parity-promotion/35-CONTEXT.md`, `.planning/phases/35-detector-gated-correlated-evidence-and-exact-parity-promotion/35-04-PLAN.md`, `.codex/tasks/todo.md`, `.planning/quick/260720-jwt-implement-repository-wide-private-first-/260720-jwt-SUMMARY.md`; the independent verifier owns `.planning/quick/260720-jwt-implement-repository-wide-private-first-/260720-jwt-VERIFICATION.md`

**Action:**
- Make Phase 35 request dual evidence mode, verify the captured private digest, send only `flash-monitor.classifier-input.log` to the unchanged Phase 33 classifier, recheck the digest, invoke the already-built software-only finalizer directly, recheck again, and only then use the preserved `flash-monitor.log` compatibility/admission projection.
- Extend the real-process suite to reproduce attempt 11's early-redaction failure and prove operational runtime origin survives only in the private input, `direct_flash < classifier < finalize_evidence < original_read/PATCH`, classifier failure creates no admitted output, cleanup/restoration remain secondary, and direct built-tool/runfiles behavior remains hermetic without nested `just` or Bazel.
- Append a policy clarification to Phase 35 context and Plan 35-04: attempts 1-11 remain immutable, the next possible run is attempt 12, and exact-head preflight plus fresh authorization are required. Do not create `35-04-SUMMARY.md` or alter evidence truth.
- Mark the existing private-classifier todo complete only after all software checks pass; record remaining active evidence-pipeline migrations as follow-up work without rewriting historical evidence.
- Run the exact verification below. Immediately rerun the ordered mandatory Rust sequence before the Task 3 implementation commit. Create the quick summary without committing it; the orchestrator and independent verifier own the later GSD-artifact commit. Do not detect hardware, access credentials, contact the device, promote evidence, push, or authorize attempt 12.

**Verify:** `bash -n scripts/phase35-correlated-evidence-effects.sh scripts/phase35-correlated-evidence-test.sh`; `shfmt -d scripts/phase35-correlated-evidence-effects.sh scripts/phase35-correlated-evidence-test.sh`; `shellcheck scripts/phase35-correlated-evidence-effects.sh scripts/phase35-correlated-evidence-test.sh`; `bazel test //scripts:phase33_confirmed_settings_durability_test //scripts:phase35_http_boundary_read_test //scripts:phase35_correlated_evidence_test //scripts:phase35_promotion_contract_test //scripts:phase30_no_promotion_contract_test //scripts:phase29_doc_redaction_check_test //tools/flash:tests`; `just verify-reference`; `just parity`; `node "$HOME/.codex/get-shit-done/bin/gsd-tools.cjs" verify lifecycle 35`; `just verify-redaction`; `git diff --check`; then immediately before the Task 3 implementation commit run, in order, `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, `cargo test --all-features`; after the commit run exact-current-head `just phase35-evidence preflight-only=true`.

**Done:** Attempt 11's defect is covered by a hermetic regression, exact-head software gates pass, Phase 35 is prepared for—but does not perform—one separately authorized attempt 12, and all quick-task artifacts truthfully record the result.

## Orchestrator Artifact Commit

After the independent verifier creates
`.planning/quick/260720-jwt-implement-repository-wide-private-first-/260720-jwt-VERIFICATION.md`
and the orchestrator updates `.planning/STATE.md`, stage only the quick PLAN,
SUMMARY, VERIFICATION, and STATE artifacts. Immediately before that final docs
commit run, in order: `cargo fmt --all`,
`cargo clippy --all-targets --all-features -- -D warnings`,
`cargo build --all-targets --all-features`, and
`cargo test --all-features`. Do not push.
