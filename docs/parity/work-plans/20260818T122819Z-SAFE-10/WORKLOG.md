# Parity work log

## 2026-08-18T13:21:51Z | projection checkpoint

- Source commit: `1c0ad96dd799bd188a8d36c1da9b28f43ebe3aa9`.
- Actions: added the Rust SAFE-10 evidence contract and validator, generated
  TypeScript schema/command, 19-path source/reference inventory, attempt-to-
  current production compatibility check, protected private-first projector,
  CLI/Just/Bazel wiring, and synthetic real-validator/privacy/drift tests.
- Verification: 103 Rust contract tests and focused automation tests passed,
  including current inventory, complete protected fixture, source drift, and
  prerequisite withholding. Ordered Cargo gates, Bright Builds, all 47 Bazel
  targets, parity/progress, firmware build/package, redaction, reference, file-
  size, sensitive-value, and diff checks passed.
- Evidence: the sole projection command failed before reading/classifying the
  campaign candidate because its validator executable was absent from the
  projector binary runfiles. No candidate or projection was created and the
  protected attempt remained unchanged. The missing runfiles dependency was
  then added to generic/specialized/test binary data, the built projector
  runfile was verified executable, and focused/full gates passed again.
- Outcome: implementation complete and pushed, but the immutable plan's sole
  projection invocation was consumed by `process_failed`; `SAFE-10` remains
  `implemented` and no checklist/progress transition is requested.
- Blocker or next safe action: close this plan. A fresh software-only immutable
  plan may run the now-corrected exact projection command once; it requires no
  device, credentials, network, or hardware effect.
