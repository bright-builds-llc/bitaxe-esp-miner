---
status: resolved
trigger: "Attempt 26 repeated approved_reboot_failed after the targeted passive-monitor runfiles repair."
created: 2026-07-22T15:58:27Z
updated: 2026-07-23T02:06:21Z
---

## Current Focus

hypothesis: Confirmed. Attempt 26 bound runtime observation to one espflash process/device node instead of a qualified receive-only lifecycle owner. The repaired host layer keeps USB physical identity stable while allowing enumeration replacement and attributes restart only after a complete one-shot request plus HTTP Boot B quorum.
test: Completed focused and repository-wide Rust/Bazel tests for the request/projection schema, terminal precedence, private artifact modes/caps, request ambiguity, stable-node and same-device re-acquisition, and shared HTTP transport.
expecting: The deterministic device-session layer passes without hardware or device-network access and Phase 35 no longer selects espflash for runtime reboot observation.
next_action: Commit the verified software repair atomically, run exact-head Phase 35 preflight, then start the separately authorized Attempt 27 only if the repository remains clean and every preflight gate passes.

## Symptoms

expected: The approved reboot produces a passive post-loss serial capture containing the same-board Boot B identity and the next boot ordinal.
actual: The passive monitor reached readiness and exclusive ownership, the reboot request completed, service loss occurred, and post-cleanup readiness passed, but the entire capture remained zero bytes.
errors: Public primary category is `approved_reboot_failed`; private Boot B classifier category is `post_restart_identity_missing`; restoration and cleanup have no secondary failures.
reproduction: Attempt 26 is sealed and must not be reused. A repeated attempt is prohibited by the progress-gated hardware policy.
started: Attempt 26 at exact source `a4de3c3a480bb29075c1c17df5c7cb8fe9d69f7c` after doctor and exact-head preflight passed.

## Eliminated

- Attempt 25 runfiles defect: the passive helper loaded, created its protected artifacts, and reached active-owner readiness.
- Reboot request failure: the protected response exists, curl stderr is empty, and the supervisor continued through service-loss observation.
- Serial holder or cleanup failure: active ownership and post-cleanup readiness passed; cleanup has no secondary category.
- Restoration failure: the restoration HTTP projection classified `ready`, and the original setting was restored.
- Blind retry: the same public primary category recurred immediately after its targeted verified fix.

## Evidence

- timestamp: 2026-07-22T15:58:27Z
  checked: Attempt 26's sealed typed metadata and protected artifact-presence/size facts without printing device, network, credential, command, process, or path values.
  found: Passive readiness, reboot response, service-loss probe, monitor log, and Boot B classifier artifacts exist with mode `0600`; the raw serial capture is exactly zero bytes.
  implication: The helper closure and reboot path executed, but no serial identity evidence was available.
- timestamp: 2026-07-22T15:58:27Z
  checked: Closed safe fields from the passive monitor and Boot B classifier.
  found: Capture disposition is `timed_out_after_capture`; trace status is complete; pre/post readiness and active ownership passed; Boot B category is `post_restart_identity_missing`.
  implication: The failure moved beyond Attempt 25's missing runfile but retained the same public primary category.
- timestamp: 2026-07-22T15:58:27Z
  checked: The repository hardware-attempt decision contract.
  found: A primary category recurring once after its targeted verified fix selects `stop_repeated_boundary`.
  implication: Attempt 27 is prohibited even though the private subcategory is more specific.
- timestamp: 2026-07-22T20:48:14Z
  checked: The complete Attempt 26 shell reader path, serial-session helpers, HTTP boundary, common bug patterns, and active repository lessons.
  found: The monitor is hard-coded to `espflash`, binds one tty path for the full capture, and verifies identity only at pre-attach/post-cleanup. Repository lessons separately record passive espflash yielding zero bytes while the receive-only OS-native reader yielded heartbeats, and require stable physical identity to be distinct from enumeration identity.
  implication: The reusable repair belongs in a host-side capability/state-machine layer; it must model disappearance and same-device re-acquisition rather than treating one child process on one device node as a durable session.
- timestamp: 2026-07-22T21:19:46Z
  checked: New `device-session` library/CLI, macOS adapter, evidence writer, typed fixture seam, and shared HTTP transport integration.
  found: Cargo test-target checks and strict Clippy pass for `bitaxe-device-session`, `bitaxe-http-transport`, and `bitaxe-parity`. The model requires a complete request write before ready, accepts a missing response only when the authoritative Boot B quorum succeeds, preserves the earliest terminal category, and keeps raw USB/HTTP/serial/application material out of the public projection.
  implication: The host-side repair compiles and is statically verified without hardware or network access.
