# STR-007 validator-boundary worklog

## 2026-08-12T13:37:13Z | Fresh selection and validator-boundary plan

- Source commit: `b90d88c77dbc093d1ad7388a292c99856baf5f72`
- Actions: Confirmed clean synchronized `main`, clean reference, terminal
  closures for both prior ordinals, and absence of candidate and projection.
  Selected `STR-007` again without skipping a row.
- Verification: The second failure was isolated to the external validator's
  repository-relative input under Bazel's runfiles working directory; the
  closed projector and internal candidate validation had succeeded.
- Evidence: Only committed public source and closure records were inspected.
  No protected evidence, credentials, network, or hardware was accessed.
- Outcome: A repository-owned absolute-path boundary can remove this host
  orchestration ambiguity without changing the evidence contract.
- Blocker or next safe action: Run and seal the plan-only gate, push the
  immutable plan, then implement the narrow validator command and regression.

## 2026-08-12T13:40:06Z | Validator-boundary plan gate sealed

- Plan SHA-256: `58424e52830a91acc8586d2a82b3089cb740f29d3b7e64767cc12101fa304922`
- Actions: Ran the complete ordered plan-only gate and all plan-specific
  absence, digest, redaction, reference, and cleanliness checks.
- Verification: Format, clippy, all-target build, all-feature tests, Bright
  Builds checks, all 37 Bazel test targets, parity, progress, redaction,
  reference, immutable plan digest, projection absence, and diff checks pass.
- Evidence: No projection, candidate, protected input, network, credential, or
  hardware action occurred.
- Outcome: The immutable plan is ready to commit and push before implementation.
- Blocker or next safe action: Commit and push the plan/task records, then add
  the absolute-path validator command and focused child-boundary regression.

## 2026-08-12T13:50:54Z | Absolute validator boundary sealed

- Actions: Added `just validate-mining-criteria-evidence`, which requires an
  existing input and canonicalizes it with `/bin/realpath` before invoking the
  existing Rust validator through Bazel. Added a real-child regression with a
  fake Bazel boundary that requires exactly one existing absolute file path,
  and exposed the Justfile to the sandboxed automation test runfiles.
- Verification: Focused test iterations caught stdout capture assumptions,
  macOS `/var` canonicalization, typed command construction, host-tool lookup,
  and missing Justfile runfiles data; each was fixed at the boundary. The final
  focused test and 222-case automation target pass. Format, clippy, all-target
  build, all-feature tests, Bright Builds checks, all 37 Bazel test targets,
  parity, progress, redaction, reference, plan digest, projection absence,
  reference cleanliness, and diff checks pass.
- Evidence: No projection, candidate, protected input, credentials, network,
  or hardware was accessed.
- Simplification: The solution adds one seven-line human command surface and
  one boundary test while reusing the existing Rust validator unchanged.
- Outcome: The validator boundary is ready to commit and push.
- Blocker or next safe action: Commit and push the guarded boundary, confirm a
  clean synchronized head, then run the single projector plus independent
  validator transaction allowed by this plan.
