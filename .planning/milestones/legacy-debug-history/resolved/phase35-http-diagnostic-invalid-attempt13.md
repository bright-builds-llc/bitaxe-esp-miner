---
status: resolved
trigger: "Investigate and fix the repeated Phase 35 http_diagnostic_invalid from attempt 13 at source 02f128db56b332e50e11f57935f29e22e3830f66 without rerunning hardware."
created: 2026-07-21T21:13:21Z
updated: 2026-07-21T21:37:29Z
---

## Current Focus

hypothesis: Confirmed root cause is fixed and independently verified in software at the original sealed boundary.
test: Complete — orchestrator independently reran the focused Bazel, Rust, formatting, redaction, and diff checks.
expecting: Complete — software fix accepted without rerunning attempt 13 or treating software replay as hardware evidence.
next_action: None — session resolved and archived; no commit or push requested.

## Ranked Hypotheses

1. **H1 — scheme case mismatch:** If curl emits an uppercase scheme but the adapter accepts only lowercase, lowercasing only that metric will make `http_diagnostic_invalid` disappear and reveal the next typed category.
2. **H2 — boundary-size inconsistency:** If the curl-reported header/body sizes disagree with the observed zero-byte files, scheme normalization alone will still yield `http_diagnostic_invalid`; aligning only the inconsistent size would then expose a typed category.
3. **H3 — process-status mismatch:** If the captured curl exit metric differs from the real process status, replaying the exact stored exit as the fake process status will still fail; substituting only the matching status would remove the invalid fallback.
4. **H4 — numeric/timing parse inconsistency remains:** If another numeric field is rejected or violates timing order/bounds, scheme normalization will still yield `http_diagnostic_invalid`, and a direct private classifier probe on the converted metrics will identify the consistency family without protected output.
5. **H5 — built classifier resolution or output contract failure:** If the adapter cannot resolve or accept the runfiles classifier, a lowercase known-good fixture through the same built target will also fail invalid; otherwise this branch is eliminated.

## Symptoms

expected: The original-settings GET classifier should preserve valid curl boundary metrics and return a discriminating terminal category, ideally `ready`, so the Phase 35 state machine can proceed safely.
actual: The adapter rejects its own real curl output as `http_diagnostic_invalid` and writes the all-zero `phase35-http-boundary-v1` invalid projection.
errors: `non-promotion.seal category=http_diagnostic_invalid`; HTTP projection `terminal_category=http_diagnostic_invalid` with all counters and timings zero; no restoration or cleanup secondary category.
reproduction: Replay attempt 13's sealed private HTTP inputs from `<sealed-attempt-13-root>/raw/http-original/` through the real adapter/classifier seam without hardware or protected-data output.
started: Attempt 12 exposed this after the private-first work. Commit `58b7e33a` attempted a sub-millisecond timing repair, but attempt 13 reproduced the same category on 2026-07-21.

## Eliminated

- hypothesis: H1 as the complete root cause — normalizing only the uppercase scheme would make the invalid fallback disappear.
  evidence: The normalized replay did contain lowercase `http`, but still produced the exact all-zero `http_diagnostic_invalid` projection.
  timestamp: 2026-07-21T21:31:00Z
- hypothesis: H2 — curl-reported sizes disagree with the observed files.
  evidence: Both header and body byte-count comparisons passed against the sealed files in place.
  timestamp: 2026-07-21T21:33:00Z
- hypothesis: H4 lexical-shape branch — an integer or seconds token violates the adapter grammar.
  evidence: All expected integer and seconds fields passed the adapter's exact lexical patterns.
  timestamp: 2026-07-21T21:33:05Z
- hypothesis: H3 — curl process status differs from its stored exit metric.
  evidence: The replay process exits with the stored curl exit metric; after correcting the two confirmed values, the adapter reaches the Rust classifier and returns a typed category.
  timestamp: 2026-07-21T21:33:35Z
- hypothesis: H5 — classifier resolution/output contract fails in the built runfiles layout.
  evidence: The cumulative counterfactual invoked the same built adapter/runfiles seam and produced a valid typed projection and matching stdout category.
  timestamp: 2026-07-21T21:33:35Z

## Evidence

- timestamp: 2026-07-21T21:13:21Z
  checked: Active lessons and repository guidance
  found: The applicable guardrails require a private-first immutable classifier input, earliest typed-failure precedence, a real cross-process/runfiles boundary, and no unchanged hardware retry.
  implication: The investigation must stay entirely software-only and replay the production boundary from the sealed private root.
