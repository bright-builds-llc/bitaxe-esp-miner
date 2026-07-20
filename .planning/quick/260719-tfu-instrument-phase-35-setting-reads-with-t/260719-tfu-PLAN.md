---
quick_task: 260719-tfu-instrument-phase-35-setting-reads-with-t
quick_id: 260719-tfu
phase: quick
plan: "01"
type: execute
mode: quick-full
wave: 1
depends_on: []
files_modified:
  - tools/parity/src/phase35_http.rs
  - tools/parity/src/phase35_http/tests.rs
  - tools/parity/src/main.rs
  - tools/parity/BUILD.bazel
  - scripts/phase35-http-boundary-read.sh
  - scripts/phase35-http-boundary-read-test.sh
  - scripts/phase35-correlated-evidence.sh
  - scripts/phase35-correlated-evidence-effects.sh
  - scripts/phase35-correlated-evidence-fixture.sh
  - scripts/phase35-correlated-evidence-test.sh
  - scripts/BUILD.bazel
  - .codex/tasks/todo.md
autonomous: true
generated_by: gsd-plan-phase
lifecycle_mode: direct-fallback
phase_lifecycle_id: quick-260719-tfu
generated_at: 2026-07-20T02:18:44Z
requirements:
  - QUICK-TFU-01
  - QUICK-TFU-02
  - QUICK-TFU-03
  - QUICK-TFU-04
must_haves:
  truths:
    - "Each Phase 35 original, immediate, and restoration setting read performs exactly one direct HTTP/1.1 GET with `--noproxy '*'`, no redirects, a 5-second connect bound, a 10-second total bound, a 65,536-byte response maximum, retries disabled, and no `--fail`; diagnostics never add a probe, retry, HEAD, or second request."
    - "A pure Rust classifier distinguishes the exact ordered terminal categories `tcp_connection_failure`, `tls_handshake_failure` when HTTPS, `request_transmission_incomplete`, `response_status_missing`, `response_headers_missing`, `non_success_response_status`, `response_body_missing`, `response_body_incomplete_or_over_limit`, `invalid_json`, `invalid_hostname_schema`, and `ready`."
    - "Private per-read artifacts are limited to body, headers, stderr, strict write-out metrics, projection, private hostname, and necessary private argv/test records; every file is mode 0600 beneath a mode-0700 directory and no other curl artifact is created."
    - "The exact `phase35-http-boundary-v1` projection contains only its schema version, booleans, bounded counts/durations, response-status class, and one terminal category; it contains no digest or raw origin, host, IP, port, header, body, hostname, curl error, credential, or device identifier."
    - "The first HTTP/supervisor primary remains authoritative through restoration, cleanup, sealing, and reporting; restoration and cleanup details are always secondary-only, and finalization-only failure uses the compatible primary `supervisor_finalization_failed`."
    - "Pure unit tests, a real-process fake-curl matrix, and full Phase 35 supervisor regressions prove every boundary, one-request behavior, runfiles/direct-tool resolution, permissions, redaction, and failure precedence without hardware or network access."
    - "Attempts 1 through 10, Phase 35 evidence truth, parity rows, lifecycle truth, Plan 35-04, and its absent summary remain unchanged; the pending todo is completed only after every software gate and preflight-only command passes."
  artifacts:
    - path: tools/parity/src/phase35_http.rs
      provides: "Pure typed HTTP observation classifier, earliest-boundary precedence, and redacted projection"
    - path: tools/parity/src/phase35_http/tests.rs
      provides: "Exhaustive Arrange/Act/Assert unit matrix for valid and fail-closed observations"
    - path: scripts/phase35-http-boundary-read.sh
      provides: "Exact single-request curl HTTP/1.1 adapter with private body/header/stderr/metrics/projection/hostname artifacts and direct classifier invocation"
    - path: scripts/phase35-http-boundary-read-test.sh
      provides: "Real-process fake-curl boundary, permission, redaction, and no-retry regressions"
    - path: scripts/phase35-correlated-evidence-effects.sh
      provides: "Original, immediate, and restoration read integration plus direct built-tool/runfiles resolution"
    - path: scripts/phase35-correlated-evidence-test.sh
      provides: "Supervisor-level typed precedence, reuse, secondary-failure, and no-nested-runner regressions"
    - path: .codex/tasks/todo.md
      provides: "Localized completion record for task-phase35-redacted-http-boundary-diagnostic after verification"
  key_links:
    - from: scripts/phase35-http-boundary-read.sh
      to: tools/parity/src/phase35_http.rs
      via: "direct classify-phase35-http executable invocation from bazel-bin or opaque Bazel runfiles"
      pattern: "classify-phase35-http|bazel-bin/tools/parity/report|runfiles"
    - from: scripts/phase35-correlated-evidence-effects.sh
      to: scripts/phase35-http-boundary-read.sh
      via: "read_setting delegates original, immediate, and restoration to the same one-request adapter"
      pattern: "read_setting|original|immediate|restoration"
    - from: scripts/phase35-correlated-evidence.sh
      to: scripts/phase35-correlated-evidence-effects.sh
      via: "typed HTTP/supervisor failure becomes the immutable compatible category while finalization records restoration and cleanup only as secondary fields"
      pattern: "primary_failure|secondary|seal_non_promotion"
    - from: scripts/phase35-http-boundary-read-test.sh
      to: scripts/phase35-http-boundary-read.sh
      via: "fresh fake-curl and fake classifier processes exercise the real filesystem and process boundary"
      pattern: "fake-curl|0700|0600"
