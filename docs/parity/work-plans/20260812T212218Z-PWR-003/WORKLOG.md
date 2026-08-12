# Parity work log

## 2026-08-12T21:22:18Z | Selection and immutable-plan checkpoint

- Source commit: `2264f71393949436f1f15306b71f890a6478dc0a`
- Actions: Confirmed a clean synchronized selector, skipped API-009 because
  its explicit two-prompt observer-readiness occurrence is not yet complete,
  and selected the next actionable PWR-003 software-only matcher retry.
- Verification: Confirmed the prior closure's exact two-occurrence failure,
  the absence of a public PWR-003 projection, the accepted PWR-002 source
  digest, and the unique intended stabilization sleep site.
- Evidence: Prior closure
  `docs/parity/work-plans/20260812T203223Z-PWR-003/CLOSURE.md`; no new parity
  evidence exists at this checkpoint.
- Outcome: Fresh bounded plan and task prepared before implementation.
- Blocker or next safe action: Run every plan checkpoint gate, commit and push
  the immutable plan/task, then edit only the PWR-003 matcher and regressions.

## 2026-08-12T21:31:00Z | Immutable-plan verification

- Source commit: `2264f71393949436f1f15306b71f890a6478dc0a`
- Actions: Froze the plan with SHA-256
  `dbd5d3a620726f270fd2827d4c8f53f0301834cea4999107964c22c711cb277e`,
  archived the exhausted prior task as superseded, and retained exactly one
  fresh active PWR-003 task.
- Verification: `cargo fmt --all`, strict Cargo Clippy, all-target Cargo
  build, all-feature Cargo tests, Bright Builds with zero findings, all 41
  Bazel test targets, parity report/progress, generated contracts, pinned
  reference cleanliness, redaction across 15 evidence artifacts, immutable
  digest, task uniqueness, candidate absence, and diff checks passed. An
  initial auxiliary redaction call used unsupported positional paths and was
  rejected as `invalid_invocation`; the canonical `--evidence-root` rerun
  passed and did not change repository state.
- Evidence: Immutable plan and worklog under
  `docs/parity/work-plans/20260812T212218Z-PWR-003/`; no PWR-003 projection
  exists.
- Outcome: The plan/task checkpoint is ready to commit and push before source
  work.
- Blocker or next safe action: Commit and push this checkpoint, then implement
  only the source-shaped matcher, task/plan binding, and production-file
  regression.

## 2026-08-12T21:38:00Z | Matcher correction and regression

- Source commit: `27049ae9727936f55e898aa02786e9509b3ae2df`
- Actions: Replaced the two-occurrence stabilization token with the exact
  multiline sleep expression at the intended use site; rebound the projector
  and fixtures to this plan/task; and added a projector regression that reads
  the real production adapter through Bazel runfiles and substitutes it at the
  production `git show` admission seam.
- Verification: The regression proves the old token still occurs exactly
  twice while the complete production file now passes the configured fragment
  set and reports 500 ms stabilization before ASIC enable. The first Bazel
  analysis correctly rejected a cross-package raw `filegroup`; replacing it
  with the required package-local `js_library` preserved the test unchanged,
  after which the full automation suite and the focused Rust core-voltage
  contract tests passed.
- Evidence: Source and tests only; no PWR-003 projection or candidate exists
  and no hardware command ran.
- Outcome: Root cause is fixed at the semantic matcher boundary with a
  production-file regression. A simplification review found no narrower
  robust shape: the source must be an explicit runfile to prevent fixtures
  from masking production drift.
- Blocker or next safe action: Run every mandatory implementation gate,
  commit and push the exact clean source, then invoke the projector once.

## 2026-08-12T21:48:00Z | Complete implementation gate

- Source commit: `27049ae9727936f55e898aa02786e9509b3ae2df`
- Actions: Completed the matcher, immutable lineage binding, Bazel runfile,
  and production-source regression without changing firmware behavior or the
  evidence schema.
