# Parity work plan

- Run ID: `20260812T114949Z-STR-006`
- Parity row: `STR-006`
- Initial status: `implemented`
- Source commit: `4d745dc4dfd8c9a0f5aa2e4e80872e17e0559667`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-str006-protocol-coordinator-promotion`

## Selection

The canonical selector returned no open plan and selected `STR-006` first,
followed by `STR-007` and the remaining unfinished rows. The clean `main`
worktree was exactly synchronized with `origin/main` after fetch, and the
read-only reference tree was clean at the commit above.

The accepted conservative Ultra 205 campaign at source commit
`3e0966a140edbff1a14d2a48ca63d140649762c0` already supplies every live effect
needed by this row. Four independently validated public projections share that
exact attempt and reference lineage:

- ASIC initialization SHA-256
  `eee750561a7c1dcec1a5698b1e5827d3f1508d43655c3c4aa237097338dcf8d4`
- ASIC work-send SHA-256
  `447af65ae9e6cd5cc2199ef639ff8e0fa7f63d4c9708570bd66781c5a162e80c`
- ASIC result-parsing SHA-256
  `e99c054c4d660155d5c2b1ee38d3f17aed5ae7101e7e4a5fd1c6451d1b48b7c7`
- Stratum socket SHA-256
  `dcb3eed396a268114b017d7ef4fbca9c427a390d7acf405fc52fbef6472122b8`

Together they prove exact package admission, trusted runtime identity, fresh
safety, complete hardware preparation, a retained production UART, initialized
live work transmission, a qualified correlated result, an authorized live
Stratum session, a real accepted submit response, confirmed safe stop, lease
cleanup, and USB cleanup. The complete coordinator, recovery, owner, and ASIC
worker source set has not changed since the initialization projection source
commit `f9df1412abbc05a4852022f3fb6741f67ab43272`.

## Scope and non-scope

This run will derive one redacted `bitaxe-protocol-coordinator-evidence-v1`
projection from the four committed projections and exact Git source history.
It must bind all prerequisite digests and independent validators; shared
attempt/current/reference lineage; unchanged coordinator source; the single
owner and bounded inbox; the 1,000-ms readiness reread cadence; all six
fail-closed readiness gates; hardware preparation before pool access;
authorization before ASIC dispatch; a qualified correlated ASIC result before
submit; the observed accepted response; ordered terminal safe stop; watchdog
feeding; cleanup; independent validation; atomic publication; and redaction.

No protected campaign input will be reopened. No detector, package rebuild,
flash, reset, USB session, credential read, serial or network request, mining
lease, pool contact, fan/voltage/power/ASIC actuation, recovery action, direct
UART, pin manipulation, or other hardware effect is permitted. The public
projection must not contain raw protocol lines, responses, work, nonces,
targets, difficulty, pool or Wi-Fi values, endpoints, ports, users, workers,
owner addresses, credentials, USB/network identifiers, device paths, local
paths, secrets, or secret-derived hashes.

This row does not claim fallback or reconnect hardware behavior, long-running
coordination, watchdog timing under sustained load, arbitrary pools, automatic
fan control, accepted-share profitability, upstream-default mining, unbounded
mining, TLS, Stratum v2, other boards, updates, recovery, or release readiness.

## Implementation

- [ ] Add a Rust-owned closed evidence contract and independent validator for
      the exact accepted protocol-coordinator projection.
- [ ] Add a thin host projector that independently validates all four source
      projections and digests, their shared lineage, accepted-source ancestry,
      unchanged coordinator modules, compatible unique lifecycle/owner spans,
      and clean relevant paths.
- [ ] Derive the bounded owner/readiness facts and ordered coordinator lifecycle
      from admitted current source while binding live effects only to the
      accepted public projections.
- [ ] Add behavior-focused regressions for malformed or incomplete source
      evidence, digest/lineage/module/span/dirty-path drift, invalid coordinator
      facts, validator failure, sensitive output, and a real child-process/file
      seam.
- [ ] Publish evidence only after the independent validator accepts the atomic
      candidate; any failure must remove the candidate and withhold evidence.

## Verification and promotion

Focused verification will run the new Rust contract tests, host projector
tests, real-child integration, production-session lifecycle/recovery tests,
owner/ASIC adapter tests, and direct validation of
`docs/parity/evidence/str006-protocol-coordinator/protocol-coordinator-projection.json`.
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
immutable-plan digest, all prerequisite validators and digest bindings, public
sensitive-value scan, source compatibility, and `git diff --check`.

Promote only `STR-006` from `implemented` to `verified` with
`unit,workflow,hardware-smoke,hardware-regression` if the closed proof shows
the accepted conservative Ultra 205 traversed the unchanged single-owner
protocol coordinator from complete readiness through hardware preparation,
authorized pool operation, initialized work dispatch, qualified result,
accepted submit response, ordered fail-closed safe stop, and cleanup while the
relevant current source remains compatible. Any malformed, incomplete,
digest- or lineage-mismatched, source-drifted, dirty, validator-rejected, or
sensitive input withholds evidence and leaves the row `implemented`; there is
no hardware retry or effect path in this plan.
