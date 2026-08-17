# Parity work log

## 2026-08-17T08:22:20Z | immutable attempt-015 plan

- Source commit: `4e9f32820716df85c73783ced779791c6cdd972c`
- Actions: Selected STAT-001 after SELF-001/BAP-002 blockers; froze one fresh
  v15/v9 attempt-015 command, unit, safety, privacy, recovery, retry, stop, and
  promotion contract around pushed diagnostic fix `c3b0dcb9`.
- Verification: Clean synchronized source/reference, private-input presence
  metadata, fresh-path absence, full ordered plan gate, and immutable digest
  pass.
- Evidence: Plan SHA-256
  `da3c3eb4fa4d4a9f949307db2b0e6e905f4e905ad31352a271a0b52ff1096205`.
- Outcome: Immutable hardware plan committed/pushed before rebind or effects.
- Blocker or next safe action: Rebind only attempt-015 surfaces, gate, push,
  rebuild exact package, then run only PLAN commands.

## 2026-08-17T08:30:00Z | attempt-015 software rebind checkpoint

- Source commit: pending implementation checkpoint
- Actions: Rebound ordinal, roots, immutable plan/task admission, Rust
  validator, generated contract, Bazel plan input, and fixtures from consumed
  attempt-014 to fresh attempt-015.
- Verification: Pending focused/full software, firmware, privacy, reference,
  exact-source, generated-contract, package, plan-hash, and diff gates.
- Evidence: Result v15/network v9, coherent read outcomes, earliest atomic
  tuple, priority 5, 18-source identity, and production behavior are unchanged.
- Outcome: Rebind ready for verification; no device access performed.
- Blocker or next safe action: Pass gates, push exact source, rebuild package,
  then execute only the frozen detector and conditional capture.

## 2026-08-17T08:51:56Z | attempt-015 terminal checkpoint

- Source commit: `1892800bbaf4eba2dd1d5c076699b41ed09908a1`
- Actions: Rebuilt/validated the clean package, ran the sole detector, checked
  only modes/presence/provenance metadata, and consumed the sole attempt-015
  capture. No retry or out-of-band device probe ran.
- Verification: Exact package/runtime identity, attestation, active safety,
  same-package state, terminal HTTP/WebSocket/pool state, safe stop, USB
  cleanup, protected modes, redaction, result/network digests, and projection
  withholding pass.
- Evidence: Capture closed after 364,110 active ms and 12/20 windows at
  `watchdog_feed_stale/stable/handling_inbox/not_waiting`; work renewal remains
  incomplete. The v15/v9 result and wrapper expose the identical closed tuple.
- Outcome: `stop_hardware_blocker`; STAT-001 remains `implemented` and no
  checklist/progress transition is permitted.
- Blocker or next safe action: Add a closed inbox/feedback/effect subphase and
  real-boundary regression before any attempt-016 contract.