- Verification: `cargo fmt --all`, strict Cargo Clippy, all-target Cargo
  build, all-feature Cargo tests, Bright Builds with zero findings, all 41
  Bazel tests, parity report/progress, focused automation and Rust contract
  tests, generated contracts, independent PWR-002 source validation, exact
  source and plan digests, unchanged production ownership paths, pinned
  reference cleanliness, redaction across 15 artifacts, unique task binding,
  absent final/candidate paths, and diff checks passed. The first standalone
  source-validator call used a Bazel-run relative path and failed to locate the
  file; its absolute-path rerun passed without repository changes.
- Evidence: Source and tests only. No new projection exists and no hardware
  command ran.
- Outcome: The exact implementation is ready to commit and push before the
  one permitted projection attempt.
- Blocker or next safe action: Commit and push, require a clean synchronized
  exact HEAD, then invoke the sealed PWR-003 projector once.

## 2026-08-12T21:52:00Z | Single projection succeeded

- Source commit: `a2fefad3b5863b0162747d98cdd1033878745a7a`
- Actions: From clean synchronized pushed source, invoked the one permitted
  PWR-003 projector against the sealed accepted PWR-002 evidence; no hardware
  command was used.
- Verification: The projector returned `complete`; the independent final
  validator passed; the public file is mode `0644`; the candidate is absent;
  and sensitive-value scanning found no forbidden match.
- Evidence:
  `docs/parity/evidence/pwr003-core-voltage-control/core-voltage-control-projection.json`
  with SHA-256
  `11dd1abbf6fda86d203fdcff49b420ab5139e1d29c35f4d17000c61c3112ae68`.
- Outcome: The complete closed PWR-003 quorum supports `verified`; RESULT.md
  records the conclusion and explicit non-claims.
- Blocker or next safe action: Run evidence checkpoint integrity/redaction
  gates, commit and push the projection/result without changing the checklist,
  then transition only PWR-003 and synchronize progress.

## 2026-08-12T21:55:00Z | Redaction false-positive remediation

- Source commit: `a2fefad3b5863b0162747d98cdd1033878745a7a`
- Actions: The first repository-wide scan of the newly published projection
  rejected one safe aggregate key because the prohibited `ip` pattern matched
  those two letters inside `compatible_path_count`. Tightened only the IP-key
  alternative to semantic `_`/`-`/string boundaries and extended the
  core-voltage redaction regression to admit the aggregate while still
  rejecting an explicit IP-address key.
- Verification: Diagnosis used schema/key names only and exposed no value.
  The projection was not edited and the projector was not rerun.
- Evidence: Existing projection SHA-256 remains
  `11dd1abbf6fda86d203fdcff49b420ab5139e1d29c35f4d17000c61c3112ae68`.
- Outcome: This is a host redaction-policy false positive, not an evidence or
  device failure. The correction preserves the prohibited operational-field
  policy while removing the substring collision.
- Blocker or next safe action: Rerun focused redaction tests and the complete
  evidence checkpoint gates against the existing sole projection.

## 2026-08-12T22:03:00Z | Evidence checkpoint verified

- Source commit: `a2fefad3b5863b0162747d98cdd1033878745a7a`
- Actions: Retained the sole projection byte-for-byte, completed the narrowly
  bounded redaction-key correction, and prepared RESULT.md without changing
  the parity checklist.
- Verification: Focused automation/redaction tests, `cargo fmt --all`, strict
  Cargo Clippy, all-target Cargo build, all-feature Cargo tests, Bright Builds
  with zero findings, all 41 Bazel tests, parity report/progress, repository
  redaction across 16 artifacts, independent final validation, pinned-reference
  cleanliness, exact evidence digest, mode `0644`, candidate absence,
  sensitive-value absence, and diff checks passed.
- Evidence: Projection SHA-256 remains
  `11dd1abbf6fda86d203fdcff49b420ab5139e1d29c35f4d17000c61c3112ae68`;
  RESULT.md binds it to implementation commit `a2fefad3` and reference commit
  `c1915b0a`.
- Outcome: Evidence and its validation policy are ready to commit and push as
  `SOURCE_COMMIT` before checklist mutation.
- Blocker or next safe action: Commit and push this evidence checkpoint, then
  use the typed transition command for PWR-003 and immediately synchronize
  progress.