---

<objective>
Instrument the three Phase 35 setting reads with typed, redacted HTTP-boundary diagnostics while preserving the existing one-request semantics and earliest primary failure.

Purpose: Resolve the ambiguity exposed by sealed attempts 9 and 10 entirely in software before any separately authorized future external attempt.

Output: A pure Rust classifier/projection, a private single-request curl adapter, Phase 35 supervisor integration, exhaustive software regressions, and a verified todo completion. No hardware, device/network request, credentials, promotion, evidence-truth change, or push occurs.
</objective>

<execution-context>
@/Users/peterryszkiewicz/.codex/get-shit-done/workflows/execute-plan.md
@/Users/peterryszkiewicz/.codex/get-shit-done/templates/summary.md
</execution-context>

<context>
@AGENTS.md
@AGENTS.bright-builds.md
@standards/core/architecture.md
@standards/core/code-shape.md
@standards/core/testing.md
@standards/core/verification.md
@standards/languages/rust.md
@.planning/STATE.md
@.codex/tasks/todo.md
@.planning/phases/35-detector-gated-correlated-evidence-and-exact-parity-promotion/35-04-PLAN.md
@.planning/phases/35-detector-gated-correlated-evidence-and-exact-parity-promotion/35-HARDWARE-EVIDENCE.md
@scripts/phase35-correlated-evidence.sh
@scripts/phase35-correlated-evidence-effects.sh
@scripts/phase35-correlated-evidence-test.sh
@scripts/BUILD.bazel
@tools/parity/src/main.rs
@tools/parity/BUILD.bazel

The repository guidance and Bright Builds architecture, testing, verification, code-shape, and Rust standards materially constrain this plan: raw HTTP input is parsed once into Rust domain types; classification is a pure functional core; curl and filesystem work stay in a thin shell adapter; tests use Arrange/Act/Assert and real process/filesystem boundaries; nullable Rust names use `maybe_`; and each implementation commit follows the exact Rust pre-commit sequence.

Hard boundary:

