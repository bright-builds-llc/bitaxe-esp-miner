# Parity work plan

- Run ID: `20260814T023531Z-API-009`
- Parity row: `API-009`
- Initial status: `implemented`
- Source commit: `0ae0842149e98d05e7ce03bf10071fd7071a2355`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api009-command-effect-evidence-audit`
- Continues plan: `docs/parity/work-plans/20260814T014014Z-API-009/PLAN.md`

## Selection

The clean synchronized selector has no open plan and ranks API-009 first, so
no candidate is skipped. Attempt-013 flashed and admitted the exact package,
trusted runtime identity, fresh safety, and ready USB cleanup, but consumed its
campaign before a genuine notification or command checkpoint after only two
milliseconds active.

Pushed commit `0ae08421` deterministically reproduces the pre-active expiry and
replaces its preparation-anchored resumable wall clock with a bounded activation
phase followed by one continuous resumable active epoch. Exact-boundary and
pause/resume regressions prove the epoch cannot expire before first active,
activation timeout becomes inapplicable after activation, and pause/resume does
not reset the epoch. Rust capture and the checked TypeScript parent/fixture
budgets now cover both 600-second phases and terminal cleanup, while a distinct
typed activation-timeout category preserves the true failure boundary.

Focused and real-process tests, every mandatory gate, and the real ESP firmware
build pass. This is new objectively verified production-orchestration
information, not an ordinal-only retry, so this plan authorizes exactly one
attempt-014 and no attempt-015.

## Scope and non-scope

Build and package exact clean pushed HEAD, admit exactly one Ultra 205 through
the protected detector, and run one fresh `attempt-014` through
`just api-command-effects-campaign`. Use fresh mode-`0700` attempt, wrapper,
and detector roots plus mode-`0600` files. Keep the campaign attached to its
closed checkpoint stream.

Allowed effects are exact-package USB flash/reset; private Wi-Fi and generated
local-fixture NVS seed; conservative BM1366 initialization and local-fixture
mining for at most 600 seconds after a separately bounded activation phase;
one pause; one resume; two IDENTIFY toggles; one genuine notification
dismissal; one canonical software restart; same-device recovery; safe stop;
and cleanup. Consume `ready` only after the user confirms they are watching.
Consume `rendered` only when they report the exact blank / `BITAXE IDENTIFY` /
`Hello!` / blank frame while its 30-second checkpoint is live. Consume
`cleared` only when they report that frame absent.

Do not use an external or owner pool; infer, automate, pre-confirm, or reuse a
physical observation; read prior protected attempts; expose origins,
hostnames, ports, USB/network identities, credentials, workers, addresses,
passwords, tokens, sensor values, or raw traces; or weaken the five-command
quorum. No erase, OTA, rollback, power cycle, direct UART, pin/pad/GPIO
manipulation, probe, jumper, soldering, injected signal, fault injection,
voltage/frequency/fan override, non-205 device, attempt-015, or unchanged retry
is in scope.

## Implementation and verification

- [ ] Commit and push this immutable plan/task checkpoint before package,
      detector, credential, USB, network, mining, HTTP, display, or restart
      effects.
- [ ] Re-run focused timing/budget/timeout/checkpoint/campaign/CLI/real-process
      tests, every mandatory software/privacy/reference gate, real firmware
      build, package validation, and exact clean-pushed identity admission.
- [ ] Require ignored `wifi-credentials.json` to be non-empty without reading
      it. Require fresh wrapper, detector, and attempt-014 paths plus the public
      projection path to be absent before creating private wrapper/detector
      roots.
- [ ] Run the detector exactly once and continue only on one board-205
      ESP32-S3, ready typed handoff, private modes, and no holder.
- [ ] Invoke exactly once:
      `just api-command-effects-campaign --private-root scratch/api009-command-effects/attempt-014 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/api009-command-effects/detector-014/detector.stdout --projection docs/parity/evidence/api009-command-effects/command-effects-projection.json --duration-seconds 600`.
- [ ] Consume `ready`, `rendered`, and `cleared` exactly once each only after
      their matching live user reports using the repo-owned confirmation
      command and the attempt-014 campaign root.
- [ ] Promote only on the complete independently validated command and restart
      quorum; otherwise keep `implemented`, preserve the earliest typed failure,
      withhold evidence, record cleanup/recovery, and stop without attempt-015.

Before plan commit and final source/evidence commit, run in order: Cargo format,
strict Clippy, all-target build, all-feature tests, Bright Builds, `just test`,
`just parity`, and `just parity-progress`. Also run focused tests,
`just verify-redaction`, `just verify-reference`, `just build`, plan digest,
unique task binding, selector closure, exact package validation, private-mode
and holder checks, sensitive-output review, `git diff --check`, and full diff
review.

Campaign start consumes attempt-014. Promote only if the closed projection
proves genuine network-target block notification, active pause/resume with the
same-session safe-stop join, operator readiness before IDENTIFY, both matching
physical IDENTIFY observations, dismissal with block-count preservation,
exactly one canonical software restart, same physical device, exact build,
changed boot session, ordinal `N+1`, safe stop, cleanup, recovery, private
modes, and redaction. Otherwise close as `blocked`, withhold the projection,
and do not create attempt-015.
