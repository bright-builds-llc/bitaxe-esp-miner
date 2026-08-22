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
