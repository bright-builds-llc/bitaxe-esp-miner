# Parity work plan

- Run ID: `20260813T110706Z-API-009`
- Parity row: `API-009`
- Initial status: `implemented`
- Source commit: `ecb19811feaae5494af38a6fdd8cf3a17ba10f4e`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api009-command-effect-evidence-audit`
- Continues plan: `docs/parity/work-plans/20260813T100428Z-API-009/PLAN.md`

## Selection

The clean synchronized selector ranks API-009 first, so no row is skipped.
Attempt-009 stopped before pause confirmation because operator-resumable pause
synchronously used the terminal eight-step shutdown, whose 120-second fresh-
temperature cooling proof blocked the production owner inside the host's
130-second pause join. The immediately preceding software plan fixed that exact
boundary at commit `ecb19811`: production now selects a closed
`ResumablePause` purpose whose six immediate fail-closed effects retain full fan
duty without the terminal cooling wait, while terminal, fault, shutdown, lease
consumption, and preparation rollback retain the full eight-step plan.

A composed production-session-to-actuation-to-same-lease-confirmation test and
focused host/campaign/ownership tests prove the corrected boundary. This is a
material production change to the attempt-009 failure, not an ordinal-only,
timing, instrumentation, or unchanged retry. This plan therefore explicitly
supersedes only the prior attempt-010 prohibition and authorizes exactly one
attempt-010 after every plan, software, package, privacy, recovery, and detector
gate passes. It does not authorize attempt-011. The user's current report that
the display shows new information confirms presence and readiness, but it does
not satisfy either future IDENTIFY observation; each confirmation must follow
its matching live closed checkpoint.

The active lesson set remains above its deterministic loading budget with the
unchanged 2026-08-03 audit baseline and no new audit trigger. Complete safety,
authorization, evidence, retry, redaction, physical-observation,
earliest-failure, real-process, ESP-IDF, device-session, and host-stall blocks
are loaded. Caption/VTT, small-table deduplication, and legacy GSD separator
blocks remain disclosed unrelated omissions. Repo-local hardware/privacy
guidance plus the architecture, code-shape, verification, testing, Rust, and
TypeScript standards govern this continuation.

## Scope and non-scope

Run one fresh `attempt-010` through the existing repo-owned
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

Do not use an external or owner pool input; infer, automate, pre-confirm, or
reuse a display observation; weaken the five-command quorum; read prior
protected attempts; or expose origins, hostnames, ports, USB/network
identities, credentials, workers, addresses, passwords, tokens, checkpoint
secrets, paths, sensor values, or raw traces. No diagnostic setter, erase, OTA,
rollback, power cycle, direct UART, pin/pad/GPIO manipulation, probe, jumper,
soldering, injected signal, fault injection, voltage/frequency/fan override,
non-205 device, attempt-011, or unchanged retry is in scope.

## Implementation

- [ ] Persist and push this immutable plan/task checkpoint before any package,
      detector, credential, USB, network, mining, HTTP, display, or restart
      effect.
- [ ] Revalidate the resumable-pause shutdown split, same-session join, and
      live checkpoint supervisor at focused and real-process boundaries, then
      pass every mandatory software, privacy, reference, and exact-package
      gate at clean pushed HEAD.
- [ ] Run exactly one fresh detector-gated attempt-010, relay each live closed
      IDENTIFY requirement, and issue its request-once confirmation only after
      the matching user-observed physical state.
- [ ] Publish and promote API-009 only on the complete five-command quorum;
      otherwise keep `implemented`, retain the earliest typed failure, withhold
      public evidence, prove cleanup/recovery, and stop without attempt-011.

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

Also require the focused resumable-pause actuation, production session,
campaign marker, API-command-effects, checkpoint, and real-child-process tests;
the real firmware build; `just verify-redaction`; `just verify-reference`;
generated/build source ownership; immutable-plan digest; unique task binding;
selector closure; reference cleanliness; sensitive-output review; fresh-root
and projection absence; `git diff --check`; and full diff review.

After the plan commit is pushed and the worktree is clean and synchronized:

1. Re-run the focused pause/safe-stop/session/checkpoint tests and real firmware
   build.
2. Run `just package`; require the package manifest source commit to equal
   exact clean pushed HEAD and its reference commit to equal the pinned
   reference.
3. Require non-empty ignored `wifi-credentials.json` without reading it.
   Require `scratch/api009-command-effects/wrapper-010`,
   `scratch/api009-command-effects/detector-010`,
   `scratch/api009-command-effects/attempt-010`, and the public projection to
   be absent, then create only private mode-`0700` wrapper/detector parents.
4. Run exactly once with separate private streams:
   `just detect-ultra205 > scratch/api009-command-effects/detector-010/detector.stdout 2> scratch/api009-command-effects/detector-010/detector.stderr`.
   Continue only when exactly one board-205 ESP32-S3 is admitted and no holder
   or cleanup blocker exists.
5. Invoke exactly once:
   `just api-command-effects-campaign --private-root scratch/api009-command-effects/attempt-010 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/api009-command-effects/detector-010/detector.stdout --projection docs/parity/evidence/api009-command-effects/command-effects-projection.json --duration-seconds 600`.
6. After the live closed `rendered` signal and only after the user reports the
   matching rendered physical state, invoke exactly once:
   `just confirm-api-command-identify --evidence-dir scratch/api009-command-effects/attempt-010/campaign --observation rendered`.
7. After the live closed `cleared` signal and only after the user reports the
   matching cleared physical state, invoke exactly once:
   `just confirm-api-command-identify --evidence-dir scratch/api009-command-effects/attempt-010/campaign --observation cleared`.

Campaign start consumes attempt-010. Promote only if the independently
validated closed projection proves genuine network-target block notification,
active pause/resume with the corrected same-session safe-stop join, both
matching physical IDENTIFY observations, dismissal with block-count
preservation, exactly one canonical software restart, same physical device,
exact build, changed boot session, ordinal `N+1`, safe stop, cleanup, recovery,
private modes, and redaction. Otherwise withhold the projection, select the
truthful typed terminal category and one closed outcome from
`stop_repeated_boundary`, `stop_hardware_blocker`, `stop_authority_boundary`,
or `stop_impossible_contract`, and do not create attempt-011.
