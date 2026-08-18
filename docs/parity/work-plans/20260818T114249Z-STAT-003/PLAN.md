# Parity work plan

- Run ID: `20260818T114249Z-STAT-003`
- Parity row: `STAT-003`
- Initial status: `implemented`
- Source commit: `e8d49d92ae9eccd9c7cdd32fed557ff78e76d3a7`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat003-scoreboard`

## Selection

The worktree and pinned reference are clean, `main` equals `origin/main`, and
the selector reports no open plan. Candidate order begins `SELF-001`, `BAP-002`,
then `STAT-003`.

`SELF-001` remains blocked without a production-safe complete hardware self-
test route. `BAP-002` remains blocked by not-started `BAP-001` plus an external
UART/electrical dependency outside standing USB authorization. `STAT-003` is
first actionable.

Attempt-003 proved accepted campaign/network evidence, 20/20 windows, real
scoreboard candidates, accepted submit, identity, safety, watchdog, natural
terminal closure, safe stop, cleanup, a stable 20-entry scoreboard, live SPA,
and an exact session-changing ordinal-plus-one software-CPU restart with false
boot intent. It stopped because the verifier rejected the closed non-active
state `paused`. The pushed correction at
`251205a57b42a1f0a1e4a59a0d90dd5b5837f5af` centralizes disabled boot mining as
false intent plus `paused` or `safe_blocked`, rejects active/unknown/enabled
shapes, and passes pure plus full real-child persistence/projection tests. This
targeted verified change supplies the new information required for one retry.
Fresh wrapper-004, attempt-004, and projection paths are absent; one nonempty
ignored Wi-Fi input and one nonempty ignored pool input are available without
reading their contents.

Loaded guidance materially includes `AGENTS.md`, `AGENTS.bright-builds.md`,
managed architecture/code-shape/verification/testing/Rust standards, the active
task/checklist, and bounded whole lesson blocks for safety, authorization,
privacy, diagnostic completeness, failure precedence, retry discipline,
qualified transports, transitive evaluator identity, and agent-runtime timing.
The repository lesson ledger remains above startup budget; all headings were
inventoried, less-relevant whole blocks were omitted, and the existing August
audit baseline means no new audit trigger is due.

## Scope and non-scope

Advance only `STAT-003`. Rotate the scoreboard workflow from consumed
attempt-003 to fresh attempt-004: bind this plan/digest, ordinal 4, exact paths,
active task, runfiles, Rust/TypeScript/generated contracts, invocation tests,
fixtures, 31-path source identity, and independent validator. Keep all behavior,
privacy, terminal, stopped-state, and promotion gates unchanged.

After focused/full gates, commit and push the rotation, build the exact current
package, and prove clean source/reference manifest identity. Then run one
detector and, conditionally, one attempt-004 workflow.

The workflow may run the conservative Ultra 205 profile at 400 MHz, 1,100 mV
ASIC core, 100% fan, and 600 accumulated active seconds. Its NVS seed
intentionally replaces earlier settings/scoreboard with ignored owner Wi-Fi/
pool inputs, package defaults, and no scoreboard keys. On campaign success it
may passively reacquire the same origin, read the scoreboard twice, serve
`/scoreboard`, issue one normal HTTP restart, and prove identical post-restart
persistence on the exact next software-CPU boot. The final state retains the
new scoreboard and owner inputs with mining stopped.

Only mode-0600 private files below fresh ignored mode-0700 roots may contain
runtime values. Scoreboard fields, credentials, owner/worker data, endpoints,
ports, USB/network identity, sensors, hashrates, HTTP, serial, commands, PIDs,
and traces remain private. Committed output may contain only schema/
cryptographic identity, booleans, bounded counts, closed categories, and
admitted private-document digests.

No arbitrary/overclocked control, unbounded mining, OTA, erase, ad hoc raw
write, fault injection, physical power action, external UART/BAP, or pin/pad/
header/probe/jumper/solder/signal manipulation is authorized. The supervisor
retains one exact-package recovery flash only for a post-flash campaign failure;
no later API/restart failure may flash or retry.

## Implementation

- [ ] Rotate plan/task binding, paths, ordinal, runfiles, contracts, generated
      copy, fixtures, and invocation tests to attempt-004 without weakening any
      source, behavior, safety, stopped-state, or privacy gate.
- [ ] Run focused/full gates, commit/push, build/package the exact clean source,
      and confirm manifest/reference identity.
- [ ] Run the sole wrapper-004 detector and, only after exact admission, the
      sole attempt-004 scoreboard capture.
- [ ] Independently validate and promote only on the complete campaign/API/SPA/
      restart persistence quorum; otherwise close at the first typed failure
      without another retry.

## Authorized live commands and recovery

After clean pushed gates and package identity, run only:

1. `test ! -e scratch/stat003-scoreboard/wrapper-004 && (umask 077; mkdir -m 700 -p scratch/stat003-scoreboard/wrapper-004 && just detect-ultra205 > scratch/stat003-scoreboard/wrapper-004/detector.stdout 2> scratch/stat003-scoreboard/wrapper-004/detector.stderr)`
2. After exact detector admission, nonempty protected inputs, and absent child/
   projection:
   `test ! -e scratch/stat003-scoreboard/attempt-004 && test ! -e docs/parity/evidence/stat003-scoreboard/scoreboard-projection.json && test -s wifi-credentials.json && test -s pool-credentials.json && (umask 077; just capture-scoreboard-evidence --private-root scratch/stat003-scoreboard/attempt-004 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --pool-credentials pool-credentials.json --detector-output scratch/stat003-scoreboard/wrapper-004/detector.stdout --projection docs/parity/evidence/stat003-scoreboard/scoreboard-projection.json --duration-seconds 600 --capture-timeout-seconds 1800 > scratch/stat003-scoreboard/wrapper-004/capture.stdout 2> scratch/stat003-scoreboard/wrapper-004/capture.stderr)`

The caller owns only wrapper-004 streams; the verifier exclusively creates the
attempt child. Starting command 2 consumes attempt-004. Never rerun it or create
attempt-005 under this plan. Preserve earliest failure through recovery,
reacquisition, restart, sealing, validation, and cleanup. Any nonzero command or
missing identity/safety/privacy/cleanup/recovery fact withholds projection and
ends this plan.

If the exact restart again returns session change, ordinal +1, `software_cpu`,
false boot intent, and `paused` but blocks before post-restart scoreboard reads,
the same boundary recurred after its fix; stop further retries.

## Verification and promotion

Focused verification covers plan/task/ordinal/path admission, generated
contracts, runfiles, invocation, real child processes, 31-path inventory, both
stopped states, rejected unsafe shapes, protected modes, first failure, and
redaction. Build firmware/package and run reference/redaction before device use.

Mandatory ordered gates are `cargo fmt --all`, `cargo clippy --all-targets
--all-features -- -D warnings`, `cargo build --all-targets --all-features`,
`cargo test --all-features`, `bun scripts/bright-builds-check.ts all`, `just
test`, `just parity`, and `just parity-progress`.

Agent-local pre-plan note: exact `just test` produced 46/47 green targets but
hit the agent sandbox's 300-second automation timeout twice with no assertion
failure. The identical `bazel test //... --test_timeout=900` graph then passed
all 47 targets in 72.4 seconds, and the same source had already passed exact
`just test` before this plan-only documentation change. This is a disclosed
agent-runtime exception, not a changed repository timeout or host-failure claim;
finalization must still attempt the exact gate again.

