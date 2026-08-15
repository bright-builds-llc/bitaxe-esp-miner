# Parity work log

## 2026-08-15T19:59:49Z | software-only producer-tag contract

- Selection: Clean synchronized source `524f074b`; THR-001 is the first
  unfinished candidate and no earlier row was skipped.
- Failure signal: Attempt-006 contained one direct fault/recovery pair and
  eleven complete retained replay triplets, all with one of two canonical
  producer tags; the host allowlist omitted the replay tag.
- Action: Freeze a red-first software plan for the exact direct/replay origin
  boundary. No attempt-007 or hardware effect is authorized.

## 2026-08-15T20:05:00Z | immutable-plan verification

- Plan SHA-256:
  `c3dfb3219e73e8c4fd1d1c88e4fe52db06bc02a0a1721b94cdb5ac9d2adf65be`.
- Verification: Ordered Cargo gates, Bright Builds, real firmware, all 45
  Bazel tests, parity/progress, redaction, reference cleanliness, live selector,
  and diff checks passed without hardware.
- Outcome: The exact replay-origin red loop may begin after this commit is
  pushed. Attempt-007 remains unauthorized.
