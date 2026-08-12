# Parity work plan

- Run ID: `20260812T111705Z-STR-001`
- Parity row: `STR-001`
- Initial status: `implemented`
- Source commit: `c69c7922379b643ab00e8f226808ce6cc39d928a`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-str001-socket-promotion`

## Selection

The canonical selector returned no open plan and selected `STR-001` first,
followed by `STR-006`, `STR-007`, and the remaining unfinished rows. The clean
`main` worktree was exactly synchronized with `origin/main` after fetch, and
the read-only reference tree was clean at the commit above.

The accepted conservative Ultra 205 campaign at source commit
`3e0966a140edbff1a14d2a48ca63d140649762c0` already supplies the bounded live
network effect required by this row. Its independently validated ASIC-002
projection at SHA-256
`eee750561a7c1dcec1a5698b1e5827d3f1508d43655c3c4aa237097338dcf8d4`
proves an exact admitted package, trusted runtime, completed production
initialization, live initialized work, a real accepted submit response, fresh
safety, confirmed safe stop, lease cleanup, and USB cleanup. The production
TCP transport module is byte-for-byte unchanged between that accepted source
and the current source. The current owner shell still maps typed connected,
bytes, failure, and closed events and routes typed connect, write, and close
effects through that transport; the pure lifecycle accepts a submit response
only for a pending submit on an authorized active pool session.

## Scope and non-scope

This run will derive one redacted `bitaxe-stratum-socket-evidence-v1`
projection from the committed, independently validated ASIC-002 initialization
projection and exact Git source history. It must bind the prerequisite digest,
accepted/current/reference commits, the unchanged production TCP adapter,
bounded command queue and connect/read/write settings, TCP no-delay, typed
connect/write/read/closed handling, transport-epoch isolation, the authorized
live session required before submit, the observed accepted response, current
source compatibility, safe stop, cleanup, independent validation, atomic
publication, and redaction.

No protected campaign input will be reopened. No detector, package rebuild,
flash, reset, USB session, credential read, serial or network request, mining
lease, pool contact, fan/voltage/power/ASIC actuation, recovery action, direct
UART, pin manipulation, or other hardware effect is permitted. The public
projection must not contain raw socket bytes, protocol lines, responses,
nonces, targets, difficulty, pool or Wi-Fi values, endpoints, ports, users,
workers, owner addresses, credentials, USB/network identifiers, device paths,
local paths, secrets, or secret-derived hashes.

This row does not claim fallback or reconnect hardware behavior, DNS or IP-
family preference parity, upstream timeout/keepalive values, arbitrary pool
compatibility, TLS, Stratum v2, unbounded socket stability, accepted-share
profitability, other boards, updates, recovery, or release readiness.

## Implementation

- [ ] Add a Rust-owned closed evidence contract and independent validator for
      the exact accepted Stratum v1 socket projection.
- [ ] Add a thin host projector that validates the committed ASIC-002 source
      projection and digest, accepted source ancestry, unchanged TCP adapter,
      compatible owner/lifecycle semantics, and clean relevant paths.
- [ ] Derive bounded current transport constants and fail-closed categories
      from the admitted source while binding live success only to the accepted
      initialization projection.
- [ ] Add behavior-focused regressions for malformed or incomplete source
      evidence, digest/commit/module/semantic/dirty-path drift, invalid socket
      facts, validator failure, sensitive output, and a real child-process/file
      seam.
- [ ] Publish evidence only after the independent validator accepts the atomic
      candidate; any failure must remove the candidate and withhold evidence.

## Verification and promotion

Focused verification will run the new Rust contract tests, host projector
tests, real-child integration, production transport loopback tests, production
session lifecycle/recovery tests, and direct validation of
`docs/parity/evidence/str001-socket/stratum-socket-projection.json`. The
mandatory ordered repository gate is:

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
immutable-plan digest, prerequisite validator and digest binding, public-
sensitive-value scan, source compatibility, and `git diff --check`.

Promote only `STR-001` from `implemented` to `verified` with
`unit,workflow,hardware-smoke,hardware-regression` if the closed proof shows
the accepted conservative Ultra 205 session traversed the unchanged bounded
production TCP adapter through an authorized Stratum v1 lifecycle to a real
accepted submit response, then achieved confirmed safe stop and cleanup while
the relevant current source remains compatible. Any malformed, incomplete,
digest-mismatched, source-drifted, dirty, validator-rejected, or sensitive
input withholds evidence and leaves the row `implemented`; there is no hardware
retry or effect path in this plan.
