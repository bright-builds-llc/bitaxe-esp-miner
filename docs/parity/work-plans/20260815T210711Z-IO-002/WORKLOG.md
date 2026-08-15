# Parity work log

## 2026-08-15T21:22:55Z | implementation checkpoint

- Source commit: `a035a6a6216049f9c40951e47730b61137610817`.
- Actions: Added the closed Rust ADC observation and private-input contracts,
  independent validators, generated TypeScript surface, repo-owned composite
  capture command, exact plan/task/source binding, protected publication,
  redaction admission, human command surface, and focused regressions. The
  production ADC, safety, mining, and control paths were not changed.
- Verification: Five focused Rust contract/input tests passed. The new Bazel
  command and both validators built. The first automation-test analysis exposed
  that a raw cross-package ADC reference file required a `js_library` runfiles
  owner; converting all new source inventories to package-owned `js_library`
  targets resolved that exact boundary. Both the Rust contract target and the
  full real-child automation target then passed.
- Evidence: The closed public schema carries only fixed ADC configuration,
  boolean quorum facts, identities, and digests. Tests prove successful atomic
  publication, lossless-validator rejection and withholding, checked-in source
  semantics, protected modes, and real child-process ownership.
- Outcome: The bounded IO-002 evidence workflow is implemented and ready for
  the mandatory software, firmware, package, privacy, and repository gates.
- Blocker or next safe action: Run the complete ordered verification sequence,
  simplify and review the diff, then commit and push the implementation before
  detector or device access.

## 2026-08-15T21:31:58Z | software verification checkpoint

- Source commit: `a035a6a6216049f9c40951e47730b61137610817` plus the
  current uncommitted implementation diff.
- Actions: Completed the explicit simplification pass. The first Bright Builds
  run found four new registrations 5–13 lines over the 628-line ceiling. The
  contracts and invocation fixture were tightened, and the pre-existing typed
  request parser was extracted from the crowded CLI into a focused module so
  the ADC command remains readable without a file-length workaround.
- Verification: `cargo fmt --all`, Clippy with warnings denied, all-target/all-
  feature Cargo build, all-feature Cargo tests, Bright Builds checks, the
  production firmware build, all 45 Bazel test targets, parity validation,
  parity progress, redaction verification, and pinned-reference verification
  passed. The post-simplification real-child automation target also passed.
- Evidence: Parity remained at 69 verified rows out of 94 active rows before
  any IO-002 promotion. Generated TypeScript contracts remained byte-identical
  to their Rust-owned source, and `git diff --check` passed.
- Outcome: Software, firmware, workflow, privacy, and reference gates are
  green; no production ADC, mining, safety, or control behavior changed.
- Blocker or next safe action: Rerun the ordered final pre-commit gates over the
  completed diff, commit and push the exact source, build its package, and only
  then run the detector-gated attempt-001 command from the immutable plan.

## 2026-08-15T21:50:26Z | attempt-001 terminal checkpoint

- Source commit: `0bd2dfff2e662431fba3bb95d5654b1dbce3c80a`.
- Actions: Pushed the clean implementation, built its exact package, admitted
  exactly one Ultra 205 through the repo-owned detector, and ran the sole
  authorized attempt-001 command. No retry or second hardware command ran.
- Verification: The base exact-package system-info projection independently
  validates. It records safe boot, disabled mining and hardware control,
  complete cleanup, and passed redaction. Protected roots/files had the planned
  `0700`/`0600` modes. The lossless ADC validator rejected both fresh samples
  as outside its immutable 400–2,000 mV range.
- Evidence: Terminal status `failed`, category `evidence_invalid`, stage
  `adc_observation_capture`, and `projection_published=false`. The public ADC
  projection is absent. Raw values, device identifiers, origins, and network
  details remain only in ignored protected evidence and are not recorded here.
- Outcome: `blocked`; IO-002 remains `implemented`, with verification claimed
  `no`. `CLOSURE.md` records the terminal boundary and non-claims.
- Blocker or next safe action: A fresh task and immutable plan must establish a
  justified passive-safe-state ADC range from authoritative electrical evidence
  or separately authorized safe stimulus before retry eligibility. This plan
  authorizes no attempt-002 or unchanged retry.
