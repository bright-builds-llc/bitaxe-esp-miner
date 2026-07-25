---
quick_id: 260712-0a9
verified: "2026-07-12T06:12:59Z"
status: passed
score: "7/7 must-haves verified"
generated_by: gsd-verifier
source_commit: a38bb0f37f96ad1b4ed6fc72cb7771617e275a3c
hardware_used: false
reverification: true
---

# Quick Task 260712-0a9 Verification

## Conclusion

The heartbeat implementation, Plan 13 validation behavior, and Phase 13 monitor
completion boundary are verified in software. The previously reproducible fast
process race is repaired, and the required monitor target passes from a fresh
uncached Bazel execution. An independent fresh uncached run of all six affected
Bazel targets also passes at exact HEAD `a38bb0f37f96`. The quick task is ready
for the root workflow's hardware exact-HEAD boundary.

Material guidance applied: the repo-local serial-session and hardware boundary
rules in `AGENTS.md`, `AGENTS.bright-builds.md`,
`standards/core/architecture.md`, `standards/core/code-shape.md`,
`standards/core/testing.md`, `standards/core/verification.md`, and
`standards/languages/rust.md`. No hardware, credentials, network discovery,
flash, monitor, reset, push, or destructive command was used.

## Must-Have Verification

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Normal firmware starts one boot-lifetime observer after logger initialization and emits the exact serial-only heartbeat from the shared boot session without service dependencies. | VERIFIED | `main.rs` calls `initialize_observer()` immediately after logger initialization; `boot_evidence.rs` owns the sole `runtime-observer`, shared `BOOT_SESSION`, and unconditional heartbeat loop. The static ownership test passed directly and through Bazel. |
| 2 | A host-tested pure heartbeat model owns rendering, sequence, cadence, deadline, and listener latch while the ESP adapter remains effect-only. | VERIFIED | `RuntimeHeartbeatModel` and `RuntimeHeartbeatSample` live in host-compatible `bitaxe-api`; seven focused tests passed. The firmware adapter only reads canonical uptime, synchronizes the model, sleeps, replays allowlisted evidence, and calls `log::info!`. |
| 3 | Sequence and uptime advance without backfill; the 120000 ms boundary renders 1000 ms cadence and schedules 130000 ms, then transitions to 10000 ms. | VERIFIED | Focused tests cover first due time, 119999/120000/120001 cadence, the 130000 deadline, delayed-wakeup coalescing, sequence increments, and one-way listener latching. |
| 4 | Heartbeats are serial-only and retain an independent 10000 ms / 1880000 ms Plan 13 replay schedule. | VERIFIED | `emit_due_heartbeat()` calls only `log::info!`; no heartbeat append exists in `log_buffer`, HTTP, or WebSocket paths. Replay constants remain `10000` and `1880000` and are scheduled independently in the same observer. |
| 5 | Reinit stays strict while retained cold start may combine same-session dedicated and stateful heartbeat proof. | VERIFIED | The lifecycle comparator requires original reinit boot/listener markers plus dedicated states and a heartbeat. Cold start accepts heartbeat boot proof and only `listener_armed=true` as listener fallback, while unifying dedicated and heartbeat sessions. Focused comparator fixtures passed. |
| 6 | Heartbeat faults map deterministically to closed member-specific codes and redacted summary fields. | VERIFIED | Typed comparator errors implement malformed, session-conflict, monotonicity, cadence, absence, and listener-proof precedence; the shell adapter parses one exact code record into the closed state vocabulary. Comparator, state, adapter, and 84 invalid-case exact-head suites passed. |
| 7 | All focused software gates pass before preparing the hardware exact HEAD. | VERIFIED | The direct monitor suite and fresh uncached Bazel monitor target pass, followed by the complete six-target Bazel gate, focused suites, firmware build, reference/redaction guards, and mandatory Rust sequence. |

## Closed Verification Gap

The failure came from polling transient `ps` state for an unreaped, fast
process-group leader. Under the race, the wrapper could reach its timeout path
even though the fake monitor had already exited successfully.

The monitor wrapper now writes its exit status through a private mode-0600
atomic completion handoff. The timeout boundary rechecks that durable state
before signaling the group. Successful completion still waits and reaps the
leader, then checks and cleans the entire process group, so orphaned descendants
and truly live timeouts retain their previous fail-closed cleanup semantics.

Regression coverage runs ten immediate exits and requires status 0/completed
every time. Existing slow timeout, timeout-descendant, TERM, and orphan cleanup
fixtures continue to pass.

## Verification Commands

Passed:

- `cargo test -p bitaxe-api --all-features runtime_heartbeat` — 7 passed.
- Observer ownership, lifecycle comparator, state authority, diagnostic adapter,
  and exact-head suites — passed; exact-head covered 84 invalid cases.
- `bash scripts/phase13-monitor-capture-test.sh`.
- `bazel test --nocache_test_results //scripts:phase13_monitor_capture_test` —
  freshly executed and passed.
- Complete six-target Bazel gate with `--nocache_test_results` — all six targets
  freshly executed and passed; the exact-head target exercised 84 invalid cases.
- `shellcheck` and `shfmt -d` for touched shell scripts.
- `cargo fmt --all -- --check`, Clippy with warnings denied, all-target/all-feature
  build, and all-feature tests.
- `bazel build //firmware/bitaxe:firmware`, `just verify-reference`, and
  `git diff --check`.

## Remaining Boundary

No software verification gap remains. The root workflow owns any later commit
push and the separately gated real-hardware Plan 13 chain. This repair performed
no hardware, credential, network-discovery, reset, flash, monitor, or push action.
