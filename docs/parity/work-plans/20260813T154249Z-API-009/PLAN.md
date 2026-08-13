# Parity work plan

- Run ID: `20260813T154249Z-API-009`
- Parity row: `API-009`
- Initial status: `implemented`
- Source commit: `fca339cd98b3273f678d2f7347d82e016354dfdf`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api009-command-effect-evidence-audit`
- Continues plan: `docs/parity/work-plans/20260813T144901Z-API-009/PLAN.md`

## Selection

The clean synchronized selector has no open plan and ranks API-009 first, so
no candidate is skipped. Attempt-010 already proved exact-package identity, a
genuine block, five accepted shares, pause, resumable safe stop, resume, and
active-after-resume. Its physical IDENTIFY boundary expired because the host
sent the 30-second effect before establishing that the operator was watching
and its public signal did not describe the expected frame.

Pushed commits `5c99f38d` and `9d632439` fix that exact boundary: the campaign
now emits and consumes a request-once `ready` checkpoint while the IDENTIFY
request count remains zero, then emits self-describing `rendered` and
`cleared` checkpoints after the enable and disable requests. Focused Rust,
TypeScript, canonical Bazel, and real-child-process regressions prove the new
ordering. Commit `fca339cd` repairs the closure metadata contract and the
selector passes. This is new production-boundary information, not an ordinal-
only or unchanged retry, so this plan authorizes exactly one attempt-011. It
does not authorize attempt-012.

The active lesson set exceeds the deterministic startup budget. The unchanged
2026-08-03 audit baseline has one later lesson and no distinct audit trigger.
Complete authorization, safety, retry, physical-observation, redaction,
evidence-root, earliest-failure, real-process, flash-versus-monitor, USB
identity, and host-stall blocks are loaded. Caption/VTT, small-table
deduplication, legacy GSD separator, and unrelated manual-removal blocks remain
disclosed omissions. Repo-local hardware/privacy guidance and the verification,
testing, Rust, and TypeScript standards govern this continuation.

## Scope and non-scope

Build and package exact clean pushed HEAD, admit exactly one Ultra 205 through
the protected detector, and run one fresh `attempt-011` through
`just api-command-effects-campaign`. Use fresh mode-`0700` attempt, wrapper,
and detector roots plus mode-`0600` files. Keep the campaign process attached
to its closed checkpoint stream.

The allowed effects are exact-package USB flash/reset; private Wi-Fi and
generated local-fixture NVS seed; conservative BM1366 initialization and
local-fixture mining for at most 600 seconds; one pause; one resume; two
IDENTIFY toggles; one genuine notification dismissal; one canonical software
restart; same-device recovery; safe stop; and cleanup. First consume `ready`
only after the user confirms they are watching. Then consume `rendered` only
when the user reports the exact blank / `BITAXE IDENTIFY` / `Hello!` / blank
frame while its 30-second checkpoint is live. Consume `cleared` only when the
user reports that frame absent after the disable request. Recovery remains
pause/safe-stop first and exact-package restoration only when required by the
closed workflow. Preserve the earliest typed failure; cleanup and recovery are
secondary.

Do not use an external or owner pool input; infer, automate, pre-confirm, or
reuse a physical observation; read prior protected attempts; expose origins,
hostnames, ports, USB/network identities, credentials, workers, addresses,
passwords, tokens, paths, sensor values, or raw traces; or weaken the complete
five-command quorum. No erase, OTA, rollback, power cycle, direct UART,
pin/pad/GPIO manipulation, probe, jumper, soldering, injected signal, fault
injection, voltage/frequency/fan override, non-205 device, attempt-012, or
unchanged retry is in scope.

## Implementation

- [ ] Commit and push this immutable plan/task checkpoint before package,
      detector, credential, USB, network, mining, HTTP, display, or restart
      effects.
- [ ] Re-run focused checkpoint/campaign/CLI and real-process tests, every
      mandatory software/privacy/reference gate, the real firmware build, and
      exact-package admission at clean pushed HEAD.
- [ ] Run exactly one detector-gated attempt-011 and consume `ready`,
      `rendered`, and `cleared` only after their matching live user reports.
- [ ] Promote API-009 only on the complete independently validated command and
      restart quorum; otherwise keep `implemented`, withhold public evidence,
      prove cleanup/recovery, and stop without attempt-012.

## Verification and promotion

Before the immutable plan commit, run in order:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Also run the focused flash checkpoint/campaign/CLI and automation checkpoint,
orchestration, and real-child tests; `just verify-redaction`;
`just verify-reference`; `just build`; immutable-plan digest; unique task
binding; selector closure; reference cleanliness; sensitive-output review;
fresh-root/projection absence; `git diff --check`; and full diff review.

After the plan commit is pushed and the worktree is clean and synchronized:

1. Re-run the focused checkpoint/campaign/CLI and real-child tests, mandatory
   privacy/reference gates, and real firmware build.
2. Run `just package`. Require manifest source commit equal exact pushed HEAD,
   reference commit equal the pinned reference, and valid package admission.
3. Require ignored `wifi-credentials.json` to exist and be non-empty without
   reading it. Require
   `scratch/api009-command-effects/wrapper-011`,
   `scratch/api009-command-effects/detector-011`,
   `scratch/api009-command-effects/attempt-011`, and
   `docs/parity/evidence/api009-command-effects/command-effects-projection.json`
   to be absent. Create only private wrapper/detector roots.
4. Run exactly once with separate private streams:
   `just detect-ultra205 > scratch/api009-command-effects/detector-011/detector.stdout 2> scratch/api009-command-effects/detector-011/detector.stderr`.
   Continue only when exactly one board-205 ESP32-S3 is admitted and no holder
   or cleanup blocker exists.
5. Invoke exactly once:
   `just api-command-effects-campaign --private-root scratch/api009-command-effects/attempt-011 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/api009-command-effects/detector-011/detector.stdout --projection docs/parity/evidence/api009-command-effects/command-effects-projection.json --duration-seconds 600`.
6. On the live `ready` signal, explain the exact frame and duration to the user.
   Only after they confirm they are watching, invoke exactly once:
   `just confirm-api-command-identify --evidence-dir scratch/api009-command-effects/attempt-011/campaign --checkpoint ready`.
7. On the live `rendered` signal, consume it only after the user reports the
   exact frame, using `--checkpoint rendered`. On the live `cleared` signal,
   consume it only after the user reports the frame absent, using
   `--checkpoint cleared`.

Campaign start consumes attempt-011. Promote only if the closed projection
proves genuine network-target block notification, active pause/resume with the
same-session safe-stop join, operator readiness before IDENTIFY, both matching
physical IDENTIFY observations, dismissal with block-count preservation,
exactly one canonical software restart, same physical device, exact build,
changed boot session, ordinal `N+1`, safe stop, cleanup, recovery, private
modes, and redaction. Otherwise select the earliest typed terminal category,
withhold the projection, close as `blocked`, and do not create attempt-012.