- timestamp: 2026-07-21T21:16:00Z
  checked: `.planning/debug/knowledge-base.md`
  found: Exact keyword match to attempt 12, where a positive sub-millisecond curl duration became the reserved zero sentinel; attempt 13 nevertheless failed after that repair.
  implication: Test the known timing-consistency mechanism first, but also distinguish a second conversion or schema mismatch rather than assuming the prior root cause repeated unchanged.
- timestamp: 2026-07-21T21:21:00Z
  checked: Attempt 13 HTTP artifact structure and production adapter contract
  found: The sealed body and headers are both zero bytes; metrics are present; the adapter wrote the standard invalid projection. The adapter invokes curl once, transforms metrics, then invokes the built Rust classifier through runfiles only if shell consistency checks pass.
  implication: A safe replay can avoid copying body, headers, hostname, or stderr by recreating only their observed empty shape while streaming sealed metrics directly into the production adapter.
- timestamp: 2026-07-21T21:27:00Z
  checked: Built adapter/runfiles replay, two independent private roots
  found: Both runs deterministically returned `http_diagnostic_invalid` and persisted the exact all-zero invalid projection in under one second, with no private hostname and no protected values emitted.
  implication: The temporary protected replay is a fast, deterministic, agent-runnable red-capable command for the exact attempt 13 failure.
- timestamp: 2026-07-21T21:27:00Z
  checked: Minimal replay inputs
  found: Sealed metrics plus the observed zero-byte body/header shape are sufficient; raw stderr, hostname, target origin, credentials, device identifiers, and hardware are not required.
  implication: The minimized regression can live at the existing fake-curl built adapter/runfiles seam without embedding or copying protected operational data.
- timestamp: 2026-07-21T21:27:00Z
  checked: Safe scheme-category shape
  found: The captured scheme token has length four, is ASCII alphanumeric, matches uppercase HTTP case-insensitively, and does not match the adapter's accepted lowercase literal.
  implication: H1 is the highest-probability single-point failure; test it first with a one-variable counterfactual.
- timestamp: 2026-07-21T21:31:00Z
  checked: One-variable scheme-case counterfactual
  found: The replayed metric was confirmed lowercase `http`, yet the adapter still wrote the all-zero invalid projection.
  implication: Scheme case is a real earliest rejection in the original input, but it is not the only invalidating condition; continue to H2 without treating the partial repair as sufficient.
- timestamp: 2026-07-21T21:33:00Z
  checked: Reported-versus-observed size consistency
  found: Header and body byte counts both match their corresponding sealed zero-byte files.
  implication: The second rejection is not a size mismatch; continue to numeric grammar and consistency checks.
- timestamp: 2026-07-21T21:33:05Z
  checked: Exact integer and seconds lexical grammar
  found: Both invalid-field counts were zero.
  implication: Numeric tokens are lexically valid; the second rejection is in conversion, bounds, or cross-field consistency.
- timestamp: 2026-07-21T21:33:10Z
  checked: Adapter-equivalent bounds and consistency probe
  found: Every checked bound, ordering, size, TLS, and absence/presence invariant passed except `total_millis <= 10000`.
  implication: Attempt 13 contains at least two independent adapter-invalidating facts: uppercase curl scheme and a total duration slightly or materially above the adapter's exact max-time ceiling.
- timestamp: 2026-07-21T21:33:15Z
  checked: Bounded total-duration bucket
  found: The converted total is greater than 10,000 ms and no greater than 10,005 ms.
  implication: This is small timeout-observation overshoot, not an unbounded or malformed duration.
- timestamp: 2026-07-21T21:33:25Z
  checked: Cumulative counterfactual through the production built adapter/runfiles seam
  found: Normalizing scheme case and substituting only the total-duration ceiling changed the exact invalid fallback to `request_transmission_incomplete`; every other captured metric and empty-file fact remained unchanged.
  implication: The mechanism is confirmed: both adapter assumptions reject valid real curl output before the existing typed category order can classify the observation.
- timestamp: 2026-07-21T21:33:35Z
  checked: New attempt-13 combined fake-curl regression through the real adapter/classifier seam
  found: The test fails before the fix because stdout lacks `category=request_transmission_incomplete`; the adapter instead preserves its existing invalid fallback.
  implication: The regression is red on the exact mechanism and can verify the production repair.
