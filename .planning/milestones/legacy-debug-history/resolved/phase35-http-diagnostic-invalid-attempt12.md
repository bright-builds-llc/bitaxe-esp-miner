---
status: resolved
trigger: "phase35-http-diagnostic-invalid-attempt12"
created: 2026-07-21T03:38:50Z
updated: 2026-07-21T04:38:00Z
---

## Current Focus

hypothesis: confirmed and fixed — positive duration presence survives integer-millisecond projection, including derived TLS handshake duration
test: complete
expecting: complete
next_action: archive the resolved session and commit the diagnostic artifact

## Symptoms

expected: The bounded original-settings GET captures strict metrics, the Rust `classify-phase35-http` command validates them, and either emits a precise terminal HTTP category or reports ready; only ready may permit PATCH.
actual: Attempt 12 reached the original-settings boundary after Boot A succeeded, then failed closed as `http_diagnostic_invalid`. No PATCH or mutation occurred. The root is sealed non-promotable and non-reusable and cleanup succeeded with no secondary failure.
errors: Only the committed typed category `http_diagnostic_invalid` is shareable. Protected values and raw response bodies must not be printed, committed, or summarized.
reproduction: Reproduce in software using the Phase 35 HTTP adapter/classifier boundary and hermetic fake-curl or safe typed projection metadata. Do not contact the device, use the detector, access credentials, issue HTTP requests, flash, monitor, or run hardware.
started: The typed HTTP diagnostic was added in quick task 260719-tfu, private-first dual capture followed in quick task 260720-jwt, and attempt 12 at exact source `7fcad709` exposed the invalid diagnostic.

## Eliminated

## Evidence

- timestamp: 2026-07-21T03:38:50Z
  checked: committed Phase 35 attempt-12 record and quick-task summaries
  found: Boot A private-first classification and finalization passed; the first failure was `http_diagnostic_invalid` at the original-settings HTTP diagnostic classification boundary before PATCH, reboot, or mutation.
  implication: investigation is confined to the HTTP adapter/classifier software boundary; private-first Boot A ordering and finalization are not the failing boundary.

- timestamp: 2026-07-21T03:38:50Z
  checked: common bug patterns and active project lessons
  found: the highest-probability candidates are a data-shape/API-contract mismatch, numeric boundary/coercion defect, or deployed/runfiles process-boundary difference; earliest typed failure must remain authoritative.
  implication: reproduce through the actual shell-to-Rust process boundary and deployed layouts before editing.

- timestamp: 2026-07-21T03:51:00Z
  checked: private metrics artifact format using a non-content-emitting parser probe
  found: the artifact remains in curl write-out key/value form because the adapter rejected it before normalized JSON was written.
  implication: the failing invariant is in the shell adapter's pre-classifier validation path, not Rust JSON deserialization.

- timestamp: 2026-07-21T03:56:00Z
  checked: non-output predicate over the permitted private metrics artifact
  found: the exact positive-duration-to-zero-sentinel producer mismatch predicted by the hypothesis is present; no protected values or paths were emitted.
  implication: root cause is confirmed and a test-first minimal repair is justified.

- timestamp: 2026-07-21T03:59:00Z
  checked: direct real-process fake-curl adapter regression before production changes
  found: the new otherwise-ready sub-millisecond case fails at the adapter ready assertion.
  implication: the software reproduction matches the attempt-12 failure boundary and will serve as the regression guard.

- timestamp: 2026-07-21T04:03:00Z
  checked: Bazel-deployed real-process fake-curl adapter regression before production changes
  found: the same sub-millisecond ready case fails under the runfiles boundary.
  implication: the failure is independent of source-tree execution and the regression covers the deployed adapter/classifier layout.

- timestamp: 2026-07-21T04:09:00Z
  checked: direct real-process fake-curl adapter regression after the minimal duration fix
  found: the full terminal matrix, invalid fallback, zero-duration sentinel, ordinary duration projection, sub-millisecond HTTP boundary, sub-millisecond HTTPS derived boundary, private-hostname separation, and unauthorized override checks pass.
  implication: the fix resolves the reproduced defect without weakening strict classification or privacy behavior.

- timestamp: 2026-07-21T04:13:00Z
  checked: Bazel/runfiles adapter regression after the minimal duration fix
  found: the deployed-layout test passes.
  implication: the fix works at both direct-built and Bazel/runfiles process boundaries.

- timestamp: 2026-07-21T04:18:00Z
  checked: named attempt-12 adapter precedence regression
  found: a valid sub-millisecond observation with a non-success response now preserves `non_success_response_status` rather than falling back to `http_diagnostic_invalid`.
  implication: positive-duration normalization restores entry into the strict ordered classifier without weakening earliest-category precedence.

- timestamp: 2026-07-21T04:24:00Z
  checked: focused software verification
  found: direct and Bazel HTTP adapter tests, parity Rust tests, Phase 35 correlated-evidence tests, Phase 35 promotion contract, Phase 30 non-promotion contract, correlated-evidence build, shell syntax/style/lint, reference cleanliness, parity validation, redaction, and Phase 35 lifecycle all pass.
  implication: adjacent workflow, privacy, evidence, and non-promotion behavior remains intact before the full Rust gate.

- timestamp: 2026-07-21T04:32:00Z
  checked: mandatory ordered Rust pre-commit sequence
  found: `cargo fmt --all`, Clippy for all targets/features with warnings denied, all-target/all-feature build, and all-feature tests pass in the required order.
  implication: the repository-wide Rust surface is green and the fix is ready for staged redaction and atomic commit.

- timestamp: 2026-07-21T04:38:00Z
  checked: staged redaction, cached diff, and atomic implementation commit
  found: staged redaction passed; the staged diff contained only the HTTP adapter and its hermetic test; commit `58b7e33a` recorded the fix and regression coverage.
  implication: implementation scope is atomic and contains no planning, evidence, hardware, credential, or promotion changes.

## Resolution

root_cause: `seconds_to_millis` uses nearest-integer rounding, so positive curl durations below half a millisecond become `0`; the schema reserves `0` for an absent boundary, and downstream request facts therefore trip the adapter's strict consistency guard before Rust classification.
fix: Quantize every positive curl duration to at least one integer millisecond while retaining exact zero as the absence sentinel; compute the HTTPS handshake delta from raw cumulative seconds with the same presence-preserving rule.
verification: Pre-fix direct and Bazel/runfiles regressions failed at the exact sub-millisecond adapter boundary. Post-fix direct/Bazel adapter, Phase 35 correlated-evidence, promotion/non-promotion, parity, reference, lifecycle, redaction, shell syntax/style/lint, full Rust format/lint/build/test, and diff checks all passed.
files_changed: [scripts/phase35-http-boundary-read.sh, scripts/phase35-http-boundary-read-test.sh]
