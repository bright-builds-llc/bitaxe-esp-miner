---
quick_id: 260720-wfa
phase: quick
plan: 260720-wfa
type: execute
wave: 1
depends_on: []
mode: quick-full
status: planned
created_at: "2026-07-20T00:00:00Z"
autonomous: true
requirements: []
files_modified:
  - docs/hardware/hardware-attempt-policy.md
  - AGENTS.md
  - .codex/tasks/lessons.md
  - BUILD.bazel
  - scripts/BUILD.bazel
  - scripts/hardware-attempt-policy-contract-test.sh
  - .planning/phases/35-detector-gated-correlated-evidence-and-exact-parity-promotion/35-CONTEXT.md
  - .planning/phases/35-detector-gated-correlated-evidence-and-exact-parity-promotion/35-04-PLAN.md
  - .planning/phases/35-detector-gated-correlated-evidence-and-exact-parity-promotion/35-VALIDATION.md
  - .planning/STATE.md
must_haves:
  truths:
    - "One canonical repository policy governs progress-gated hardware repair for all current and future repository hardware work, with concise AGENTS guidance outside the managed block."
    - "The stable outcomes are exactly continue_after_verified_fix, continue_after_manual_remediation, complete, stop_repeated_boundary, stop_hardware_blocker, stop_authority_boundary, and stop_impossible_contract."
    - "There is no fixed attempt cap and no unchanged blind retry; the same typed failure recurring once after its targeted verified fix selects stop_repeated_boundary."
    - "Every continuation uses a fresh ordinal, protected parent, nonexistent supervisor child, private sibling logs, immutable root, and exact-current-HEAD package/preflight."
    - "Hardware remains phase-gated through repo-owned commands with detection, allowed effects, safety, recovery, evidence, and tests; UART/pins, archived Phase 28.1.1, privacy, evidence, and Phase 30 boundaries remain unchanged."
    - "Agent-selected fault testing requires plan- and command-encoded repo/vendor-safe limits, automatic abort, recovery, and evidence; electrical overstress is prohibited."
    - "Phase 35 preserves attempts 1-12, authorizes fresh attempt 13, and allows later attempts only after a positive progress decision; each fresh attempt invokes the full hardware command exactly once."
    - "The executor may run the inert preflight-only flash CLI/package-admission path, but never detector, credentials, device interaction, effectful flash, monitor, device HTTP/PATCH/reboot, evidence admission, promotion, or other hardware effects."
    - "Executor commits contain only policy/guidance/lesson or contract-test/build wiring; the orchestrator alone commits STATE, Phase 35 planning synchronization, PLAN, SUMMARY, and VERIFICATION artifacts."
  artifacts:
    - path: docs/hardware/hardware-attempt-policy.md
      provides: "Canonical closed progress-gated hardware-attempt policy"
    - path: AGENTS.md
      provides: "Concise repo-local policy pointer outside managed Bright Builds text"
    - path: .codex/tasks/lessons.md
      provides: "One stable append-only four-field retry lesson"
    - path: scripts/hardware-attempt-policy-contract-test.sh
      provides: "Hermetic policy and AGENTS placement regression"
    - path: .planning/phases/35-detector-gated-correlated-evidence-and-exact-parity-promotion/35-04-PLAN.md
      provides: "Attempt-13-first execution contract with one invocation per fresh attempt"
    - path: .planning/STATE.md
      provides: "Truthful attempt-12 history and next authorized progress-gated action"
  key_links:
    - from: AGENTS.md
      to: docs/hardware/hardware-attempt-policy.md
      via: "unmanaged repo-local guidance"
      pattern: "hardware-attempt-policy"
    - from: .planning/phases/35-detector-gated-correlated-evidence-and-exact-parity-promotion/35-04-PLAN.md
      to: docs/hardware/hardware-attempt-policy.md
      via: "closed continuation and stop vocabulary"
      pattern: "continue_after_verified_fix|stop_repeated_boundary"
    - from: scripts/hardware-attempt-policy-contract-test.sh
      to: docs/hardware/hardware-attempt-policy.md
      via: "Bazel-run exact vocabulary and invariant checks"
      pattern: "stop_authority_boundary|stop_impossible_contract"
---

# Quick Task 260720-wfa: Persist a Progress-Gated Hardware Repair Loop

## Goal