- timestamp: 2026-07-21T21:33:40Z
  checked: Rust regression before production fix
  found: `accepts_bounded_timeout_observation_overshoot` failed with `OutOfBounds("total_millis")`.
  implication: The Rust classifier shared the same exact-ceiling mismatch and required alignment with the shell boundary.
- timestamp: 2026-07-21T21:33:45Z
  checked: Direct adapter suite after rebuilding the Rust classifier
  found: The combined attempt-13 regression, uppercase scheme coverage, post-timeout observation coverage, existing terminal matrix, and invalid fallback cases all pass.
  implication: The source-level adapter/classifier seam is green; verify the unchanged sealed replay and deployed Bazel/runfiles test next.
- timestamp: 2026-07-21T21:33:50Z
  checked: Original sealed attempt-13 replay after fix
  found: Without either counterfactual enabled, the rebuilt production adapter/runfiles classifier returns `request_transmission_incomplete`, preserves TCP-connected truth, and does not create a private hostname.
  implication: The exact original symptom is resolved in software and the earliest precise typed category is restored.
- timestamp: 2026-07-21T21:33:55Z
  checked: Focused Rust verification and format/lint checks
  found: All 13 `phase35_http` tests pass, including the new overshoot regression; `cargo fmt --all --check`, `shfmt -d` on both shell files, and package-scoped Clippy with warnings denied pass.
  implication: The Rust and shell changes are formatted, lint-clean, and retain the full typed classifier behavior.
- timestamp: 2026-07-21T21:34:00Z
  checked: Bazel/runfiles verification
  found: `//scripts:phase35_http_boundary_read_test` and `//tools/parity:tests` both pass from the deployed Bazel layout.
  implication: The direct built-tool/runfiles contract is green, not only the source-tree invocation.
- timestamp: 2026-07-21T21:34:01Z
  checked: Redaction, integrity, cleanup, and diff review
  found: `just verify-redaction` passes; the sealed attempt-13 HTTP artifact digests are unchanged; no `[DEBUG-*]` logs remain; the temporary replay harness is absent from the workspace; `git diff --check` passes; only the four owned source/test files and this debug record are changed.
  implication: The repair preserves private-first evidence, leaves the sealed root immutable, and has no unintended or unrelated workspace side effects.
- timestamp: 2026-07-21T21:37:29Z
  checked: Independent orchestrator verification and privacy scrub
  found: The orchestrator confirmed the focused Bazel, Rust, shfmt, redaction, and diff checks; the explicit ignored private-root pathname was replaced with the opaque label `<sealed-attempt-13-root>` and no protected operational paths or raw values remain.
  implication: Human verification is complete and the session can be archived as resolved without further effects.

## Resolution

root_cause: The shell adapter treated curl's `scheme` write-out as a lowercase enum even though the real invocation emitted uppercase `HTTP`, and it reused the configured 10,000 ms timeout as an exact observed-duration ceiling even though real `time_total` exceeded that deadline by at most 5 ms. Each guard independently called the generic invalid fallback before the Rust classifier, erasing valid boundary facts and the earliest precise `request_transmission_incomplete` category.
fix: Canonicalize curl's case-insensitive scheme token to lowercase at the shell boundary. Keep the configured curl timeout at 10 seconds but admit only a separately named, bounded 1-second post-deadline observation envelope in both shell and Rust; reject observations above 11 seconds. Add exact combined, uppercase-case, bounded-overshoot, and over-bound regression coverage without changing terminal-category precedence.
verification: Exact sealed replay was red twice before the fix and green after the fix as `request_transmission_incomplete`; direct adapter tests pass; all 13 focused Rust classifier tests pass; Bazel `//scripts:phase35_http_boundary_read_test` and `//tools/parity:tests` pass; Cargo format, shfmt diff, package-scoped Clippy with warnings denied, redaction verification, immutable-root digest checks, debug-log cleanup, diff checks, independent orchestrator review, and final privacy scrub pass. No hardware, detector, credential, USB/serial, network request, PATCH, reboot, admission, commit, or push occurred.
files_changed: [scripts/phase35-http-boundary-read.sh, scripts/phase35-http-boundary-read-test.sh, tools/parity/src/phase35_http.rs, tools/parity/src/phase35_http/tests.rs]
