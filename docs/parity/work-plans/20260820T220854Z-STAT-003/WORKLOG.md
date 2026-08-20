# Parity work log

## 2026-08-20T22:30:00Z | sealed evidence recheck implementation

- Evaluator source: `d7ecc5066babe15a37d181bd4b799c235985f8fa`.
- Actions: added a read-only protected attempt-005 recheck, exact tree/mode/
  symlink and old-failure admission, current source inventory, campaign/system/
  SPA/restart/scoreboard recomputation, atomic independent validation, and
  redaction-safe CLI/Just/Bazel wiring.
- Verification: focused valid/tampered/mode/symlink/privacy/validator tests,
  Rust contract tests, ordered Cargo gates, Bright Builds, all 48 Bazel tests,
  firmware build/package, redaction, reference, parity/progress, selector, and
  diff checks passed. The evaluator and plan were committed and pushed cleanly.
- Package anchor investigation: a detached exact-source rebuild established
  that package-manifest bytes are not source-only deterministic. Rebuilding
  with the retained original build timestamp still produced a different app
  ELF identity because workspace/build paths affect the ELF hash.

## 2026-08-20T22:42:00Z | sole protected recheck

- The exact authorized command ran once at clean pushed evaluator source.
- Result: nonzero, projection absent, candidate absent, wrapper streams mode
  0600. Metadata-only audit proved the protected tree inventory, regular-file
  types, and private modes were complete.
- Redaction-safe diagnosis proved capture source/reference, before/after source/
  reference, changed boot session, ordinal +1, `software_cpu`, and disabled boot
  mining all match. The failure was the reconstructed app/package anchor, not
  campaign, restart, or durable scoreboard behavior.
- Outcome: stop without retry or promotion. The protected attempt remains
  immutable and no hardware/device/network effect occurred.
- Next safe action: a fresh software-only contract may represent the proof
  truthfully as a retained capture-package identity digest plus the old
  terminal boundary that proves the original manifest admission. It must not
  fabricate or relabel an unavailable package-manifest byte digest.
