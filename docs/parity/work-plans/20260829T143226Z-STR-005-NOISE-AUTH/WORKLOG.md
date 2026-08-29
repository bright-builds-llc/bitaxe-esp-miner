# STR-005 Noise authentication worklog

## 2026-08-29T14:32:26Z | Immutable plan

- Source base: `607336a60bc22180e5eb636b222be932b1bfd2df`
- Immutable plan SHA-256:
  `9a3e5a630a52de6b8819dcb33aac64f5324df030fab50fd248fc33437b6587ea`
- Action: created the decision-complete no-mining Noise-authentication plan and
  activated the dependency-satisfied successor task.
- Hardware/network effects: none.

## 2026-08-29T16:10:00Z | Noise-auth implementation

- Actions: added the distinct `noise_auth_v1` consume-once admission, replayed
  firmware transcript and private tuple marker, exact standard-stream send
  accounting, configured-authority authentication, reserved encrypted proof,
  bounded three-candidate fixture inventory, closed tuple projection, hardened
  recovery preflight, recovery-only continuation, offline finalizer, evaluator
  identity, and the `stratum-v2-noise-auth` command surface.
- Red-to-green evidence: host contract/schema tests, fixture exact-proof and
  candidate-inventory tests, firmware source-ownership tests, and restore-root
  authority tests failed at their intended seams before the minimum production
  changes and now pass.
- Verification: formatting, strict all-target/all-feature Clippy, the
  all-target/all-feature build, all-feature Cargo tests, focused Bazel tests,
  Bright Builds, and the real ESP32-S3 firmware build pass. The remaining full
  Bazel/package/parity/integrity gate runs before the implementation commit.
- Hardware/network effects: none.
- Next safe action: finish the full repository gate, commit/push the
  implementation separately, package exact clean HEAD, then run fresh detector
  admission and ordinal-001 preflight.

## 2026-08-29T16:35:00Z | Implementation verification complete

- Verification: all ordered Rust gates, Bright Builds, all 68 Bazel tests,
  canonical ESP32-S3 firmware build and six-artifact package, parity with no
  validation errors, parity progress, redaction, pinned-reference cleanliness,
  whitespace, and final diff review passed. The first parity invocation hit the
  known transient host resource error after rendering its report; an isolated
  rerun passed.
- Hardware/network effects: none.
- Outcome: implementation is ready for its separate commit and push.
- Next safe action: verify, commit, and push this immutable plan separately;
  then implement the pre-agreed TDD seams without hardware effects.

## 2026-08-29T15:05:00Z | Plan verification

- Verification: formatting, strict all-target/all-feature Clippy, the
  all-target/all-feature build, all-feature Cargo tests, and Bright Builds
  passed. The remaining Bazel, package, parity, redaction, reference, and diff
  gates run immediately before the plan commit.
- Hardware/network effects: none.
