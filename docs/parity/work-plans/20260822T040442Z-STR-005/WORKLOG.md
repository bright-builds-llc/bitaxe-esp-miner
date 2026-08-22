# Parity work log

## 2026-08-22 04:04 UTC | Immutable audit plan

- Source commit: `d2e0835ab07a1b32521a6bfe5ce4576acda8c974`
- Actions: Recorded explicit deferred activation, pinned-reference scope, the
  local Noise pool campaign contract, privacy rules, safe stop, recovery,
  retry bound, and promotion boundary.
- Verification: Ordered Rust gates, Bright Builds, reference cleanliness, and
  diff checks passed.
- Evidence: `PLAN.md` and the updated active `TASKS.md` block.
- Outcome: Plan commit `b9679882cc94747a28b2880333126c1c9208e4b1`
  was pushed before implementation.
- Blocker or next safe action: Repair the selector without rewriting immutable
  historical plans.

## 2026-08-22 04:17 UTC | Explicit activation

- Source commit: `b9679882cc94747a28b2880333126c1c9208e4b1`
- Actions: Ignored metadata-less historical plans only for already-verified
  rows, retained deferred rows outside automatic ranking, and permitted only
  audit-plan-bound `deferred -> in-progress` activation.
- Verification: Focused selector/transition tests, ordered Rust gates, Bright
  Builds, parity validation, progress validation, and diff checks passed.
- Evidence: Transition `20260822T041700Z-STR-005` and synchronized progress
  record at 90 verified of 95 active rows.
- Outcome: Governance commit `4718a9e5` was pushed; `STR-005` is truthfully
  `in-progress` with pending evidence.
- Blocker or next safe action: Implement bounded protocol behavior without any
  device or network effect.

## 2026-08-22 05:02 UTC | Pure protocol checkpoint

- Source commit: `4718a9e5` plus the current working diff.
- Actions: Added official `noise_sv2` 1.4.2 with default features disabled;
  bounded frame and message codecs; standard/extended channel, job, target,
  share, and BM1366 work state; authenticated Noise transport poisoning and
  nonce-budget guards; and pinned-reference golden vectors.
- Verification: 19 focused Cargo SV2 tests pass, including two real initiator/
  responder Noise handshakes. Canonical
  `bazel test //crates/bitaxe-stratum:tests` passed all crate tests and compiled
  the secp256k1 C dependency through Bazel. Focused Clippy passes with warnings
  denied.
- Evidence: `crates/bitaxe-stratum/fixtures/stratum-v2-protocol-vectors.json`
  binds the pinned reference commit and exact source files.
- Outcome: Pure protocol and crypto boundaries are ready for an atomic source
  commit; no network, USB, device, credential, mining, or hardware effect
  occurred.
- Blocker or next safe action: Add the sole firmware SV2 owner and keep it
  mutually exclusive with the existing V1 production owner.

## 2026-08-22 06:08 UTC | Software implementation checkpoint

- Source commit: `ab6e89ac` plus the current working diff.
- Actions: Added the sole campaign-gated firmware V2 owner, protocol selection,
  bounded same-protocol fallback, private Base58Check authority loading, ESP
  hardware RNG, bounded encrypted transport, shared ASIC execution, watchdog
  feeds, PSRAM/safety admission, complete safe stop, reverse-direction pool
  codecs, a real TCP Noise fixture, private V2 NVS admission, and ordered V2
  campaign marker validation. Bound the S3 cc-rs build to the target compiler,
  archiver, and `-mlongcalls`.
- Verification: 23 pure V2 tests, one real TCP fixture lifecycle, 393 flash
  tests, firmware source-ownership tests, canonical `just build`, all 52 Bazel
  tests, `just package`, Bright Builds, and the full Cargo sequence passed. The
  Cargo license report was regenerated after reviewing the exact
  MIT/Apache-2.0, CC0-1.0, and CDLA-Permissive-2.0 package licenses.
- Evidence: `docs/parity/evidence/str005-stratum-v2/summary.md` and the
  provenance-bound protocol fixture.
- Outcome: The pinned-reference software surface supports `implemented` with
  `unit,golden,workflow`; no credential, network, USB, device, mining, or other
  hardware effect occurred.
- Blocker or next safe action: Implement and test the exact-restoration outer
  `just stratum-v2-campaign` transaction before attempt-001 can become effect
  eligible. Do not substitute the lower-level fixture or mining-campaign stage.

## 2026-08-22 06:19 UTC | Implemented promotion

- Source commit: `abf6c1bdfaf3f929f2fea30ec630635262221755`
- Actions: Transitioned only `STR-005` from `in-progress` to `implemented` and
  bound the exact implementation pointers, evidence types, notes, plan digest,
  reference commit, and source evidence. A second same-status transition
  corrected the pointers to the checklist's required Markdown code-span shape
  without rewriting the original receipt.
- Verification: Transition replay validation and deterministic progress sync
  accepted the exact checklist mutation. Verified-row count remains 90 of 95
  active rows because no hardware claim was made.
- Evidence: Transitions `20260822T061900Z-STR-005` and
  `20260822T062500Z-STR-005-TARGETS`, plus
  `docs/parity/evidence/str005-stratum-v2/summary.md`.
- Outcome: `implemented` with `unit,golden,workflow`; task remains active.
- Blocker or next safe action: Complete the outer restoration transaction and
  all hardware-effect gates before invoking attempt-001 or claiming `verified`.

## 2026-08-22 06:45 UTC | Outer campaign transaction

- Source commit: `8ec85eb8` plus the current working diff.
- Actions: Added the exact `just stratum-v2-campaign` command, immutable argument
  parser, pre-effect current-origin settings/theme capture, protected local-input
  reconstruction, exact prior-package discovery, owned fixture supervision,
  private V2 mining-campaign invocation, success/failure package and settings
  restoration, bounded child output, closed public projection, and a separate
  validator process.
- Verification: Parser, exact/ambiguous restore-package inventory, accepted and
  forbidden projection, TypeScript compile, real fixture, flash campaign, and
  complete automation tests pass. Failure recovery attempts restoration while
  preserving the original typed outcome.
- Evidence: `tools/automation/src/stratum-v2-campaign.ts`,
  `tools/automation/src/stratum-v2-campaign-validator.ts`, and their tests.
- Outcome: The immutable outer hardware transaction is implemented; no hardware
  or external-network effect occurred.
- Blocker or next safe action: Commit and push the command, rebuild/package exact
  HEAD, run all effect gates, then invoke attempt-001 once. A missing or
  ambiguous prior package is a pre-effect `hardware_blocked` result, not grounds
  to weaken restoration.