Persist one repository-wide hardware-attempt policy, guard it with a hermetic contract test, synchronize Phase 35 for attempt 13 and policy-gated later attempts, and prove all software gates without device or effectful hardware interaction.

## Context

Read `.planning/STATE.md`, `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md`, the relevant local-guidance/verification/testing/Rust standards, `docs/parity/evidence-policy.md`, Phase 35 `35-CONTEXT.md`, `35-04-PLAN.md`, `35-VALIDATION.md`, `35-HARDWARE-EVIDENCE.md`, and both active lesson ledgers before editing. Canonical detail belongs in a repo-owned document; AGENTS stays concise; managed/global policy stays untouched; lessons remain append-only; and every implementation commit runs the ordered Rust pre-commit gate.

## Tasks

<tasks>

<task type="auto">
  <name>Task 1: Persist the canonical policy, concise guidance, and lesson</name>
  <files>docs/hardware/hardware-attempt-policy.md, AGENTS.md, .codex/tasks/lessons.md</files>
  <action>Create `docs/hardware/hardware-attempt-policy.md` as the canonical policy for all current/future repository hardware attempts. Define exactly the seven locked outcomes. Require exact-current-HEAD software/preflight admission; a fresh ordinal, mode-0700 parent, nonexistent supervisor-owned child, distinct mode-0600 sibling logs, and immutable root; one phase-gated repo-owned command invocation per fresh attempt; earliest typed-failure precedence; and exactly one outcome before continuation.

Specify no fixed cap and no unchanged blind retry. `continue_after_verified_fix` requires diagnosis, one targeted fix, real-boundary regression, complete software gates, and fresh exact-HEAD/package identity. `continue_after_manual_remediation` requires an authorized non-invasive remediation plus objective proof that the boundary changed. One recurrence of the same typed failure after its targeted verified fix selects `stop_repeated_boundary`. Define `complete` only for genuine phase success; use the other three stops for unresolved hardware, authority, or impossible-contract boundaries without weakening truth.

Require active phase plans and repo-owned commands to encode detection, effects, safety, recovery, evidence, and tests. Preserve direct-UART/pin authorization, archived Phase 28.1.1 closure, Phase 30 non-promotion, and `docs/parity/evidence-policy.md`. Allow fault testing only with repo/vendor-safe limits, automatic abort, recovery, and evidence encoded in both plan and command; prohibit electrical overstress.

Add a concise AGENTS subsection outside the managed block linking the policy and summarizing the closed outcomes, progress requirement, repeated-boundary stop, fresh-attempt invariants, and unchanged authority/privacy boundaries. Append one stable four-field repo lesson about blind retries without new information. Do not edit managed Bright Builds content, standards, global lessons, evidence policy, Phase 35 artifacts, STATE, PLAN, SUMMARY, or hardware evidence.

Run Markdown check mode on the new canonical policy. The unchanged base versions of `AGENTS.md` and `.codex/tasks/lessons.md` are not bare-`mdformat` clean, so validate their localized edits with managed-block byte comparison, append-only/four-field lesson checks, scoped diff review, redaction, reference, parity, Phase 30, lifecycle, and diff checks rather than rewriting pre-existing content. Immediately before committing, run in order: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, `cargo test --all-features`. Commit only these three files as `docs(quick-260720-wfa): define progress-gated hardware attempts`; do not push.</action>
  <verify>
    <automated>mdformat --check docs/hardware/hardware-attempt-policy.md &amp;&amp; just verify-redaction &amp;&amp; just verify-reference &amp;&amp; just parity &amp;&amp; bazel test //scripts:phase30_no_promotion_contract_test &amp;&amp; node "$HOME/.codex/get-shit-done/bin/gsd-tools.cjs" verify lifecycle 35 --expect-id 35-2026-07-17T17-00-37 --expect-mode yolo --require-plans --raw &amp;&amp; diff &lt;(sed -n '/&lt;!-- bright-builds-rules-managed:begin --&gt;/,/&lt;!-- bright-builds-rules-managed:end --&gt;/p' AGENTS.md) &lt;(git show HEAD:AGENTS.md | sed -n '/&lt;!-- bright-builds-rules-managed:begin --&gt;/,/&lt;!-- bright-builds-rules-managed:end --&gt;/p') &amp;&amp; git diff --check &amp;&amp; cargo fmt --all &amp;&amp; cargo clippy --all-targets --all-features -- -D warnings &amp;&amp; cargo build --all-targets --all-features &amp;&amp; cargo test --all-features</automated>
  </verify>
  <done>The canonical policy, concise unmanaged guidance, and one repo lesson exist in one scoped implementation commit; all managed/global/planning/evidence surfaces remain uncommitted and unchanged.</done>