- Software-only. Do not invoke the detector, access a device, open a real network connection, inspect credentials, flash, monitor, PATCH, reboot, restore a real setting, promote evidence, update parity rows, alter lifecycle or verification truth, create attempt documentation, create `35-04-SUMMARY.md`, push, use direct UART, or manipulate pins.
- Existing attempts 1 through 10 and their evidence are immutable. Do not reuse, reopen, retry, splice, rewrite, or reinterpret their roots.
- Fake curl must be a fresh local process that writes only synthetic fixture data; production curl must not run during tests.
- The executor may commit the two code/test slices atomically and the final todo closure separately. Quick PLAN/SUMMARY/STATE artifacts are left for the orchestrator.
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Add the strict typed HTTP classifier and redacted projection</name>
  <files>tools/parity/src/phase35_http.rs, tools/parity/src/phase35_http/tests.rs, tools/parity/src/main.rs, tools/parity/BUILD.bazel</files>
  <behavior>
    - A valid bounded HTTP/1.1 observation with one complete 2xx response and a JSON string hostname returns terminal category `ready`, a private parsed hostname, and projection schema `phase35-http-boundary-v1`.
    - The strict metrics document denies unknown or missing keys and contains only: scheme category, curl exit code, TCP timing, TLS timing, request bytes, response status, response header count, response header bytes, response body bytes, total duration, first-byte duration, and TLS verification.
    - The classifier reads the response body separately from metrics; malformed, extra, inconsistent, over-limit, or mismatched facts are diagnostic-invalid input rather than an ordered HTTP terminal result.
    - For valid metrics/body inputs, the exact terminal order is `tcp_connection_failure`, `tls_handshake_failure` when HTTPS, `request_transmission_incomplete`, `response_status_missing`, `response_headers_missing`, `non_success_response_status`, `response_body_missing`, `response_body_incomplete_or_over_limit`, `invalid_json`, `invalid_hostname_schema`, then `ready`.
    - `http_diagnostic_invalid` is a separate malformed-input fallback and is never inserted into or allowed to reorder the terminal classifier sequence.
    - The projection allowlist is exactly schema version plus booleans, bounded counts/durations, response-status class, and one terminal category; it contains no digests and forbids origin, host, IP, port, headers, body, hostname, curl error, credentials, and device identifiers.
  </behavior>
  <action>
Create `phase35_http.rs` plus `phase35_http/tests.rs` using `foo.rs` + `foo/` module layout. Define closed enums/newtypes for scheme category (`http` or `https`), TLS applicability/verification, bounded bytes and milliseconds, status class, typed observation facts, the exact ordered terminal category enum, private classified result, and shareable projection. Make illegal combinations unrepresentable where practical. Parse a deny-unknown metrics schema containing only `scheme_category`, `curl_exit_code`, `tcp_connect_millis`, `tls_handshake_millis`, `request_bytes`, `response_status`, `response_header_count`, `response_header_bytes`, `response_body_bytes`, `total_millis`, `first_byte_millis`, and `tls_verification`; read body bytes separately. Reject malformed, missing, extra, inconsistent, or out-of-bound values before terminal classification. Keep classification and projection pure; the module performs no file, process, curl, network, clock, or environment access.

Add `classify-phase35-http` to the parity CLI. It accepts only the strict private metrics input and private body input plus private projection-output and hostname-output paths; it does not require or accept raw headers. Reject aliased/non-regular inputs and pre-existing output ambiguity. For a valid observation, always persist a `phase35-http-boundary-v1` projection and its terminal category, including non-`ready` results, before returning nonzero. Write a hostname only for `ready`, only to the private hostname output, and never stdout. The exact projection fields are: `schema_version`; booleans for TCP connected, TLS applicable/established/verified, request transmission complete, response status received, response headers received, response body received/complete, JSON parsed, and hostname schema valid; bounded counts for curl exit code, request bytes, response header count/bytes, and response body bytes; bounded TCP/TLS/first-byte/total milliseconds; `response_status_class`; and `terminal_category`. No other field is permitted. The shell remains responsible for mode-0700/0600 ownership, while the CLI verifies supplied files satisfy the expected private-file contract.

