# Parity work log

## 2026-08-13T02:49:57Z | Selection and root-gap audit

- Source commit: `b7a74b5642f2bcbcb709c99ec69d62390d407e9c`
- Actions: Re-ran the clean selector; skipped API-009 at its repeated-boundary
  stop and THR-001 at its missing bounded fault-stimulus contract; selected
  THR-002; audited the pinned fan-controller reference, current pure thermal
  controller, production safety owner, EMC2101 fan path, and accepted PWR-002
  projection/result lineage.
- Verification: Clean synchronized `main`; no open plan; reference commit
  `c1915b0a63bfabebdb95a515cedfee05146c1d50`; committed PWR-002 projection
  SHA-256 `0668c274d09b3e39d7d5edfea4b2e66c97248ff77de9192981f3af00e547ddfe`,
  mode `0644`, and independent Rust validation pass; no hardware command issued.
- Evidence: The accepted campaign proves the typed 100% fan command and fresh
  nonzero post-command RPM. The source audit proves the remaining gap is the
  absent production scheduler for the otherwise tested pure controller.
- Outcome: THR-002 is actionable by composing new workflow proof of a typed
  production scheduler with the existing immutable physical fan-response
  regression, without reflashing or repeating a fan effect.
- Blocker or next safe action: Run every plan-checkpoint gate, commit and push
  the immutable plan/task, then implement the narrow production controller
  runtime without changing the qualified raw fan actuation boundary.

## 2026-08-13T03:08:00Z | Immutable-plan verification

- Source commit: `b7a74b5642f2bcbcb709c99ec69d62390d407e9c`
- Actions: Froze the THR-002 plan at SHA-256
  `3adf53a19701e33ef195898560b7f8f17baef0f36033912bc86de54ace0d178d`
  and retained exactly one matching active task. No implementation, checklist,
  or evidence file changed.
- Verification: The ordered Cargo format, strict Clippy, all-target build, and
  all-feature test gates passed; Bright Builds reported zero findings; all 41
  Bazel tests passed; parity reported no validation errors; progress remained
  64 of 94 active rows verified (68.1%); focused thermal, EMC2101, and source-
  ownership tests passed; the PWR-002 Rust validator, repository redaction
  across 18 evidence artifacts, pinned-reference cleanliness, selector plan
  binding, task uniqueness, projection digest/mode, and diff checks passed.
- Evidence: Immutable plan and worklog under
  `docs/parity/work-plans/20260813T024957Z-THR-002/`; no THR-002 result or
  checklist transition exists.
- Outcome: The plan/task checkpoint is ready to commit and push before source
  implementation. The existing PWR-002 projection remains byte-identical and
  independently valid.
- Blocker or next safe action: Commit and push this checkpoint, then implement
  the typed production scheduler and focused regressions.

## 2026-08-13T03:36:00Z | Production controller implementation verified

- Source commit: `62f9680aa56ecc9c011ff78d43de90552f8240e9`
- Actions: Added a pure fan-controller planner and a separate 100 ms production
  shell; wired startup after the production owner; loaded only typed confirmed
  settings, mining state, and fresh producer thermal truth; retained PID state;
  suppressed unchanged duties; bounded failed-write retries to 2000 ms; and
  routed commands only through the existing typed safety queue. Control is
  deferred unless mining is already active, preserving the default boot claim
  that hardware control is disabled and avoiding races with the qualified
  preparation transaction.
- Verification: Nine focused planner tests cover qualification, stable cadence,
  mode priority, validated manual and automatic duties, PID retention,
  unchanged-duty suppression and reassertion, bounded retry, invalid settings,
  and invalid thermal truth. Source-ownership tests prove startup order, typed
  queue use, current settings/observation inputs, active-mining gate, and absence
  of raw I2C/EMC2101 ownership. The real normal and rollback ESP32-S3 firmware
  builds pass. The first real firmware compile exposed an incorrect host-only
  type name (`LoadedSettings`); replacing it with the exported
  `PersistenceDecision` fixed the production build without changing evidence
  or hardware state.
- Evidence: The ordered Cargo, Bright Builds, all-42-target Bazel, parity, and
  progress gates pass. Focused controller, source-ownership, EMC2101, PWR-002
  validator, redaction, and pinned-reference gates pass. The PWR-002 projection
  remains SHA-256
  `0668c274d09b3e39d7d5edfea4b2e66c97248ff77de9192981f3af00e547ddfe`
  at mode `0644`; all nine qualified PWR-002 source paths remain unchanged from
  the pushed plan commit. No hardware command was issued.
- Outcome: The implementation checkpoint is ready to commit and push. A
  simplification review retained the deep pure-plan/imperative-shell split and
  reused the existing queue instead of adding a second actuator owner or
  evidence schema.
- Blocker or next safe action: Commit and push the clean implementation, then
  revalidate exact source/evidence lineage and write the THR-002 result.