- timestamp: 2026-07-22T21:19:46Z
  checked: Focused Cargo and Bazel test execution.
  found: Newly linked Rust test processes stall in macOS `_dyld_start` before the Rust harness lists or runs tests; Bazel likewise cannot connect to its newly starting local server within 80 seconds. The commands were terminated without hardware or network effects.
  implication: Behavioral tests exist and compile, but runtime verification is incomplete because of a host process-loader/tool startup blocker outside the test logic.
- timestamp: 2026-07-22T21:44:28Z
  checked: The extracted HTTP transport, stable-node recovery state, request-attribution precedence, real-CLI fixture matrix, shell supervisor integration, and local macOS process-policy state.
  found: Both device-session and the Phase 35 HTTP probe now consume one bounded TCP/TLS implementation. The model requires a complete request write and a post-request three-sample same-device qualification, supports reader loss/re-acquisition independently from USB disappearance, and prevents serial-only success from replacing an HTTP quorum failure. Focused Cargo check and strict Clippy pass; the complete hermetic Phase 35 shell suite, Bash syntax, ShellCheck warning gate, shfmt, and diff checks pass. Fresh Rust test binaries and a fresh Bazel server still stall before their entrypoints while macOS `syspolicyd` is active and developer mode reports disabled.
  implication: Repository behavior has been strengthened and statically verified, but the exact executable verification and every commit remain blocked on a host SystemPolicy recovery rather than a source failure.
- timestamp: 2026-07-23T02:06:21Z
  checked: The complete mandatory Rust sequence, focused device-session/HTTP/parity Cargo tests, focused and supporting Bazel suites, the hermetic Phase 35 shell suite, shell syntax/style, Markdown scope, redaction, reference, parity, lifecycle, doctor, and diff integrity.
  found: Every source and behavioral gate passes. The final zero-test host harness initially repeated the confirmed macOS loader stall, then launched and passed after an ad-hoc signature was applied only to the ignored local build artifact; no repository source, evidence, device, credential, or network state was changed by that recovery.
  implication: The software root cause and regression-backed repair are verified. The debug session can resolve before exact-head preflight and the authorized hardware attempt.

## Resolution

root_cause: Attempt 26 used a concrete fixed-path espflash monitor as though it were a qualified durable runtime-observation capability. That abstraction did not model USB disappearance/re-enumeration or re-acquire the same physical device, and prior repository evidence already showed this reader could own the node yet deliver zero application bytes where the OS-native receive-only reader delivered heartbeats.
fix: Added a reusable Rust device-session package/CLI with an exact-schema pure state machine, typed real-binary fixture seam, receive-only macOS ioreg/lsof adapter, stable-node and same-physical three-sample re-acquisition, reader-loss recovery, one-shot strict HTTP restart/recovery, bounded mode-0600 private artifacts beneath a caller-owned empty mode-0700 root, exact private Boot B result fields, closed public projection, approved terminal precedence, and explicit non-macOS rejection. Extracted one reusable strict HTTP request/response transport and made both device-session and the Phase 35 parity probe consume it.
verification: The exact mandatory Rust sequence passes in order: format, strict Clippy, all-target/all-feature build, and all-feature tests. Focused device-session, shared HTTP, parity, flash, detector, doctor, Phase 30, Phase 33, and Phase 35 Bazel suites pass. The direct hermetic Phase 35 shell suite, Bash syntax, ShellCheck warning gate, shfmt, Markdown checks for new files, redaction, reference, parity, exact lifecycle, doctor, and diff checks pass. No hardware, device, credential, push, or evidence-admission action occurred.
files_changed:

- .planning/debug/phase35-attempt26-post-restart-identity-missing.md
- Cargo.toml
- Cargo.lock
- tools/device-session/Cargo.toml
- tools/device-session/BUILD.bazel
- tools/device-session/src/evidence.rs
- tools/device-session/src/fixture.rs
- tools/device-session/src/lib.rs
- tools/device-session/src/live.rs
- tools/device-session/src/macos.rs
- tools/device-session/src/macos_unsupported.rs
- tools/device-session/src/main.rs
- tools/device-session/src/model.rs
- tools/device-session/src/model/tests.rs
- tools/device-session/tests/cli.rs
- tools/http-transport/Cargo.toml
- tools/http-transport/BUILD.bazel
- tools/http-transport/src/lib.rs
- tools/parity/Cargo.toml
- tools/parity/BUILD.bazel
- tools/parity/src/phase35_http_probe.rs