Write RED tests first for every exact terminal category and precedence edge, HTTP versus HTTPS/TLS applicability, 2xx versus non-success status classes, zero/partial/over-limit body, malformed JSON, missing/non-string hostname, every missing/extra/malformed/inconsistent metrics field, bounded counts/durations, exact projection field equality, and successful private-hostname/redacted-projection separation. Assert the forbidden-output matrix exactly: origin, host, IP, port, headers, body, hostname, curl error, credentials, and device identifiers. Register the new module and tests in both the parity binary and Bazel target. Run the mandatory Rust sequence in order, then the focused parity Bazel test, diff, and redaction checks. Commit exactly these four files as `feat(quick-260719-tfu): classify Phase 35 HTTP boundaries`. Do not include planning artifacts and do not push.
  </action>
  <verify>
    <automated>cargo fmt --all &amp;&amp; cargo clippy --all-targets --all-features -- -D warnings &amp;&amp; cargo build --all-targets --all-features &amp;&amp; cargo test --all-features &amp;&amp; bazel test //tools/parity:tests &amp;&amp; git diff --check &amp;&amp; bash -c 'set -euo pipefail; scan="$(mktemp)"; trap '\''rm -f "$scan"'\'' EXIT; chmod 600 "$scan"; git diff --unified=0 -- tools/parity/src/phase35_http.rs tools/parity/src/phase35_http/tests.rs tools/parity/src/main.rs tools/parity/BUILD.bazel | awk '\''substr($0,1,4) != "+++ " &amp;&amp; substr($0,1,1) == "+" { print substr($0,2) }'\'' &gt;"$scan"; scripts/phase28.1.1-promoted-evidence-denylist.sh "$scan"'</automated>
  </verify>
  <done>The pure core and CLI enforce the strict metrics/body contract, exact terminal order, `phase35-http-boundary-v1` projection allowlist, persistent non-ready category, private hostname, and separate `http_diagnostic_invalid` fallback in one tested atomic commit.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Integrate one bounded curl request and preserve primary failure precedence</name>
  <files>scripts/phase35-http-boundary-read.sh, scripts/phase35-http-boundary-read-test.sh, scripts/phase35-correlated-evidence.sh, scripts/phase35-correlated-evidence-effects.sh, scripts/phase35-correlated-evidence-fixture.sh, scripts/phase35-correlated-evidence-test.sh, scripts/BUILD.bazel</files>
  <behavior>
    - Original, immediate, and restoration labels all traverse the same adapter and each causes exactly one GET request.
    - The production request invokes curl directly as HTTP/1.1 GET with `--noproxy '*'`, no redirects, 5-second connect timeout, 10-second total timeout, 65,536-byte response maximum, retries disabled, and no `--fail`.
    - Private files are limited to body, headers, stderr, write-out metrics, projection, private hostname, and necessary private argv/test records.
    - Fake-curl real-process cases distinguish every classifier boundary, enforce 0700 directories and 0600 files, prove actual process status equals reported curl exit code, and prove no retry or diagnostic request occurs.
    - Direct source-tree and opaque Bazel/runfiles execution resolve the already-built parity tool without invoking nested `just`, `bazel`, or another build runner.
    - Original read terminal category `ready` is recorded before `mutation_started` may change; every earlier category exits before mutation or PATCH.
    - An HTTP/supervisor primary remains compatible `category=` even when restoration or cleanup later fails; restoration and cleanup remain secondary-only, and finalization-only failure uses primary `supervisor_finalization_failed`.
  </behavior>
  <action>