</task>

<task type="auto">
  <name>Task 2: Add the hermetic policy contract regression</name>
  <files>scripts/hardware-attempt-policy-contract-test.sh, scripts/BUILD.bazel, BUILD.bazel</files>
  <action>Create a rerunnable non-echoing Bash `sh_test` using Bazel runfiles and stable failure categories. Assert all seven tokens occur exactly once in the canonical policy; no fixed numeric cap or unchanged retry is allowed; repeated same-boundary-after-fix stops; fresh ordinal/parent/absent child/sibling logs/immutable root/exact HEAD are required; phase commands own detection/effects/safety/recovery/evidence/tests; UART/pins, archived lineage, Phase 30, and evidence privacy remain closed; fault testing has safe limits/abort/recovery/evidence; and electrical overstress is forbidden. Prove the AGENTS pointer occurs after the managed block and remains concise. Export only required policy/AGENTS sources from root `BUILD.bazel`, add `//scripts:hardware_attempt_policy_contract_test`, and preserve existing targets.

Run Bash syntax, `shfmt -d`, `shellcheck`, the new target, Phase 30, redaction, reference, parity, lifecycle, and diff checks. Immediately before committing, repeat the mandatory Rust sequence in exact order. Commit only these three files as `test(quick-260720-wfa): enforce hardware attempt progress`; do not stage or commit STATE, Phase 35 planning, any PLAN/SUMMARY, or push.</action>
  <verify>
    <automated>bash -n scripts/hardware-attempt-policy-contract-test.sh &amp;&amp; shfmt -d scripts/hardware-attempt-policy-contract-test.sh &amp;&amp; shellcheck scripts/hardware-attempt-policy-contract-test.sh &amp;&amp; bazel test //scripts:hardware_attempt_policy_contract_test //scripts:phase30_no_promotion_contract_test &amp;&amp; just verify-redaction &amp;&amp; just verify-reference &amp;&amp; just parity &amp;&amp; node "$HOME/.codex/get-shit-done/bin/gsd-tools.cjs" verify lifecycle 35 --expect-id 35-2026-07-17T17-00-37 --expect-mode yolo --require-plans --raw &amp;&amp; git diff --check &amp;&amp; cargo fmt --all &amp;&amp; cargo clippy --all-targets --all-features -- -D warnings &amp;&amp; cargo build --all-targets --all-features &amp;&amp; cargo test --all-features</automated>
  </verify>
  <done>The hermetic policy contract is green in a second scoped implementation commit, with no planning artifact, STATE, SUMMARY, hardware, or push included.</done>
</task>

<task type="auto">
  <name>Task 3: Prove the software-only boundary and hand off artifacts</name>
  <files>docs/hardware/hardware-attempt-policy.md, AGENTS.md, .codex/tasks/lessons.md, scripts/hardware-attempt-policy-contract-test.sh, scripts/BUILD.bazel, BUILD.bazel</files>
  <action>Run the complete policy, Phase 35 HTTP/correlated/promotion, Phase 30, flash/parity, Markdown, redaction, reference, lifecycle, and diff gates. After the two implementation commits, run `just phase35-evidence preflight-only=true` with no credential, root, port, device, or hardware-mode argument. This explicitly permits the inert firmware-build and flash-CLI/package-admission dry-run path. Capture output in a mode-0600 temporary file, require the established public markers `status=preflight_passed` and `current_head_equal=true`, and delete it without echoing content. The existing `test_preflight_has_no_detector_or_effects` regression and private preflight-seal contract must prove the path exits before detector, credential resolution, serial/device access, effectful flash/monitor, HTTP/PATCH/reboot, admission, or promotion; do not alter production marker semantics merely for this quick task.

