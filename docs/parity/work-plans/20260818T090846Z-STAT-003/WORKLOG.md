# Parity work log

## 2026-08-18T09:36:00Z | attempt-002

- Source commit: `e9034ea11d4d9de5ebea9c34198a2f26b49a387b`.
- Actions: rotated the scoreboard workflow to attempt-002 and this immutable
  plan; added rejection of consumed attempt-001; bound the Rust evidence model
  and independent validator into the 31-path source identity; ran every
  software gate; built the exact clean pushed package; ran the sole wrapper-002
  detector; then started the sole authorized attempt-002 capture with protected
  Wi-Fi/pool inputs. No raw values were read or published.
- Verification: focused automation and Rust contract targets passed. Ordered
  Cargo format, Clippy, all-target build, and all-feature tests passed. Bright
  Builds, all 47 Bazel test targets, parity, progress, firmware build/package,
  redaction, reference cleanliness, package/source identity, detector admission,
  protected modes, and result sealing passed.
- Evidence: the sealed campaign recorded `accepted`, 600,148 active ms, 20/20
  renewed windows, 24 qualified plus 178 below-pool candidates, accepted submit,
  trusted runtime identity, fresh safety, no watchdog/panic/mixed-reset failure,
  terminal HTTP/WebSocket/pool confirmation, confirmed safe stop, and ready USB
  cleanup. Network v12 recorded final consumed and serial-finished observations
  plus `accepted_after_serial_close`, but `terminal_close_requested=false` and
  network status `failed`; the public scoreboard projection was withheld.
- Outcome: attempt-002 failed closed as public `evidence_invalid`. The source
  acceptance model overconstrains a diagnostic: it requires the worker to have
  requested serial closure even when the analyzer naturally closed serial input
  before the worker observed complete settlement.
- Blocker or next safe action: close this plan without parity transition. A
  fresh software-only plan must accept either valid closure initiator while
  still requiring final consumed state, serial finish, accepted settlement,
  terminal transports, pool persistence, and every existing safety gate. Only
  a later immutable hardware plan may authorize attempt-003 after that fix is
  gated, pushed, and package-bound.