Promotion requires exact board/attempt/source/reference/package/plan/detector;
fresh NVS without scoreboard keys; accepted conservative 600-second campaign;
qualified nonce plus accepted-or-rejected submit evidence; 20/20 windows;
stable watchdog; no panic/mixed session; accepted v12 terminal settlement;
safe stop/cleanup; stable nonempty 1-20 exact-shape scoreboard with finite
positive difficulty, bounded fields, uppercase hex, and descending order; live
SPA; one exact next-ordinal software restart with new session; disabled boot
mining; identical post-restart scoreboard/repeat; protected modes; current
source semantics; independent validation; and redaction.

On success create `RESULT.md`, commit evidence against the implementation
source without checklist change, transition only `STAT-003` to `verified` with
`unit,workflow,api-compare,static-route,hardware-smoke,hardware-regression`, sync
progress, archive the complete task, run every final gate, and push. On failure
create `CLOSURE.md`, leave `STAT-003` implemented, preserve the blocker, and do
not sync unchanged progress.

## Non-claims

No pre-claim of live scoreboard parity. This plan does not verify every hardware
eviction order, wall-clock presentation, absolute nonce-difficulty calibration,
profitability, arbitrary profiles/pools, other ASICs/boards, unbounded mining,
update recovery, or release readiness.