Review the two implementation commit subjects and changed-file scopes. Create the normal quick SUMMARY as an uncommitted artifact/handoff, recording the inert preflight result and zero-effect boundary. Do not commit STATE, any PLAN, SUMMARY, or VERIFICATION; do not modify Phase 35 planning in an implementation commit; do not run attempt 13; and do not push.</action>
  <verify>
    <automated>bazel test //scripts:hardware_attempt_policy_contract_test //scripts:phase33_confirmed_settings_durability_test //scripts:phase35_http_boundary_read_test //scripts:phase35_correlated_evidence_test //scripts:phase35_promotion_contract_test //scripts:phase30_no_promotion_contract_test //scripts:phase29_doc_redaction_check_test //tools/flash:tests //tools/parity:tests &amp;&amp; mdformat --check docs/hardware/hardware-attempt-policy.md &amp;&amp; just verify-redaction &amp;&amp; just verify-reference &amp;&amp; just parity &amp;&amp; node "$HOME/.codex/get-shit-done/bin/gsd-tools.cjs" verify lifecycle 35 --expect-id 35-2026-07-17T17-00-37 --expect-mode yolo --require-plans --raw &amp;&amp; git diff --check &amp;&amp; bash -c 'set -euo pipefail; output="$(mktemp)"; trap '\''rm -f "$output"'\'' EXIT; chmod 600 "$output"; just phase35-evidence preflight-only=true &gt;"$output" 2&gt;&amp;1; rg -q -F "status=preflight_passed" "$output"; rg -q -F "current_head_equal=true" "$output"'</automated>
  </verify>
  <done>All software gates pass, the inert preflight reports exact-current-HEAD success and the existing regression proves it exits before effects, exactly two implementation commits exist, and the executor hands uncommitted artifacts to the orchestrator without hardware or push.</done>
</task>

</tasks>

## Orchestrator-Owned Final Docs and Artifact Commit

After executor completion, the orchestrator first synchronizes Phase 35 and `.planning/STATE.md` in the working tree while preserving attempts 1-12, `35-HARDWARE-EVIDENCE.md`, checklist/evidence truth, and the absence of `35-04-SUMMARY.md`. Only after those uncommitted final-state edits exist may the independent verifier inspect the plan must-haves and create `260720-wfa-VERIFICATION.md`. The orchestrator then appends the normal verified quick-task row to STATE and commits the complete planning/artifact set together.

In `35-04-PLAN.md`, update every operative single-attempt statement: frontmatter truth `One final run`, objective `Run the single final`, purpose/output language, attempt policy, task names/actions, verification bullets, and success wording. Search all four synchronized files for `one final`, `single final`, `single.*run`, `sole authorized`, `attempt 12`, `no attempt beyond 12`, and per-attempt authorization wording; retain historical attempt records but remove stale operative caps. The new contract makes attempt 13 first, later attempts conditional on `continue_after_verified_fix` or `continue_after_manual_remediation`, and each fresh attempt still invokes the full hardware command exactly once. Only `complete` unlocks Task 3 or `35-04-SUMMARY.md`.

Update STATE current position, decisions, blockers, session, and quick-task history truthfully: this quick task authorized but did not execute attempt 13. Update VALIDATION with the policy target and per-fresh-attempt semantics; keep real hardware validation pending. Run Markdown/frontmatter checks, explicit stale-phrase searches, plan structure, policy/Phase 35/Phase 30 tests, redaction, reference, parity, lifecycle, diff, and the ordered Rust pre-commit sequence.

Commit exactly these seven artifacts in the final orchestrator docs commit:

- `.planning/quick/260720-wfa-persist-a-repository-wide-progress-gated/260720-wfa-PLAN.md`
- `.planning/quick/260720-wfa-persist-a-repository-wide-progress-gated/260720-wfa-SUMMARY.md`
- `.planning/quick/260720-wfa-persist-a-repository-wide-progress-gated/260720-wfa-VERIFICATION.md`
- `.planning/STATE.md`
- `.planning/phases/35-detector-gated-correlated-evidence-and-exact-parity-promotion/35-CONTEXT.md`
- `.planning/phases/35-detector-gated-correlated-evidence-and-exact-parity-promotion/35-04-PLAN.md`
- `.planning/phases/35-detector-gated-correlated-evidence-and-exact-parity-promotion/35-VALIDATION.md`

No executor implementation file belongs in this commit; do not push.

## Completion Boundary

The quick task permits only deterministic software checks plus the inert preflight-only build/flash-CLI/package-admission path that proves `effects_permitted=false`. It prohibits detector, credentials, USB/serial/device interaction, effectful flash or monitor, device HTTP/PATCH/reboot, evidence admission/promotion, attempt 13, direct UART/pins, archived work, electrical/fault action, `35-04-SUMMARY.md`, and push.
