# Parity work log

## 2026-08-18T10:53:25Z | attempt-003

- Source commit: `60a56d4935ced15eeb5ec6950b1ad4ea35fdf223`.
- Actions: rotated the scoreboard plan/task/ordinal/paths/contracts/fixtures/
  runfiles to attempt-003; ran all software gates; built the exact clean pushed
  package; ran the sole wrapper-003 detector; then started the sole attempt-003
  capture with protected Wi-Fi/pool inputs. No raw runtime values were read or
  published.
- Verification: focused automation and Rust contract targets passed. Ordered
  Cargo format, Clippy, all-target build, and all-feature tests passed. Bright
  Builds, all 47 Bazel targets, parity, progress, firmware build/package,
  redaction, reference cleanliness, exact package identity, detector admission,
  protected modes, and campaign result sealing passed.
- Evidence: the sealed campaign recorded accepted status, 600,746 active ms,
  20/20 renewed windows, 24 qualified plus 151 below-pool candidates, accepted
  submit, trusted identity, fresh safety, stable watchdog, accepted network v12,
  natural serial closure with final consumed state, terminal HTTP/WebSocket/pool,
  confirmed safe stop, and ready USB cleanup. The private 20-entry scoreboard
  repeated identically and the live SPA route passed. One exact-package restart
  changed boot session, incremented ordinal once, reported `software_cpu`, and
  kept `startMiningOnBoot=false`, but its closed non-active mining state was
  `paused` rather than the verifier's sole accepted `safe_blocked` spelling.
- Outcome: attempt-003 stopped as public `hardware_blocked`; post-restart
  scoreboard reads and projection remained withheld. The terminal-settlement
  correction passed its real boundary. The remaining failure is a verifier
  model that conflates disabled boot mining with one of two closed non-active
  runtime states.
- Blocker or next safe action: close without parity transition. A fresh
  software-only plan must define boot mining disabled as `startMiningOnBoot`
  false plus either `paused` or `safe_blocked`, reject active/unknown states,
  and use that single predicate in restart admission and evidence. Only a later
  immutable hardware plan may consider attempt-004 after the fix is gated,
  pushed, and package-bound.
