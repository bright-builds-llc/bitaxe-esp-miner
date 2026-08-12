# Parity work plan

- Run ID: `20260812T154751Z-API-009`
- Parity row: `API-009`
- Initial status: `implemented`
- Source commit: `4fb85ff5df13a40424117a75bc3b01db78e25b0f`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api009-command-effect-evidence-audit`
- Continues plan: `docs/parity/work-plans/20260812T144217Z-API-009/PLAN.md`

## Selection

The clean synchronized selector reports no open plan and ranks `API-009`
first. No row is skipped. Attempt-001 ended before device admission, so the
same row remains actionable after new host-only evidence identified the exact
pre-effect failure.

The deployed `rules_js` launcher exposes its generated inner Node shell wrapper
as `process.execPath`. The automation child environment deliberately excludes
private `JS_BINARY_*` variables. Spawning that wrapper therefore exits
immediately with `JS_BINARY__NODE_BINARY` unset. Readiness polling did not race
the child outcome and converted the early exit into a generic timeout. The same
fixture and process adapter pass outside the patched launcher; an inert
production-launcher reproduction fails at this exact wrapper boundary without
touching USB.

The active lesson set is 25,256 bytes with a conservative 8,419-token estimate,
above both loading limits. The valid 2026-08-03 audit baseline consumes that
crossing. Complete relevant safety, authorization, evidence, retry, redaction,
USB, and process-boundary blocks were loaded. Omitted global blocks are
`lesson-use-source-vtt-for-caption-fixes`, `lesson-zsh-lowercase-path-mutates-path`,
`lesson-macos-host-stalls-separate-policy-from-cache`, and
`lesson-prefer-exact-row-selection-for-small-dedup`. Omitted repository blocks
are `lesson-gsd-frontmatter-body-separators`,
`lesson-native-usb-capture-needs-prearmed-observation-or-replay`,
`lesson-boot-proof-replay-must-outlive-service-sessions`,
`lesson-heartbeat-cannot-prove-over-silent-transport`,
`lesson-manual-removal-needs-owner-observation`,
`lesson-cold-boot-proof-needs-an-independent-observer`,
`lesson-esp-idf-main-task-runtime-capacity`,
`lesson-http-liveness-is-not-response-readiness`, and
`lesson-evaluator-identity-binds-transitive-validators`. This is a flagged
budgeted lesson load, not a new audit trigger.

## Scope and non-scope

Replace the implicit `process.execPath` child with a repo-owned Bazel
`js_binary` fixture executable resolved from runfiles by the existing tool
locator. Race the private readiness document against child completion. An early
launch exception or nonzero exit becomes `process_failed`; a still-running
child that produces no readiness remains `timeout`. Preserve the earliest
category through cleanup and write only mode-`0600` closed child diagnostics:
exit/timing facts, a bounded safe category, and stdout/stderr digests, never raw
output, origins, addresses, ports, paths, credentials, or traces.

Add a real deployed-layout integration test using `createLocalProcessPort`, the
repo-owned fixture executable, a private mode-enforced root, readiness, stop,
and report files. Cover early exit, readiness timeout, cleanup, replay/freshness,
and public redaction. The test must execute the Bazel/runfiles child boundary;
an injected fake is not sufficient.

After a clean pushed fix, run exactly one fresh `attempt-002`. This retry is
eligible only because the exact failing boundary is now identified and the
real deployed-layout regression must pass. Reuse the existing command-effects
transaction, exact clean package, detector admission, conservative 600-second
lease, local easy-target fixture, one-time physical identify checkpoints,
safe-stop, cleanup, and canonical one-request software restart.

No external pool, owner pool credential, diagnostic state setter, erase, OTA,
rollback, power cycling, direct UART, pin manipulation, fault injection,
voltage/frequency/fan override, or unbounded mining is allowed. Reference source
remains pinned and read-only.

## Implementation

- [ ] Build and resolve the local Stratum fixture as its own repo-owned
      executable instead of spawning `process.execPath`.
- [ ] Race child completion with readiness and emit a protected closed launch
      diagnostic while preserving the earliest typed category.
- [ ] Add real `createLocalProcessPort` and deployed-runfiles regressions for
      success, early exit, timeout, cleanup, modes, and redaction.
- [ ] Re-run all software gates, commit, and push the exact source before any
      device effect.
- [ ] Run one fresh detector-gated `attempt-002` and publish evidence only for
      the complete API-009 quorum.

## Verification and promotion

Run focused script, process, automation, flash, device-session, and firmware
tests, including an inert reproduction through the actual `bazel run`
launcher. Then run, in order:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Also require `just verify-redaction`, `just verify-reference`, generated
contracts, selector and unique task binding, immutable-plan digest, reference
cleanliness, sensitive-output review, fresh attempt/projection paths,
`git diff --check`, and final diff review. Commit and push this plan/task
checkpoint before implementation, and commit and push verified source before
hardware.

For hardware, build the exact pushed package, capture a fresh private detector
result, require exactly one admitted board-205 ESP32-S3 port, and launch the
existing `just api-command-effects-campaign` interface with `attempt-002` and a
fresh public projection. Ask only for physical IDENTIFY rendered/cleared
observations when their private checkpoints appear; standing authorization
covers the attempt and no renewed permission is required.

Promotion requires the complete five-command quorum, a genuine network-target
ASIC result, both physical identify observations, same physical device,
exactly one software restart, exact build, changed boot session, ordinal `N+1`,
safe stop, cleanup, recovery status, and redaction. Otherwise keep API-009
`implemented`, preserve the first typed terminal category, withhold public
evidence, close the plan truthfully, and stop without another retry.