Create `phase35-http-boundary-read.sh` as the thin imperative adapter. For one requested label, create a fresh mode-0700 directory under the supplied protected root and only mode-0600 body, headers, stderr, write-out metrics, projection, private-hostname, and necessary private argv/test files. Invoke curl exactly once with `--request GET`, `--http1.1`, `--noproxy '*'`, `--max-redirs 0` and no `--location`, `--connect-timeout 5`, `--max-time 10`, `--max-filesize 65536`, `--retry 0`, and no `--fail`. Capture the response body and headers privately and use curl write-out to produce only the strict metrics keys defined in Task 1.

Before classification, the shell must strictly reject missing/extra/malformed write-out keys, compare the actual curl process exit status with reported `curl_exit_code`, compare body file length with reported body bytes, enforce all timing/byte bounds and internal ordering, and reject impossible TCP/TLS/status/header combinations. Any malformed, inconsistent, process-status-mismatched, or artifact-invalid input must persist a valid minimal `phase35-http-boundary-v1` projection with terminal category `http_diagnostic_invalid`, expose compatible `category=http_diagnostic_invalid`, and stop; it must not enter the ordered terminal classifier. Do not print the URL, origin, host, IP, port, raw curl error, headers, body, hostname, credential, device identifier, or local private path.

After that single curl process exits, invoke the already-built parity `report classify-phase35-http` executable directly. Resolve it from the workspace `bazel-bin` path or its opaque `_main/tools/parity/report` runfiles location relative to the active binary/`RUNFILES_DIR`; never call `just`, `bazel`, `bazel run`, Cargo, or a nested build tool from the adapter or supervisor. Permit fake-curl/fake-classifier overrides only under the existing explicit fixture/test authority, so production cannot silently select a test executable.

Replace the production branch of `read_setting` so `original`, `immediate`, and `restoration` all call this adapter and consume its private parsed-hostname file only when the persisted terminal category is `ready`; keep fixture semantics aligned with the same labels. Require the original projection/category to be `ready` and record that checkpoint before `mutation_started` changes from zero. Any original category before `ready` exits through compatible `category=<typed-category>` before mutated-setting creation, mutation state, PATCH, reboot, or restoration. Do not add a readiness probe, HEAD, redirect follow, retry, fallback route, or second GET. Do not instrument unrelated epoch, PATCH, restart, WebSocket, or monitor requests in this task.

Introduce capture-once authoritative HTTP/supervisor primary state. A typed HTTP category is recorded once and propagated instead of being replaced by generic `original_setting_unavailable` or `immediate_readback_missing`; existing non-HTTP supervisor failures retain their first compatible primary. Finalization must always attempt required restoration then cleanup, but restoration and cleanup failures are always secondary-only redacted fields and never become or overwrite the typed HTTP/supervisor primary. Preserve the first authoritative category in the existing compatible `category=` seal/report field. When the main workflow otherwise succeeded and only restoration and/or cleanup makes finalization fail, set the compatible primary to the explicit supervisor category `supervisor_finalization_failed`, while retaining only boolean/category-level restoration and cleanup details in separate secondary fields. Never promote raw `restoration_failed` or `cleanup_failed` into compatible `category=`.

Write RED real-process tests before the adapter: synthetic fake curl emits every exact TCP/TLS/request/status/header/body/JSON/schema terminal case plus malformed, extra, inconsistent, and process-status-mismatched metrics, records argv and invocation count, and exercises actual files/process exit codes. Assert one invocation and the exact flags: `--request GET`, `--http1.1`, `--noproxy '*'`, `--max-redirs 0` with no `--location`, connect 5, total 10, max 65536, retry 0, and absence of `--fail`. Assert the exhaustive allowed private-file set, exact 0700/0600 modes, exact projection field allowlist/version, persistent non-ready category, and the forbidden-output matrix (origin, host, IP, port, headers, body, hostname, curl error, credentials, device identifiers) across stdout, stderr, and projection. Exercise direct built-tool/runfiles resolution with sentinel `just`/`bazel` binaries that fail if called.

