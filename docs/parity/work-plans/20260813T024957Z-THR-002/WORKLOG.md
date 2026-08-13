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

## 2026-08-13T02:57:00Z | Immutable-plan verification

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

## 2026-08-13T03:10:00Z | Production controller implementation verified

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

## 2026-08-13T03:16:00Z | Qualification-edge correction

- Source commit: `d1bdbe0a2e45ffbdebbee53a2292a687286f625e`
- Actions: The post-push safety review found that using mining activity as the
  scheduler gate made paused and no-pool modes unreachable. Replaced that gate
  with a producer-owned atomic qualification published only while an admitted
  campaign is authorizing, its state is `Active`, and the existing safety and
  ASIC owners are available. The production owner clears qualification before
  safe stop and on every non-active/no-campaign publication. No raw owner or
  hardware contract changed.
- Verification: Focused planner and source-ownership tests pass, including the
  explicit active-campaign and pre-safe-stop clearing boundaries; the real
  ESP32-S3 firmware build passes. The controller now admits paused and no-pool
  priority decisions only inside the existing campaign envelope while default
  boot and preparation remain non-authorizing.
- Evidence: Software/source verification only; no device, detector, package,
  serial, network, fan, voltage, mining, or recovery command ran.
- Outcome: The safety gate and upstream mode reachability are both preserved.
- Blocker or next safe action: Re-run the complete implementation gates, commit
  and push this correction, then write the row-specific result.

## 2026-08-13T03:19:00Z | Closed-evidence composition

- Source commit: `ca81411093fb0a81e31cc556c07bd05a8d05343b`
- Actions: Revalidated the accepted PWR-002 projection, result, exact identities,
  ancestry, mode, and redaction; compared every qualified source path through
  current `main`; and added only the THR-002 row-specific result. The later
  EMC2101 temperature-offset change does not intersect the byte-identical fan
  register/write span.
- Verification: PWR-002 projection SHA-256
  `0668c274d09b3e39d7d5edfea4b2e66c97248ff77de9192981f3af00e547ddfe`;
  PWR-002 result SHA-256
  `199509d8f95dab4287f4d3c3a7b09b381823250ff990c7ee7ad1a612ffbf6b9c`;
  THR-002 plan SHA-256
  `3adf53a19701e33ef195898560b7f8f17baef0f36033912bc86de54ace0d178d`;
  independent Rust validation and mode `0644` pass; implementation and evidence
  commits are ancestors of current `main`; reference remains pinned and clean.
- Evidence: Existing closed PWR-002 projection/result, passing controller and
  source-ownership workflow tests, and
  `docs/parity/work-plans/20260813T024957Z-THR-002/RESULT.md`. No new schema or
  hardware artifact was created.
- Outcome: The complete composed THR-002 quorum supports `verified`. Reusing the
  accepted physical projection is smaller and stronger than duplicating it or
  rerunning a fan effect.
- Blocker or next safe action: Run the complete evidence-checkpoint gates,
  commit and push RESULT.md without checklist mutation, save that commit as
  `SOURCE_COMMIT`, then transition only THR-002 and synchronize progress.

## 2026-08-13T03:21:00Z | Evidence checkpoint verified

- Source commit: `ca81411093fb0a81e31cc556c07bd05a8d05343b`
- Actions: Preserved the immutable plan and reused projection byte-for-byte;
  finalized only the row-specific RESULT.md and task/worklog checkpoint without
  changing the parity checklist.
- Verification: The ordered Cargo format, strict Clippy, all-target build, and
  all-feature tests passed; Bright Builds reported zero findings; all 42 Bazel
  tests passed; parity reported no validation errors; progress remained 64 of
  94 active rows verified (68.1%); independent evidence validation, repository
  redaction, pinned-reference cleanliness, normal and rollback firmware builds,
  exact digests, mode `0644`, unique task binding, ancestry, fan-span source
  compatibility, and diff checks passed.
- Evidence: THR-002 RESULT.md SHA-256
  `3505303892578beb884bce95521a6714c0818866c3244e56ca2e45b3c6f52186`;
  reused projection SHA-256
  `0668c274d09b3e39d7d5edfea4b2e66c97248ff77de9192981f3af00e547ddfe`.
- Outcome: The evidence checkpoint is ready to commit and push as
  `SOURCE_COMMIT` before any checklist mutation.
- Blocker or next safe action: Commit and push this checkpoint, then perform
  one typed THR-002 transition and immediately synchronize progress.

## 2026-08-13T03:22:00Z | Worklog clock correction

- Source commit: `9b03c11893da91e10eae506b268cfb7bd10e62a5`
- Actions: Corrected five worklog heading times that were manually entered
  ahead of the actual UTC clock. The corrected ordering is derived from the
  authoritative commit times for plan `62f9680a`, implementation `d1bdbe0a`,
  qualification correction `ca814110`, and result checkpoint `9b03c118`.
- Verification: Plan, result, reused projection, source, task, and checklist
  content are unchanged. Their SHA-256 values and all prior gate outcomes remain
  unchanged; only the five inaccurate heading timestamps changed.
- Evidence: Git commit metadata and this append-only correction record.
- Outcome: The worklog chronology is accurate before transition publication.
- Blocker or next safe action: Commit and push the correction, then transition
  only THR-002 from that exact clean evidence source.
