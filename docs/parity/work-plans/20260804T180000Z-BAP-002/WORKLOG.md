# BAP-002 worklog

## 2026-08-04T18:00:00Z | plan created

- Source commit: `ff2d1114c7e08868ffd58ce4cb523dfcca81364d`.
- Actions: Selected `BAP-002` through the deterministic parity selector and
  inspected the pinned protocol vocabulary, checksum, parser compatibility,
  request handlers, subscription admission, setting validation, and current
  `bitaxe-core` ownership boundary.
- Verification: Confirmed the ten command tokens, eighteen parameter tokens,
  256-byte bound, XOR framing, checksum-free subscription compatibility,
  one-second duplicate window, AP-mode error values, request projections, and
  safe pure/imperative split.
- Evidence: `PLAN.md`, pinned reference breadcrumbs, current Rust/Bazel source,
  and the active parity checklist.
- Outcome: The pure protocol core is actionable without an accessory, UART,
  credentials, network activity, hardware control, or device interaction.
- Blocker or next safe action: Commit the immutable plan/task checkpoint, then
  implement the pure wire contract before handler decisions.
