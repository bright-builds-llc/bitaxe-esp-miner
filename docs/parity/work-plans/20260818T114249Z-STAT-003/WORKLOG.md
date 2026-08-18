# Parity work log

## 2026-08-18T12:18:02Z | attempt-004

- Source commit: `ca972836528ba11dd228ca0f11260e84ab90e9fd`.
- Actions: rotated the scoreboard workflow to attempt-004; ran all software
  gates; built the exact clean pushed package; ran the sole wrapper-004
  detector; then started the sole attempt-004 capture with protected Wi-Fi/pool
  inputs. No raw runtime values were read or published.
- Verification: focused automation and Rust contract targets passed. Ordered
  Cargo gates, Bright Builds, parity/progress, firmware build/package, redaction,
  reference, exact package identity, detector admission, protected modes, and
  result sealing passed. Exact `just test` timed out twice only inside the agent
  sandbox with no assertion failure; the identical 900-second Bazel graph then
  passed all 47 targets in 72.4 seconds, and the later exact pre-hardware gate
  also passed all 47 targets.
- Evidence: the sealed campaign stopped at 229,579 active ms with closed
  `network_unavailable`, 8/20 covered windows, 84 below-pool candidates, zero
  qualified candidates, and no submit response. Runtime identity and safety
  were trusted/fresh; watchdog, panic, mixed reset, and correlation diagnostics
  were clean; terminal HTTP/WebSocket/pool and final consumed settlement passed;
  safe stop and USB cleanup were confirmed.
- Outcome: public `hardware_blocked`; API/SPA/restart and projection were
  withheld. This is distinct from the fixed natural-closure and paused-restart
  verifier signatures and supplies no accepted scoreboard persistence evidence.
- Blocker or next safe action: close without parity transition and do not retry.
  Another scoreboard attempt requires an objectively renewed pool/network
  availability signal under a fresh immutable plan; changing only ordinal or
  timing is insufficient. Other actionable parity rows may proceed meanwhile.