Extend the full supervisor suite for all three labels, exact category propagation, `original=ready` before `mutation_started`, every pre-ready original category stopping before mutation/PATCH, simultaneous primary/restoration/cleanup failures retaining the primary compatible `category=`, finalization-only failure using `category=supervisor_finalization_failed`, secondary-only restoration/cleanup fields, success ordering, and the complete existing regression matrix. Register the adapter binary/test and runfiles data in `scripts/BUILD.bazel`.

Run the exact Rust sequence in order, then Bash syntax, `shfmt -d`, `shellcheck`, direct adapter tests, forced fresh Bazel adapter/supervisor/parity/Phase 35/Phase 30 tests, reference, parity, exact lifecycle, diff, and added-line redaction checks. Commit exactly these seven files as `feat(quick-260719-tfu): instrument Phase 35 setting reads`. Do not include planning artifacts and do not push.
  </action>
  <verify>
    <automated>cargo fmt --all &amp;&amp; cargo clippy --all-targets --all-features -- -D warnings &amp;&amp; cargo build --all-targets --all-features &amp;&amp; cargo test --all-features &amp;&amp; bash -n scripts/phase35-http-boundary-read.sh scripts/phase35-http-boundary-read-test.sh scripts/phase35-correlated-evidence.sh scripts/phase35-correlated-evidence-effects.sh scripts/phase35-correlated-evidence-fixture.sh scripts/phase35-correlated-evidence-test.sh &amp;&amp; shfmt -d scripts/phase35-http-boundary-read.sh scripts/phase35-http-boundary-read-test.sh scripts/phase35-correlated-evidence.sh scripts/phase35-correlated-evidence-effects.sh scripts/phase35-correlated-evidence-fixture.sh scripts/phase35-correlated-evidence-test.sh &amp;&amp; shellcheck scripts/phase35-http-boundary-read.sh scripts/phase35-http-boundary-read-test.sh scripts/phase35-correlated-evidence.sh scripts/phase35-correlated-evidence-effects.sh scripts/phase35-correlated-evidence-fixture.sh scripts/phase35-correlated-evidence-test.sh &amp;&amp; bash scripts/phase35-http-boundary-read-test.sh &amp;&amp; bazel test --nocache_test_results //tools/parity:tests //scripts:phase35_http_boundary_read_test //scripts:phase35_correlated_evidence_test //scripts:phase35_promotion_contract_test //scripts:phase30_no_promotion_contract_test &amp;&amp; bazel build //scripts:phase35_correlated_evidence &amp;&amp; just verify-reference &amp;&amp; just parity &amp;&amp; node "$HOME/.codex/get-shit-done/bin/gsd-tools.cjs" verify lifecycle 35 --expect-id 35-2026-07-17T17-00-37 --expect-mode yolo --require-plans --raw &amp;&amp; git diff --check &amp;&amp; bash -c 'set -euo pipefail; scan="$(mktemp)"; trap '\''rm -f "$scan"'\'' EXIT; chmod 600 "$scan"; git diff --unified=0 -- scripts/phase35-http-boundary-read.sh scripts/phase35-http-boundary-read-test.sh scripts/phase35-correlated-evidence.sh scripts/phase35-correlated-evidence-effects.sh scripts/phase35-correlated-evidence-fixture.sh scripts/phase35-correlated-evidence-test.sh scripts/BUILD.bazel | awk '\''substr($0,1,4) != "+++ " &amp;&amp; substr($0,1,1) == "+" { print substr($0,2) }'\'' &gt;"$scan"; scripts/phase28.1.1-promoted-evidence-denylist.sh "$scan"'</automated>
  </verify>
  <done>All three setting reads use the exact one-request curl contract and strict metrics/body classifier, original readiness gates mutation, compatible primary precedence survives restoration/cleanup, finalization-only failure is explicit, and the second atomic commit contains only the tested adapter/integration slice.</done>
</task>

<task type="auto">
  <name>Task 3: Run the complete software gate and close only the diagnostic todo</name>
  <files>.codex/tasks/todo.md</files>
  <action>
