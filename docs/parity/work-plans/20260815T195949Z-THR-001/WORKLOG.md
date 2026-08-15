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

## 2026-08-15T20:18:00Z | exact replay-origin red and green loops

- Red: The production-shaped in-process and real-child late-attachment cases
  failed as `evidence_invalid` when their complete retained triplets used the
  authoritative `bitaxe_firmware::boot_evidence` tag. This reproduced the
  consumed hardware signature at the real subprocess/file boundary.
- Design: Ranked an exact two-origin host allowlist first because it preserves
  truthful logger ownership and changes one pure parsing boundary. Retagging
  replay in firmware ranked second because it obscures the producer; a new
  retained protocol ranked last because the existing typed protocol is
  sufficient.
- Green: The parser now admits only exact INFO envelopes from
  `bitaxe_firmware` and `bitaxe_firmware::boot_evidence`. Focused automation
  tests pass, including canonical late attachment and real-child coverage.
  Bare payloads, malformed timestamps, non-INFO levels, wrong modules, nested
  replay tags, extra payload fields, missing states, and wrong order remain
  inadmissible, with restoration and evidence withholding preserved.

## 2026-08-15T20:26:00Z | complete software verification

- Verification: Focused automation, ordered Cargo formatting/lint/build/tests,
  Bright Builds, real firmware build, all 45 Bazel tests, parity/progress,
  redaction, reference cleanliness, live-plan selection, and diff checks pass.
- Review: The implementation changes only the pure log-envelope allowlist and
  production-shaped fixtures. No firmware behavior, hardware contract, public
  evidence schema, recovery behavior, or sensitive-value boundary changed.
- Outcome: The software correction is ready to commit and push. THR-001 stays
  `implemented`; this plan grants no attempt-007 authority.
