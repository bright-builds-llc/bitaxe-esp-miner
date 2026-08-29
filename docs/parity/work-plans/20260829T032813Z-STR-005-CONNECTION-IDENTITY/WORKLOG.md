# STR-005 connection identity work log

## 2026-08-29T03:28:13Z | Immutable plan checkpoint

- Source commit: `615b81cd78c7fdee00956f2e1a23eddd6c30b4e7`
- Actions: wrote the decision-complete recovery-readiness, connection-identity,
  bounded-candidate, conditional direct-send, evidence, retry, and stop plan.
- Evidence: planning only; no detector, credential, network, USB, flash,
  monitor, recovery, or device effect.
- Outcome: plan checkpoint pending verification, commit, and push.
- Next safe action: run planning gates and push this checkpoint before source
  implementation.

## 2026-08-29T04:05:00Z | Diagnostic-009 implementation checkpoint

- Source commit: `2051b88f`
- Actions: changed the committed recovery projection to public mode semantics,
  added typed tooling and idempotent admission preflight, added private firmware
  socket identity and socket-error replay, implemented a bounded three-candidate
  fixture inventory, joined the tuple privately, and defined the projection-v2
  public allowlist.
- Verification: red restore-mode, recovery-tooling, source-ownership, tuple,
  and fixture seams were observed before fixes. Focused restore, TCP supervisor,
  flash, firmware ownership, and eight real fixture tests now pass.
- Evidence: software only; no detector, credential, network, USB, flash,
  monitor, recovery, or device effect.
- Outcome: diagnostic-009 implementation is ready for full verification.
- Next safe action: run every required gate, review the diff and privacy
  surface, commit/push, package exact clean source, then run preflight and one
  diagnostic-009 hardware attempt.

## 2026-08-29T04:24:00Z | Diagnostic-009 software verification

- Source commit: `2051b88f`
- Actions: completed recovery admission reuse, public projection mode
  compatibility, private tuple/candidate join, projection-v2 allowlisting,
  accurate local byte accounting, socket-error capture, and bounded fixture
  observation. Extracted recovery readiness and projection construction to keep
  the imperative supervisor within the managed file-length boundary.
- Verification: formatting, strict Clippy, all-target/all-feature build, full
  Cargo tests, Bright Builds, all 57 Bazel tests, canonical six-artifact
  ESP32-S3 package, parity with no validation errors, parity progress,
  redaction, reference cleanliness, focused restore/fixture/tuple tests,
  whitespace, and diff review passed. The first full Bazel run found one legacy
  mode-`0600` launcher fixture; changing only its public projection to `0644`
  passed the launcher and full suites.
- Evidence: software only; no detector, credential, network, USB, flash,
  monitor, recovery, or device effect.
- Outcome: diagnostic-009 implementation is ready for a distinct clean source
  commit and push.
- Next safe action: commit/push, rebuild the exact clean package, run fresh
  detector admission and the idempotent read-only preflight, then invoke
  diagnostic-009 once.
