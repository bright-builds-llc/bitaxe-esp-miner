# Parity work plan

- Run ID: `20260817T005227Z-REL-003`
- Parity row: `REL-003`
- Initial status: `implemented`
- Source commit: `4bf594cf9f0cabd391881a6cee0e4ab0024a9151`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-rel003-large-erase-recovery`

## Selection

The worktree and pinned reference are clean, `main` equals `origin/main`, and
the deterministic selector reports no open plan. Candidate order begins
`SELF-001`, `BAP-002`, `STAT-001`, `STAT-003`, then `REL-003`.

`SELF-001` remains blocked because no production-safe firmware route exists
for its hardware self-test modes. `BAP-002` remains blocked by unfinished
`BAP-001` UART/subscription ownership and the absence of an authorized live
accessory path. `STAT-001` is closed at `stop_repeated_boundary`; another
hardware attempt is prohibited until new closed discriminators and a targeted
verified fix objectively change its producer-owned watchdog boundary.
`STAT-003` remains dependency-blocked because verified live scoreboard
population requires parsed-response-backed ASIC/share outcomes while
`ASIC-11` and `STR-09` remain unfinished and the qualified mining campaign is
under the STAT-001 stop.

REL-003 is the first actionable row. Existing Phase 18/19 evidence proves the
release gate, provenance, package workflow, invalid-image rejection, valid OTA
response, detector/flash-monitor identity, and redaction. The later verified
REL-002 result and projection prove one reset-aborted interrupted application
update with unchanged baseline, one exact pending-validation probe, native
rollback to the exact factory build, passive safe state, restoration, cleanup,
and redaction. Those accepted artifacts now satisfy the release verifier's
rollback, recovery, failed-update, and interrupted-update terms. Their explicit
non-claims identify large erase recovery as the one remaining REL-003 term.

The active lesson ledgers exceed the deterministic load limits. Every heading
was inventoried. All global lessons and complete repository blocks for safety,
authorization, privacy, evidence integrity, destructive retry policy,
transport qualification, failure precedence, and real process boundaries were
loaded within the whole-block budget. Omitted lower-priority blocks are the GSD
frontmatter lesson and historical USB power/session, prearmed native capture,
boot-replay lifetime, silent-transport heartbeat, manual-removal ownership,
physical-identity, and cold-boot-observer lessons; equivalent active repo-local
rules remain controlling. The latest audit baseline has only six later
lessons, is under 90 days old, and this work appends no lesson, so no distinct
audit trigger is due.

## Scope and non-scope

Advance only REL-003. Add a typed, private-first
`bitaxe-release-recovery-evidence-v1` contract and a plan-bound repo-owned
large-erase command. The host workflow must validate a clean exact package,
immutable task/plan, detector transcript, protected paths, and opaque Wi-Fi
input before effects. It may issue exactly one supervised ESP32-S3 full-flash
erase, then restore the exact factory image plus an owner-supplied Wi-Fi NVS
seed with `mineonboot=false`, and use the qualified receive-only monitor to
prove exact runtime identity, SPIFFS/static readiness, passive safe state,
cleanup, modes, independent validation, and redaction.

Large erase intentionally removes all onboard flash contents, including NVS,
OTA state, applications, static assets, coredump data, pool settings, hostname,
theme, and operator tuning. The exact factory restore recreates the release
partition image, and the opaque ignored Wi-Fi input restores network access;
all other settings return to package defaults. Local ignored Wi-Fi and pool
credential files remain untouched and recoverable, but pool credentials are
not reseeded and mining must remain disabled. This data effect is required to
prove large-erase recovery and is limited to the one detector-admitted board
205.

If the primary restore fails before completing a flash transfer, the workflow
may issue one recovery-only exact factory flash with the same Wi-Fi seed. If
the primary flash transfer completes but runtime proof is missing, it must not
reflash unchanged; preserve flash completion separately and close on missing
proof. Every failure must preserve the earliest category through recovery and
cleanup. The projection is written only after independent validation.

No firmware behavior, partition layout, release artifact, OTA route, rollback
logic, mining path, sensor/control path, or UI behavior changes are in scope.
No OTAWWW, interrupted power, eFuse/anti-rollback, raw arbitrary write,
repeated erase, mining, pool connection, voltage/frequency/fan/thermal/power
control, physical power action, direct UART, or pin/pad/header/GPIO/probe/
jumper/solder/signal manipulation is authorized. Other boards, factory
provisioning at scale, electrical calibration, release signing, and commercial
release readiness remain non-claims.

## Implementation

- [ ] Add the narrow plan-bound `rel003-large-erase` flash subcommand with
      exact package admission, supervised USB ownership, one fixed
      `espflash erase-flash` vector, closed diagnostics, cleanup, and tests.
- [ ] Add the typed Rust evidence contract, independent validator, generated
      TypeScript surface, thin automation command, protected-root admission,
      primary restore and conditional pre-transfer recovery logic, redaction,
      and real-child regressions.
- [ ] Add Bazel/runfiles and `just capture-release-recovery-evidence` wiring;
      prove exact source/reference/plan/task binding and rejection of altered
      plans, paths, packages, modes, erase vectors, incomplete restore,
      missing safe state, post-transfer reflash, and private-value leakage.
- [ ] Run all focused and mandatory software, firmware, package, privacy,
      reference, release-gate, immutable-plan, source-admission, and diff
      gates; commit and push the exact implementation before device access.
- [ ] Run only the frozen detector and one conditional attempt-001 capture;
      promote REL-003 only when new large-erase evidence and every accepted
      prior release/rollback artifact jointly satisfy the full verifier.

## Authorized hardware commands and recovery

After implementation is clean, fully gated, committed, pushed, and repackaged
from that exact source, run only these commands in order:

1. `test ! -e scratch/rel003-large-erase/wrapper-001 && (umask 077; mkdir -m 700 -p scratch/rel003-large-erase/wrapper-001 && just detect-ultra205 > scratch/rel003-large-erase/wrapper-001/detector.stdout 2> scratch/rel003-large-erase/wrapper-001/detector.stderr)`
2. Only after command 1 exits zero, admits exactly one Ultra 205, all holder
   and cleanup checks pass, `wifi-credentials.json` is nonempty without being
   read, and the supervisor child/projection/candidate paths are absent:
   `test ! -e scratch/rel003-large-erase/attempt-001 && test ! -e docs/parity/evidence/rel003-large-erase/release-recovery-projection.json && test -s wifi-credentials.json && (umask 077; just capture-release-recovery-evidence --private-root scratch/rel003-large-erase/attempt-001 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/rel003-large-erase/wrapper-001/detector.stdout --plan docs/parity/work-plans/20260817T005227Z-REL-003/PLAN.md --projection docs/parity/evidence/rel003-large-erase/release-recovery-projection.json --capture-timeout-seconds 360 > scratch/rel003-large-erase/wrapper-001/capture.stdout 2> scratch/rel003-large-erase/wrapper-001/capture.stderr)`

The wrapper root must be mode `0700` with distinct mode-`0600` detector and
capture siblings. The supervisor exclusively creates the absent attempt child
as mode `0700` with mode-`0600` descendants. Starting command 2 consumes
attempt-001. Never reuse it or perform an unchanged retry. Stop before erase
on detector ambiguity/failure, non-205 identity, source/reference/package/
plan/task drift, existing output, invalid modes, missing Wi-Fi input, or USB
ownership failure. After erase begins, always attempt the bounded exact
restore/recovery path and release USB/process ownership. Accepted terminal
outcomes are `complete`, `stop_hardware_blocker`, `stop_repeated_boundary`,
`stop_authority_boundary`, and `stop_impossible_contract`.

## Verification and promotion

Before hardware, run focused flash-command, contract, generated-contract,
automation, real-child, recovery-precedence, task/plan, package, release-gate,
redaction, and reference tests. Run `just verify-redaction`,
`just verify-reference`, `just package`, the release gate against the current
package manifest, and the mandatory sequence in order:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Commit and push the implementation, then rebuild and validate the clean exact
package before detection. Attempt success requires board 205, attempt ordinal
1, exact source/reference/package/plan, one detector-admitted device, one
completed large erase, one completed exact factory restore, no unnecessary
reflash, owner Wi-Fi NVS restored with `mineonboot=false`, exact runtime
identity, trusted qualified monitor evidence, SPIFFS/static readiness, disabled
mining and hardware control, complete USB/process cleanup, protected modes,
independent validation, and redaction.

On success create `RESULT.md`, commit the closed projection as
`SOURCE_COMMIT`, transition only REL-003 to `verified` with
`workflow,api-compare,hardware-smoke,hardware-regression,release-gate`, update
its notes to cite the existing Phase 18/19 evidence, verified REL-002 result,
and new large-erase projection, synchronize progress immediately, archive the
completed active task, run final gates, review, commit, and push. Any missing
fact withholds verification, records a truthful `CLOSURE.md`, leaves REL-003
`implemented`, and stops without another erase or unchanged retry.
