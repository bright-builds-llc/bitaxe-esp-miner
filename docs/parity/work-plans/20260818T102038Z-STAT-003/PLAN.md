# Parity work plan

- Run ID: `20260818T102038Z-STAT-003`
- Parity row: `STAT-003`
- Initial status: `implemented`
- Source commit: `326a528c187d9a2eeca7fc1c5677452bdc6e2a6f`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat003-scoreboard`

## Selection

The worktree and pinned reference are clean, `main` equals `origin/main`, and
the deterministic selector reports no open plan. Candidate order begins
`SELF-001`, `BAP-002`, then `STAT-003`.

`SELF-001` remains blocked because no production-safe route exercises its
complete hardware self-test lifecycle. `BAP-002` remains blocked by not-started
`BAP-001` and a remaining external UART/electrical dependency outside standing
USB authorization. `STAT-003` is first actionable.

Attempt-002 completed its accepted campaign, 20/20 windows, scoreboard
candidates, accepted submit, identity, safety, watchdog, final consumed state,
terminal transports/pool, safe stop, and cleanup. It failed only because the
network acceptance model treated `terminal_close_requested=false` as failure
when the analyzer naturally closed serial first. The pushed correction at
`9da1d2c33b3a2c7d2a200f03b19e682f476b87e4` removes only that initiator from
acceptance truth, retains the mandatory typed diagnostic, and passes Rust plus
real-child true/false/missing/non-boolean/final-state regressions. This targeted
verified change supplies the new information required for one retry. Fresh
wrapper-003, attempt-003, and projection paths are absent; one nonempty ignored
Wi-Fi input and one nonempty ignored pool input are available without reading
their contents.

Loaded guidance materially includes `AGENTS.md`, `AGENTS.bright-builds.md`,
managed architecture/code-shape/verification/testing/Rust standards, the active
task/checklist, and bounded whole lesson blocks for safety, authorization,
privacy, diagnostic completeness, failure precedence, retry discipline,
qualified transports, transitive evaluator identity, and agent-runtime timing.
The repository lesson ledger remains above startup budget; all headings were
inventoried, less-relevant whole blocks were omitted, and the existing August
audit baseline means no new audit trigger is due.

## Scope and non-scope

Advance only `STAT-003`. Rotate the private-first scoreboard workflow from
consumed attempt-002 to fresh attempt-003: bind this immutable plan and digest,
ordinal 3, exact attempt/wrapper paths, active task, Bazel runfiles, Rust and
TypeScript contracts, generated copy, invocation tests, real-child fixtures,
31-path source identity, and independent validator. Keep all behavioral,
privacy, v12 terminal, and promotion gates unchanged except that either boolean
closure initiator is valid.

After focused/full gates pass, commit and push the rotation, build the exact
current package, and verify the manifest binds the clean pushed source and
pinned reference. Only then run one detector and, conditionally, one attempt-003
scoreboard workflow.

The workflow may run the conservative Ultra 205 profile at 400 MHz, 1,100 mV
ASIC core, 100% fan, and 600 accumulated active seconds. Its NVS seed
intentionally replaces prior settings and scoreboard records with the ignored
owner Wi-Fi/pool inputs, package defaults, and no scoreboard keys. On accepted
campaign evidence it may passively reacquire the same origin, read the
scoreboard twice, serve `/scoreboard`, issue one normal HTTP restart, and prove
identical post-restart persistence on the next software-CPU boot. The final
state retains the new scoreboard and owner inputs with mining safely stopped.

Only mode-0600 private files beneath fresh ignored mode-0700 roots may contain
runtime values. Scoreboard fields, credentials, owner/worker data, pool/network/
device endpoints, ports, USB/network identity, sensors, hashrates, HTTP bodies,
serial, commands, PIDs, and traces remain private. Committed output may contain
only schema/cryptographic identity, booleans, bounded counts, closed categories,
and admitted private-document digests.

No arbitrary or overclocked control, unbounded mining, OTA, erase, ad hoc raw
write, fault injection, physical power action, external UART/BAP, or pin/pad/
header/probe/jumper/solder/signal manipulation is authorized. The campaign
supervisor retains one exact-package recovery flash only for a post-flash
campaign failure; no later API/restart failure may flash or retry.

## Implementation

- [ ] Rotate the scoreboard plan/task binding, paths, ordinal, runfiles,
      contracts, generated copy, fixtures, and invocation tests to attempt-003
      without weakening source identity, behavior, safety, or privacy.
- [ ] Run focused and mandatory gates, commit/push the rotation, build/package
      the exact clean source, and confirm manifest/reference identity.
- [ ] Run the sole wrapper-003 detector and, only after exact admission, the
      sole attempt-003 scoreboard capture.
- [ ] Independently validate accepted evidence and promote only on the complete
      mining/API/SPA/restart durability quorum; otherwise close at the first
      typed failure without another retry.

## Authorized live commands and recovery

After the rotated implementation is clean, fully gated, committed, pushed,
packaged, and identity-checked, run only:

1. `test ! -e scratch/stat003-scoreboard/wrapper-003 && (umask 077; mkdir -m 700 -p scratch/stat003-scoreboard/wrapper-003 && just detect-ultra205 > scratch/stat003-scoreboard/wrapper-003/detector.stdout 2> scratch/stat003-scoreboard/wrapper-003/detector.stderr)`
2. After command 1 exits zero, admits exactly one Ultra 205, protected inputs
   remain nonempty without being read, and child/projection remain absent:
   `test ! -e scratch/stat003-scoreboard/attempt-003 && test ! -e docs/parity/evidence/stat003-scoreboard/scoreboard-projection.json && test -s wifi-credentials.json && test -s pool-credentials.json && (umask 077; just capture-scoreboard-evidence --private-root scratch/stat003-scoreboard/attempt-003 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --pool-credentials pool-credentials.json --detector-output scratch/stat003-scoreboard/wrapper-003/detector.stdout --projection docs/parity/evidence/stat003-scoreboard/scoreboard-projection.json --duration-seconds 600 --capture-timeout-seconds 1800 > scratch/stat003-scoreboard/wrapper-003/capture.stdout 2> scratch/stat003-scoreboard/wrapper-003/capture.stderr)`

The caller owns only wrapper-003 streams; the verifier exclusively creates the
absent attempt child. Starting command 2 consumes attempt-003. Never rerun it or
create attempt-004 under this plan. Preserve the earliest typed failure through
recovery, passive reacquisition, restart, sealing, validation, and cleanup. Any
nonzero command, identity/safety/privacy/cleanup/recovery failure, or missing
fact withholds the projection and ends this plan.

If network v12 again records `accepted_after_serial_close`, final consumed,
serial finished, complete terminal transports/pool, and status failed because
`terminal_close_requested=false`, the same boundary recurred after its targeted
fix: stop without another retry or correction under this plan.

## Verification and promotion

Focused verification covers exact plan/task/attempt admission, contract
agreement, invocation shape, runfiles, real fresh child processes, 31-path
source/reference inventory, both closure initiators, invalid diagnostic shapes,
protected modes, earliest failure, projection withholding, and redaction. Build
the real firmware/package and run reference/redaction checks before device use.

Mandatory ordered gates are `cargo fmt --all`, `cargo clippy --all-targets
--all-features -- -D warnings`, `cargo build --all-targets --all-features`,
`cargo test --all-features`, `bun scripts/bright-builds-check.ts all`, `just
test`, `just parity`, and `just parity-progress`.

Promotion requires exact board/attempt/source/reference/package/plan/detector;
fresh NVS without scoreboard keys; accepted conservative 600-second campaign;
qualified nonce plus accepted-or-rejected submit evidence; 20/20 renewed
windows; stable watchdog; no panic/mixed session; accepted v12 terminal
settlement with final consumed/serial finish and typed closure initiator; safe
stop and cleanup; stable nonempty 1-20 exact-shape scoreboard with finite
positive difficulty, bounded fields, uppercase fixed-width hex, and descending
order; live `/scoreboard`; one exact next-ordinal software restart with new
session; identical post-restart scoreboard; disabled boot mining; protected
modes; current source semantics; independent validation; and redaction.

On success create `RESULT.md`, commit evidence against the implementation
source without changing the checklist, transition only `STAT-003` to `verified`
with `unit,workflow,api-compare,static-route,hardware-smoke,hardware-regression`,
sync progress, archive the completed task, run every final gate, and push. On
failure create `CLOSURE.md`, leave `STAT-003` implemented, preserve the closed
blocker, and do not synchronize unchanged progress.

## Non-claims

No pre-claim of live scoreboard behavior or promotion. This plan does not
verify every hardware eviction ordering, wall-clock presentation, absolute
nonce-difficulty calibration, profitability, arbitrary profiles/pools, other
ASICs/boards, unbounded mining, update recovery, or release readiness.
