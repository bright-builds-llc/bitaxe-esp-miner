# Parity work plan

- Run ID: `20260812T104903Z-ASIC-007`
- Parity row: `ASIC-007`
- Initial status: `implemented`
- Source commit: `d7be191c9da12f63e38cbd75092912f7903df39a`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-asic007-frequency-transition-promotion`

## Selection

The canonical selector returned no open plan and listed `ASIC-007` first,
followed by `STR-001`, `STR-006`, and the remaining unfinished rows. The clean
`main` worktree was exactly synchronized with `origin/main` after fetch, and
the read-only reference tree was clean at the commit above.

The accepted Ultra 205 conservative campaign at source commit
`3e0966a140edbff1a14d2a48ca63d140649762c0` already supplies the bounded live
effect required by this row. Its independently validated ASIC-002 projection
at SHA-256 `eee750561a7c1dcec1a5698b1e5827d3f1508d43655c3c4aa237097338dcf8d4`
proves all nine preparation steps completed, exactly one BM1366 initialized,
live initialized work followed, an accepted response was observed, and safe
stop and cleanup completed. At that accepted source, the preparation boundary
selects `production_with_frequency_ramp`; the pure command plan produces the
upstream-aligned 50-to-400-MHz conservative ramp, and the production executor
returns success only after every typed UART action completes. All complete
frequency-plan, actuation, UART, and orchestration modules are unchanged; the
unique production action-loop span is also unchanged despite unrelated later
edits elsewhere in its module.

## Scope and non-scope

This run will derive one redacted
`bitaxe-asic-frequency-transition-evidence-v1` projection from the committed,
independently validated ASIC-002 initialization projection and exact Git source
history. It must bind the prerequisite digest, accepted/current/reference
commits, conservative hardware profile, upstream-aligned 6.25-MHz steps,
100-ms inter-step delays, 50-MHz start, 400-MHz terminal target, closed command
and delay counts, complete action execution, subsequent live initialized work,
accepted submit response, safe stop, cleanup, source compatibility, independent
validation, atomic publication, and redaction.

No protected campaign input will be reopened. No detector, package rebuild,
flash, reset, USB session, credential read, serial or network request, mining
lease, fan/voltage/power/ASIC actuation, recovery action, direct UART, pin
manipulation, or other hardware effect is permitted. The public projection
must not contain raw frames or traces, PLL register bytes, nonces, targets,
difficulty, pool or Wi-Fi values, endpoints, ports, users, workers, owner
addresses, credentials, USB/network identifiers, device paths, local paths,
secrets, or secret-derived hashes.

This row does not claim arbitrary frequency targets, dynamic runtime retuning,
default-profile or overclock behavior, direct external UART use, voltage/fan/
power/thermal parity, other ASICs or boards, soak stability, updates, recovery,
profitability, or release readiness.

## Implementation

- [ ] Add a Rust-owned closed evidence contract and independent validator for
      the exact BM1366 conservative frequency-transition projection.
- [ ] Add a thin host projector that validates the committed ASIC-002 source
      projection and digest, accepted source ancestry, unchanged full modules,
      unique compatible executor spans, and clean relevant paths.
- [ ] Derive ramp counts and boundaries from the current pure command planner,
      while binding live completion to the accepted initialization projection.
- [ ] Add behavior-focused regressions for malformed or incomplete source
      evidence, digest/commit/module/span/dirty-path drift, invalid ramp facts,
      validator failure, sensitive output, and a real child-process/file seam.
- [ ] Publish evidence only after the independent validator accepts the atomic
      candidate; any failure must remove the candidate and withhold evidence.

## Verification and promotion

Focused verification will run the new Rust contract tests, host projector
tests, real-child integration, frequency-ramp golden tests, mining actuation
tests, and direct validation of
`docs/parity/evidence/asic007-frequency-transition/asic-frequency-transition-projection.json`.
The mandatory ordered repository gate is:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Also require generated automation contracts, `just verify-redaction`,
`just verify-reference`, exact reference cleanliness, task uniqueness,
immutable-plan digest, the prerequisite validator and digest binding,
public-sensitive-value scan, source compatibility, and `git diff --check`.

Promote only `ASIC-007` from `implemented` to `verified` with
`unit,golden,workflow,hardware-smoke,hardware-regression` if the closed proof
shows the accepted conservative Ultra 205 session completed every typed command
in the bounded 50-to-400-MHz frequency ramp before live initialized work and an
accepted response, then achieved confirmed safe stop and cleanup, while the
relevant current source remains compatible. Any malformed, incomplete,
digest-mismatched, source-drifted, dirty, validator-rejected, or sensitive input
withholds evidence and leaves the row `implemented`; there is no hardware retry
or effect path in this plan.