Begin from the two clean atomic implementation commits. Run the exact Rust sequence in order and all focused direct/Bazel adapter, supervisor, parity, Phase 35, and Phase 30 tests. Run reference cleanliness, parity, exact Phase 35 lifecycle, shell formatting/lint, redaction, and diff checks. Invoke `just phase35-evidence preflight-only=true` with no credential argument. This command must remain effect-free and report success without detector, device, target, credential, curl, HTTP, flash, monitor, PATCH, reboot, evidence admission, or checklist activity. If any check fails, stop with the todo still pending.

Only after every command passes, edit only the stable `.codex/tasks/todo.md` block `task-phase35-redacted-http-boundary-diagnostic`: mark its existing eight checklist items complete and append a concise completion review naming the two implementation commits and the passed software/preflight gates. State that no hardware or real network request occurred, attempts 1–10 remain sealed and immutable, Phase 35 Plan 04 Task 2 remains incomplete, and this diagnostic does not authorize another attempt or change evidence truth.

Run `git diff --check` and a mode-0600 added-line redaction scan for the localized todo change, then commit only `.codex/tasks/todo.md` as `docs(quick-260719-tfu): close HTTP diagnostic todo`. Do not edit the quick PLAN, create a quick SUMMARY, update STATE, edit Phase 35 documents, create attempt docs, create `35-04-SUMMARY.md`, touch evidence/checklist/verification truth, access credentials, run hardware/device/network commands, or push; those workflow artifacts remain the orchestrator's responsibility.
  </action>
  <verify>
    <automated>cargo fmt --all -- --check &amp;&amp; cargo clippy --all-targets --all-features -- -D warnings &amp;&amp; cargo build --all-targets --all-features &amp;&amp; cargo test --all-features &amp;&amp; bash -n scripts/phase35-http-boundary-read.sh scripts/phase35-http-boundary-read-test.sh scripts/phase35-correlated-evidence.sh scripts/phase35-correlated-evidence-effects.sh scripts/phase35-correlated-evidence-fixture.sh scripts/phase35-correlated-evidence-test.sh &amp;&amp; shfmt -d scripts/phase35-http-boundary-read.sh scripts/phase35-http-boundary-read-test.sh scripts/phase35-correlated-evidence.sh scripts/phase35-correlated-evidence-effects.sh scripts/phase35-correlated-evidence-fixture.sh scripts/phase35-correlated-evidence-test.sh &amp;&amp; shellcheck scripts/phase35-http-boundary-read.sh scripts/phase35-http-boundary-read-test.sh scripts/phase35-correlated-evidence.sh scripts/phase35-correlated-evidence-effects.sh scripts/phase35-correlated-evidence-fixture.sh scripts/phase35-correlated-evidence-test.sh &amp;&amp; bash scripts/phase35-http-boundary-read-test.sh &amp;&amp; bazel test --nocache_test_results //tools/parity:tests //scripts:phase35_http_boundary_read_test //scripts:phase35_correlated_evidence_test //scripts:phase35_promotion_contract_test //scripts:phase30_no_promotion_contract_test &amp;&amp; bazel build //scripts:phase35_correlated_evidence &amp;&amp; just verify-reference &amp;&amp; just parity &amp;&amp; node "$HOME/.codex/get-shit-done/bin/gsd-tools.cjs" verify lifecycle 35 --expect-id 35-2026-07-17T17-00-37 --expect-mode yolo --require-plans --raw &amp;&amp; just phase35-evidence preflight-only=true &amp;&amp; git diff --check &amp;&amp; bash -c 'set -euo pipefail; scan="$(mktemp)"; trap '\''rm -f "$scan"'\'' EXIT; chmod 600 "$scan"; git diff --unified=0 -- .codex/tasks/todo.md | awk '\''substr($0,1,4) != "+++ " &amp;&amp; substr($0,1,1) == "+" { print substr($0,2) }'\'' &gt;"$scan"; scripts/phase28.1.1-promoted-evidence-denylist.sh "$scan"'</automated>
  </verify>
  <done>Every required software, preflight, reference, parity, lifecycle, redaction, and diff gate passes before the localized todo completion is committed; no prohibited external or evidence-truth action occurs.</done>
