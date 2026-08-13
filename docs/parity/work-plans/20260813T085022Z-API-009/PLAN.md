# Parity work plan

- Run ID: `20260813T085022Z-API-009`
- Parity row: `API-009`
- Initial status: `implemented`
- Source commit: `c6ae688afb853f011e2203b5ca607f018812941b`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api009-command-effect-evidence-audit`
- Continues plan: `docs/parity/work-plans/20260813T070749Z-API-009/PLAN.md`

## Selection

The clean synchronized selector ranks API-009 first, so no candidate is
skipped. Attempt-008 proved the marker-v12 same-session pause/safe-stop join on
hardware: the campaign observed a genuine block, accepted shares, pause,
resumable safe stop, resume, and active mining after resume. It then stopped
only because the live rendered-IDENTIFY checkpoint expired without a matching
physical-observation reply. The prior closure permits later work only through
a separately selected plan with new retry authority and a fresh live
physical-observation contract.

The user's current-thread report that the display shows new information and
their request to continue establishes the missing pre-effect condition: the
operator is present, watching the display, and ready to answer a future live
checkpoint. It is not retroactive attempt-008 evidence and does not confirm
either attempt-009 IDENTIFY state. This objectively changes the authority
boundary without changing firmware or weakening the evidence contract, so one
fresh attempt-009 is actionable under standing task authorization.

The active lesson set remains above its deterministic loading budget with the
unchanged 2026-08-03 audit baseline and no new audit trigger. Complete safety,
authorization, evidence, retry, redaction, physical-observation,
earliest-failure, real-process, ESP-IDF, device-session, and host-stall blocks
are loaded. Caption/VTT, small-table deduplication, legacy GSD separator, and
manual-removal blocks remain disclosed unrelated omissions. Repo-local
hardware/privacy guidance plus the architecture, code-shape, verification,
testing, Rust, and TypeScript standards govern this continuation.

## Scope and non-scope

Run one fresh `attempt-009` through the existing repo-owned
`just api-command-effects-campaign` transaction at the exact clean pushed plan
commit. Bind the package to that commit, admit exactly one Ultra 205 through a
protected detector, use fresh mode-`0700` attempt/wrapper/detector roots and
mode-`0600` files, retain the caller on the closed checkpoint stream, and issue
each typed request-once IDENTIFY confirmation only after the user reports the
corresponding rendered or cleared physical display state while that checkpoint
is live.

The allowed effects are exact-package USB flash/reset; private Wi-Fi and
generated local-fixture NVS seed; conservative BM1366 initialization and
local-fixture mining for at most 600 seconds; one pause; one resume; two
IDENTIFY toggles; one genuine notification dismissal; one canonical software
restart; same-device recovery; safe stop; and cleanup. Recovery is
pause/safe-stop first and exact-package restoration only when the closed
workflow requires it. Preserve the earliest typed failure; cleanup and
recovery remain secondary.

Do not use an external pool or owner pool input; infer, automate, or pre-confirm
a display observation; reuse an expired checkpoint; weaken the five-command
quorum; read prior protected attempts; or expose origins, hostnames, ports,
USB/network identities, credentials, workers, addresses, passwords, tokens,
checkpoint secrets, paths, sensor values, or raw traces. No diagnostic setter,
erase, OTA, rollback, power cycle, direct UART, pin/pad/GPIO manipulation,
probe, jumper, soldering, injected signal, fault injection,
voltage/frequency/fan override, non-205 device, or attempt-010 is in scope.

## Implementation

- [ ] Persist and push this immutable plan/task checkpoint before any package,
      detector, credential, USB, network, mining, HTTP, display, or restart
      effect.
- [ ] Revalidate the marker-v12 pause/safe-stop join and live checkpoint
      supervisor at focused and real-process boundaries, then pass every
      mandatory software, privacy, reference, and exact-package gate at clean
      pushed HEAD.
- [ ] Run exactly one fresh detector-gated attempt-009, relay each live closed
      IDENTIFY requirement, and issue its request-once confirmation only after
      the matching user-observed physical state.
- [ ] Publish and promote API-009 only on the complete five-command quorum;
      otherwise keep `implemented`, retain the earliest typed failure, withhold
      public evidence, prove cleanup/recovery, and stop without attempt-010.

## Verification and promotion

Before the plan commit, run in order:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Also require the focused pause/safe-stop join and API-command-effects tests,
the real firmware build, `just verify-redaction`, `just verify-reference`,
generated/build source ownership, immutable-plan digest, unique task binding,
selector closure, reference cleanliness, sensitive-output review, fresh-root
and projection absence, `git diff --check`, and full diff review.

After the plan commit is pushed and the worktree is clean and synchronized:

1. Re-run the focused marker/network/checkpoint tests and real firmware build.
2. Run `just package`; require the package manifest source commit to equal
   exact clean pushed HEAD and its reference commit to equal the pinned
   reference.
3. Create only private wrapper/detector parents for
   `scratch/api009-command-effects/wrapper-009` and
   `scratch/api009-command-effects/detector-009`; require the campaign-owned
   `scratch/api009-command-effects/attempt-009` and public projection to be
   absent.
4. Run `just detect-ultra205` once with separate private stdout/stderr files.
   Continue only when exactly one board-205 ESP32-S3 is admitted and no holder
   or cleanup blocker exists.
5. Invoke exactly once:
   `just api-command-effects-campaign --private-root scratch/api009-command-effects/attempt-009 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/api009-command-effects/detector-009/detector.stdout --projection docs/parity/evidence/api009-command-effects/command-effects-projection.json --duration-seconds 600`.
6. After the live closed `rendered` signal and only after the user reports the
   matching rendered physical state, invoke exactly once:
   `just confirm-api-command-identify --evidence-dir scratch/api009-command-effects/attempt-009/campaign --observation rendered`.
7. After the live closed `cleared` signal and only after the user reports the
   matching cleared physical state, invoke exactly once:
   `just confirm-api-command-identify --evidence-dir scratch/api009-command-effects/attempt-009/campaign --observation cleared`.

Campaign start consumes attempt-009. Promote only if the independently
validated closed projection proves genuine network-target block notification,
active pause/resume with the same-session safe-stop join, both matching
physical IDENTIFY observations, dismissal with block-count preservation,
exactly one canonical software restart, same physical device, exact build,
changed boot session, ordinal `N+1`, safe stop, cleanup, recovery, private
modes, and redaction. Otherwise withhold the projection, select the truthful
typed terminal category and one closed outcome from `stop_repeated_boundary`,
`stop_hardware_blocker`, `stop_authority_boundary`, or
`stop_impossible_contract`, and do not create attempt-010.
