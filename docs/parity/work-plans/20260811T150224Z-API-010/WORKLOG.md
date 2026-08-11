# Ultra 205 boot recovery work log

## 2026-08-11T15:02:24Z | history reconciliation and plan

- Source baseline: `1bb26b4de1a552b129b5f2cf6bf5e93305ccae80`.
- Reference baseline: `c1915b0a63bfabebdb95a515cedfee05146c1d50`.
- Recent-history finding: the CFG-005 implementation and verification were
  software-only. They did not alter the board. The last confirmed installed
  package predates the screen-stack fix at `50287f62`.
- Current symptom: the user freshly connected the board and reports the same
  approximately one-second display blink seen during the prior panic loop.
- Feedback loop: one protected detector is the exact red-capable ROM-sync
  boundary. It is followed by one safe observation campaign only on success.
- Safety choice: use the existing observation campaign so the post-flash NVS
  seed explicitly contains `mineonboot=false` and no pool credentials or live
  hardware lease.
- Outcome: Planning checkpoint pending verification and push.
