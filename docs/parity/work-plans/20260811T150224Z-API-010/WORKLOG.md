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
- Verification: The ordered Cargo format, strict Clippy, all-target build, and
  all-feature tests passed. Bright Builds reported zero findings; all 36 Bazel
  tests, parity, progress, redaction, reference cleanliness, the exact plan
  selector, and diff checks passed. The selector checks caught and corrected
  the uncommitted parity-row key and predecessor-plan lineage metadata.
- Outcome: Plan commit `5becba9f9fce805c05a13732c0c98116a8c2973e`
  is pushed. Immutable plan SHA-256 is
  `26af4ac6711842bf91c8d928461adc39c580dddaa5d2448757b0e8d743de36e7`.
- Next safe action: Commit and push this checkpoint, rebuild the exact package
  from that clean source identity, then run the sole protected detector.

## 2026-08-11T15:18:00Z | attempt-010 terminal result

- Exact source: `d677069601e82804687502942252315ba57afe84`.
- Package admission: schema v3, six required artifacts, matching source and
  reference identities, and clean source truth passed.
- Detector: Passed. The private wrapper and output modes passed, and exactly
  one likely Ultra 205 port was present. This objectively changes the prior
  ROM-synchronization boundary.
- Conditional campaign: Stopped before the campaign binary because the
  documented passthrough used `stage=observation`; the canonical CLI requires
  `--stage observation`. Protected stderr contains only the argument-rejection
  boundary and remained mode 0600.
- Device effects: None from the campaign invocation. The attempt root was not
  created; no USB session, package flash, NVS write, credential read, runtime
  observation, mining, or hardware control occurred.
- Outcome: `process_failed`, detail `cli_argument_rejected`. Attempt-010 is
  consumed without retry. No parity evidence or `RESULT.md` exists, and
  `API-010` remains `implemented`.
- Next safe action: Add a focused exact-observation CLI regression, push one
  corrected immutable attempt-011 plan, rebuild the exact package, and reuse
  no attempt-010 path or command.
