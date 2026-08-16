# Parity work log

## 2026-08-16T03:24:00Z | software diagnosis and boundary correction

- Source commit: `f26fff55c1513f342946f16999d8564cc761ba01`, based on
  immutable plan commit `c050e433`
- Actions: traced the firmware's source-owned
  `runtime_boot_attestation=unavailable` diagnostic through the production
  campaign serial analyzer; reproduced its false admission by the raw substring
  matcher; added one shared whitespace-delimited marker-boundary function;
  added a closed seven-category parser-failure vocabulary with saturating
  counts; and carried the first closed category plus all counts through serial
  diagnostics v2 and sealed campaign result v10.
- Verification: focused `bitaxe-api` runtime-attestation tests, focused
  `bitaxe-flash` serial-boundary tests, the sealed campaign-result test, and
  crate-scoped Clippy with warnings denied pass. The pre-fix focused flash suite
  exposed the two expected version/boundary fixture failures; after the targeted
  corrections all 336 flash tests passed. The first `just test` exposed missing
  Bazel source declarations for the two split modules; adding those exact source
  entries restored all 45 Bazel tests. The ordered final Cargo format, Clippy,
  all-target build, and all-feature test gates; Bright Builds checks; all Bazel
  tests; parity validation; parity progress; redaction verification; reference
  verification; and the canonical firmware build pass. The first parity-report
  invocation immediately after the Bazel suite hit transient OS error 35
  (`Resource temporarily unavailable`) while printing rows; an independent
  unchanged retry passed with `validation_errors: none`, and progress remained
  71 verified of 94 active rows (75.5%).
- Evidence: the production-shaped regression interleaves the exact firmware
  unavailable diagnostic between two logger-prefixed valid attestations and
  proves two candidates, one lookalike, zero parse failures, and a trusted
  result. A separate genuine-marker fixture proves `malformed_token` remains
  fail-closed and counted without retaining input.
- Outcome: the producer/parser mismatch is diagnosed and corrected in software;
  no hardware or protected evidence was accessed.
- Blocker or next safe action: close this plan without promotion. Any
  attempt-004 still requires a separate immutable hardware plan based on the
  committed and pushed correction.
