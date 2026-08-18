# Parity work plan

- Run ID: `20260818T090846Z-STAT-003`
- Parity row: `STAT-003`
- Initial status: `implemented`
- Source commit: `7bd75d600be4e7920010cc3f119a9c6015d23939`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat003-scoreboard`

## Selection

The worktree and pinned reference are clean, `main` equals `origin/main`, and
the deterministic selector reports no open plan. Candidate order begins
`SELF-001`, `BAP-002`, then `STAT-003`.

`SELF-001` remains dependency- and safety-blocked because the repository has no
production-safe route for its fan, voltage, power, ASIC, pass/fail/cancel,
reboot, and recovery effects. `BAP-002` remains blocked by not-started
`BAP-001`; the missing live accessory path also requires direct external UART
and electrical attachment, which standing USB authorization does not permit.
`STAT-003` is first actionable.

Attempt-001 completed the conservative campaign, 20/20 renewed windows, real
scoreboard candidates, an accepted submit, trusted identity, fresh safety,
stable watchdog, terminal transport/pool joins, confirmed safe stop, and USB
cleanup, but the network worker finalized from a pre-handoff snapshot. The
targeted pushed correction at `ca42d7de79ee250161904f1ae14f1bc2ff833324`
now requests capture closure and waits for the analyzer's final consumed state.
That verified change supplies the new information required before a hardware
retry. Fresh wrapper-002, attempt-002, and projection paths are absent; exactly
one nonempty ignored Wi-Fi input and one nonempty ignored pool input are
available without reading their contents.

Loaded guidance materially includes `AGENTS.md`, `AGENTS.bright-builds.md`,
`standards-overrides.md`, managed architecture/code-shape/verification/testing/
Rust standards, the active task/checklist, and bounded whole lesson blocks for
safety, authorization, privacy, diagnostic completeness, first-failure
precedence, retry discipline, qualified transports, and evaluator identity.
The repository lesson ledger remains above the startup budget; all headings
were inventoried, less-relevant complete blocks were omitted, and the existing
August audit baseline means no new audit trigger is due.

## Scope and non-scope

Advance only `STAT-003`. First rotate the scoreboard evidence workflow from
consumed attempt-001 to fresh attempt-002: bind this immutable plan and SHA-256,
the exact attempt/wrapper paths, ordinal 2, active task record, Bazel runfiles,
Rust contract, TypeScript admission, invocation tests, real-child fixtures, and
independent validator. Keep the v12 terminal-settlement requirements and every
existing scoreboard, source-identity, privacy, safety, recovery, and promotion
gate unchanged.

After focused/full software gates pass, commit and push the rotation, build the
exact current package, and verify its manifest binds the clean pushed source
and pinned reference. Only then run one detector and, conditionally, one
attempt-002 scoreboard workflow.

The workflow may run the conservative Ultra 205 profile at 400 MHz, 1,100 mV
ASIC core, 100% fan, and 600 accumulated active seconds. Its normal NVS seed
intentionally replaces prior settings and scoreboard records with the ignored
owner Wi-Fi/pool inputs, package defaults, and no scoreboard keys. On campaign
success it may passively reacquire the same origin, read the scoreboard twice,
serve `/scoreboard`, issue one normal HTTP restart, and prove identical
post-restart persistence on the next software-CPU boot. The final state retains
the new scoreboard and owner inputs with mining safely stopped.

Only mode-0600 private files beneath fresh ignored mode-0700 roots may contain
runtime values. Scoreboard entries and fields, credentials, owner/worker data,
pool/network/device endpoints, ports, USB/network identity, sensors, hashrates,
HTTP bodies, serial, commands, PIDs, and traces remain private. Committed output
may contain only schema/cryptographic identity, booleans, bounded counts,
closed categories, and admitted private-document digests.

No arbitrary or overclocked control, unbounded mining, OTA, erase, ad hoc raw
write, fault injection, physical power action, external UART/BAP, or pin/pad/
header/probe/jumper/solder/signal manipulation is authorized. The campaign
supervisor retains its one exact-package recovery flash only for a post-flash
campaign failure; no later API/restart failure may flash or retry.

## Implementation

- [ ] Rotate the scoreboard evidence contract, plan/task binding, paths,
      ordinal, runfiles, fixtures, and invocation tests to attempt-002 without
      changing its behavioral or privacy quorum.
- [ ] Run focused and mandatory gates, commit/push the rotation, build/package
      the exact clean source, and confirm manifest/reference identity.
- [ ] Run the sole wrapper-002 detector and, only after exact admission, the
      sole attempt-002 scoreboard capture.
- [ ] Independently validate accepted evidence and promote only on the complete
      mining/API/UI/restart durability quorum; otherwise close at the first
      typed failure without another retry.

## Authorized live commands and recovery

After the rotated implementation is clean, fully gated, committed, pushed,
packaged, and identity-checked, run only:

1. `test ! -e scratch/stat003-scoreboard/wrapper-002 && (umask 077; mkdir -m 700 -p scratch/stat003-scoreboard/wrapper-002 && just detect-ultra205 > scratch/stat003-scoreboard/wrapper-002/detector.stdout 2> scratch/stat003-scoreboard/wrapper-002/detector.stderr)`
2. After command 1 exits zero, admits exactly one Ultra 205, protected inputs
   remain nonempty without being read, and child/projection remain absent:
   `test ! -e scratch/stat003-scoreboard/attempt-002 && test ! -e docs/parity/evidence/stat003-scoreboard/scoreboard-projection.json && test -s wifi-credentials.json && test -s pool-credentials.json && (umask 077; just capture-scoreboard-evidence --private-root scratch/stat003-scoreboard/attempt-002 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --pool-credentials pool-credentials.json --detector-output scratch/stat003-scoreboard/wrapper-002/detector.stdout --projection docs/parity/evidence/stat003-scoreboard/scoreboard-projection.json --duration-seconds 600 --capture-timeout-seconds 1800 > scratch/stat003-scoreboard/wrapper-002/capture.stdout 2> scratch/stat003-scoreboard/wrapper-002/capture.stderr)`

The caller owns only wrapper-002 streams; the verifier exclusively creates the
absent attempt child. Starting command 2 consumes attempt-002. Never rerun it or
create attempt-003 under this plan. Preserve the earliest typed failure through
recovery, passive reacquisition, restart, sealing, validation, and cleanup.
Any nonzero command, identity/safety/privacy/cleanup/recovery failure, or
missing fact withholds the projection and ends this plan.

## Verification and promotion

Focused verification covers exact plan/task/attempt admission, Rust and
TypeScript contract agreement, invocation shape, runfiles, real fresh child
processes, source/reference inventory, protected modes, v12 terminal settlement,
earliest failure, projection withholding, and redaction. Build the real
firmware/package and run reference and redaction checks before device access.

Mandatory ordered gates are `cargo fmt --all`, `cargo clippy --all-targets
--all-features -- -D warnings`, `cargo build --all-targets --all-features`,
`cargo test --all-features`, `bun scripts/bright-builds-check.ts all`, `just
test`, `just parity`, and `just parity-progress`.

Promotion requires exact board/attempt/source/reference/package/plan/detector;
fresh NVS without scoreboard keys; accepted conservative 600-second campaign;
qualified nonce plus accepted-or-rejected submit evidence; 20/20 renewed
windows; stable watchdog; no panic or mixed session; v12 terminal settlement
accepted after serial close with final consumed handoff; safe stop and cleanup;
a stable nonempty 1-20 entry scoreboard with exact wire shape, finite positive
difficulty, bounded fields, uppercase fixed-width hex, and descending order;
live `/scoreboard`; one exact next-ordinal software restart with new session;
identical post-restart scoreboard; disabled boot mining; protected modes;
current source semantics; independent validation; and redaction.

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