</task>

</tasks>

<threat-model>

| Threat | Disposition | Mitigation |
| --- | --- | --- |
| A diagnostic issues another request and changes the scarce-attempt contract. | mitigate | One adapter invocation owns one direct HTTP/1.1 GET with `--noproxy '*'`, zero redirects, connect 5, total 10, max 65536, retry 0, and no `--fail`; real-process tests reject probe, HEAD, fallback, or second-request behavior. |
| Origin, host, IP, port, headers, body, hostname, curl error, credentials, or device identifiers escape private storage. | mitigate | Exhaustive 0700/0600 private-file allowlist, exact `phase35-http-boundary-v1` field equality, private hostname output, raw-canary tests, and existing denylist scans. |
| Malformed metrics, an exit-code mismatch, or multiple boundary failures select an invented or later category. | mitigate | Deny-unknown exact metrics schema, actual/reported curl status equality, body-size consistency, separate `http_diagnostic_invalid`, and exhaustive tests for the fixed terminal order. |
| Restoration or cleanup overwrites the failure that actually stopped the workflow. | mitigate | Preserve the first HTTP/supervisor category in compatible `category=`; secondary-only redacted finalization fields never become primary, and finalization-only failure maps to `supervisor_finalization_failed`. |
| Bazel/runfiles execution calls a nested build runner or resolves a source-tree-only helper. | mitigate | Resolve direct built executables across workspace and opaque runfiles layouts; sentinel `just`/`bazel` processes fail the real-process suite if invoked. |
| Software instrumentation is mistaken for authority to retry hardware or promote evidence. | mitigate | Explicit software-only scope, immutable attempts 1–10, unchanged Phase 35/evidence/checklist truth, preflight-only final gate, and a non-authorizing todo completion review. |

</threat-model>

<verification>

- Each code/test commit runs `cargo fmt`, `cargo clippy`, `cargo build`, and `cargo test` in that exact order before commit.
- Pure Rust tests cover the complete typed boundary and precedence matrix.
- A real-process fake-curl suite proves the exact GET/HTTP1.1/no-proxy/no-redirect/5s/10s/65536/no-retry/no-`--fail` contract, actual/reported exit equality, exhaustive private files, redacted output, and direct runfiles tool resolution.
- The existing full Phase 35 supervisor suite remains green with new original/immediate/restoration, ready-before-mutation, pre-ready stop, compatible primary, secondary-only finalization, and finalization-only-category regressions.
- Phase 35 promotion, Phase 30 non-promotion, reference cleanliness, parity, lifecycle, redaction, and diff gates all pass.
- `just phase35-evidence preflight-only=true` passes without detector, credentials, device/network access, or effects before the todo is closed.

</verification>

<success-criteria>

- The next separately authorized Phase 35 request can distinguish every unresolved HTTP readiness boundary without issuing extra requests or exposing raw values.
- Original, immediate, and restoration reads share one tested adapter and one authoritative classifier.
- Earliest typed HTTP/supervisor primary precedence is preserved through finalization, with `supervisor_finalization_failed` used only when finalization creates the overall failure.
- All software-only verification passes and the todo closes afterward.
- No hardware, real network request, credential access, mutation, evidence admission/promotion, truth change, attempt documentation, Phase 35 summary, push, direct UART, or pin work occurs.

</success-criteria>

<output>
The executor creates two atomic code/test commits and one post-verification todo commit. The orchestrator later handles quick-task planning/summary/state artifacts. Do not push.
</output>
