# Parity work plan

- Run ID: `20260816T033934Z-STAT-001`
- Parity row: `STAT-001`
- Initial status: `implemented`
- Source commit: `34768147feea166354dc97044ea1d5e12dce939a`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat001-hashrate-monitor`

## Selection

The clean synchronized selector reported no open plan and ranked `UI-001`,
`UI-002`, `UI-003`, `SELF-001`, `BAP-002`, then `STAT-001`. UI-001 and UI-002
need trusted physical panel observation, UI-003 needs trusted physical button
observation, SELF-001 still lacks a production-safe hardware execution route,
and BAP-002 depends on BAP-001 plus an authorized compatible accessory and
qualified electrical UART setup that are unavailable. STAT-001 is the first
actionable row. Attempt-003 stopped at a source-owned producer/parser boundary:
`runtime_boot_attestation=unavailable` diagnostics were falsely admitted as
stable markers and every admitted line collapsed to `malformed`. Pushed commit
`f26fff55c1513f342946f16999d8564cc761ba01` fixes that real production boundary
with one shared complete-token matcher, source-shaped regressions, and closed
parse-failure diagnostics. This is verified progress and permits one fresh
attempt-004; it is not an unchanged retry.

The two active lesson inputs total 29,963 bytes with a conservative summed
estimate of 9,990 tokens, so the startup audit flag remains active and bounded
whole-block loading was used. Every global lesson plus the repository service-
ownership/redaction, opaque-handoff, real-process, espflash reset, USB/power,
native-capture, boot-replay, transport-heartbeat, direct-electrical-authority,
protected-root, earliest-failure, private-classification, retry-progress,
qualified-transport, evaluator-identity, flash-versus-monitor, standing-
authorization, preflight-exit, and telemetry-state blocks informed this plan.
The repository blocks not loaded were `lesson-gsd-frontmatter-body-separators`,
`lesson-manual-removal-needs-owner-observation`,
`lesson-physical-usb-identity-excludes-enumeration-fields`,
`lesson-cold-boot-proof-needs-an-independent-observer`,
`lesson-esp-idf-main-task-runtime-capacity`,
`lesson-http-liveness-is-not-response-readiness`,
`lesson-time-bounded-physical-checkpoints-must-be-prearmed-and-self-describing`,
and `lesson-never-invite-ready-before-live-checkpoint`. The 2026-08-03 audit
already consumed the hard-limit crossing; five, not ten, new lessons exist and
neither the 90-day nor proposed-append trigger is active, so no new audit is
required. Repo-local guidance, the Bright Builds sidecar and empty override
table, and the architecture, code-shape, verification, testing, and Rust
standards were reviewed.

## Scope and non-scope

Advance only STAT-001. Rebind the existing private-first
`bitaxe-hashrate-monitor-evidence-v1` workflow, independent Rust validator,
generated TypeScript contract, task/plan admission, Bazel runfiles, and tests
from consumed attempt-003 to fresh attempt-004. Admit campaign-result schema
v10, which is the current production schema after the marker-boundary fix, and
preserve its sealed, closed runtime-attestation parse discriminator for any
non-ready outcome without exposing field values or source text. Preserve the
existing conservative campaign, pinned reference semantics, exact current-
source admission, projection schema, hashrate quorum, and fail-closed behavior.

After all implementation and package gates pass at clean pushed HEAD, the sole
hardware attempt may factory-flash one exact board-205 package, perform normal
USB reset/re-enumeration, seed only ignored local Wi-Fi and pool credentials,
derive a same-origin target only from the protected current-session serial
stream, run the repository's conservative 400 MHz / 1100 mV / 100% fan profile
for exactly 600 accumulated active seconds, observe HTTP, reconstructed
WebSocket, and serial state, pause and safe-stop, and use at most one exact-
package recovery flash after a post-flash failure. Credentials remain opaque
runtime inputs and are checked only for nonempty presence.

Raw hashrates, sensor values, credential fields, owner/worker strings, pool
endpoints, hostnames, origins, ports, USB/network/process identity, HTTP or
WebSocket bodies, logs, commands, PIDs, and traces remain `ProtectedOperational`
or `NeverPersistRaw` as defined by the evidence policy. They may exist only in
mode-`0600` files under ignored mode-`0700` roots when permitted, and never in
terminal or Git output. Only a closed independently validated projection may
be published. No upstream-default or overclock profile, arbitrary control
target, automatic fan mode, unbounded mining, OTA, erase, raw write, fault
injection, physical power action, external UART, or pin/pad/header/GPIO/probe/
jumper/solder/signal manipulation is authorized. Analog accuracy, electrical
measurement, profitability, dynamic retuning, extended soak, general share-
outcome parity, other ASICs/boards, update/recovery behavior, and release
readiness remain non-claims.

## Implementation

- [ ] Rebind task/plan admission, private roots, generated contracts, Rust
      validator, Bazel runfiles, and fixtures to attempt-004.
- [ ] Admit sealed campaign-result v10 and regression-test the exact current
      task/plan, production/reference sources, real child command, protected
      layout, schema, stage/profile, and incomplete-quorum boundaries.
- [ ] Preserve only the closed value-free runtime-attestation parse signature
      on a sealed non-ready campaign and prove sensitive child material cannot
      reach the public failure envelope.
- [ ] Run every required software, firmware, privacy, reference, package, and
      exact-source gate; commit and push before detector or credential access.
- [ ] Execute only the detector and conditional attempt-004 commands below,
      then publish and validate evidence only on the complete promotion quorum.

## Verification and promotion

Before hardware, run focused hashrate core, runtime-attestation, campaign,
automation, independent-validator, real-child, task/plan, source/reference,
generated-contract, privacy, protected-mode, seal, and failure-precedence
tests. Run `just verify-redaction`, `just verify-reference`, the canonical
package build, and the mandatory final sequence in order:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Commit, fetch/rebase only if conflict-free and necessary, and push the exact
implementation. Rebuild and validate the exact clean package after that push.
Only then may these commands run in order:

1. `test ! -e scratch/stat001-hashrate-monitor/wrapper-004 && (umask 077; mkdir -m 700 -p scratch/stat001-hashrate-monitor/wrapper-004 && just detect-ultra205 > scratch/stat001-hashrate-monitor/wrapper-004/detector.stdout 2> scratch/stat001-hashrate-monitor/wrapper-004/detector.stderr)`
2. Only after command 1 exits zero, admits exactly one Ultra 205 through
   `espflash board-info --chip esp32s3 --non-interactive`, cleanup and holder
   checks pass, both ignored credential files are nonempty without being read,
   and the supervisor child and public projection remain absent:
   `test ! -e scratch/stat001-hashrate-monitor/attempt-004 && test ! -e docs/parity/evidence/stat001-hashrate-monitor/hashrate-monitor-projection.json && test -s wifi-credentials.json && test -s pool-credentials.json && (umask 077; just capture-hashrate-monitor-evidence --private-root scratch/stat001-hashrate-monitor/attempt-004 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --pool-credentials pool-credentials.json --detector-output scratch/stat001-hashrate-monitor/wrapper-004/detector.stdout --projection docs/parity/evidence/stat001-hashrate-monitor/hashrate-monitor-projection.json --duration-seconds 600 --capture-timeout-seconds 1500 > scratch/stat001-hashrate-monitor/wrapper-004/capture.stdout 2> scratch/stat001-hashrate-monitor/wrapper-004/capture.stderr)`

The wrapper root must remain mode `0700`; detector and capture streams are
distinct mode-`0600` files; the supervisor-owned attempt child and public
projection must be absent immediately before launch. Starting command 2
consumes attempt-004. Preserve the earliest typed failure through the base
campaign's bounded safe stop, recovery, seal, and cleanup. No unchanged retry
or attempt-005 is authorized. Classify the attempt as exactly one of
`complete`, `stop_repeated_boundary`, `stop_hardware_blocker`,
`stop_authority_boundary`, or `stop_impossible_contract`. A recurrence of the
post-fix authoritative false-marker signature stops as repeated; a newly
discriminating closed signature may justify later software diagnosis but does
not authorize another attempt here.

Promote STAT-001 only if the independent validator proves board 205, attempt 4,
the exact clean pushed source/reference/package, one detector-admitted device,
trusted runtime identity, one-second monitor cadence and pinned register
semantics, exactly one ASIC and four domains, active mining and work renewal in
all twenty half-open 30-second windows, at least two positive changing coherent
hashrate observations in each transport, positive current and all four rolling
windows after warmup, finite bounded error, terminal zero current rate in both
transports, safe stop, USB/process cleanup, protected modes, exact seals,
independent validation, and redaction. On success create `RESULT.md`, commit
the evidence, transition only STAT-001 to `verified` with
`unit,workflow,api-compare,hardware-smoke,hardware-regression`, synchronize
progress to that evidence commit, archive the completed task, run the final
gates, review the diff, commit, and push. Any missing fact withholds promotion,
keeps STAT-001 `implemented`, and requires a truthful `CLOSURE.md`.
