---
phase: 33-confirmed-settings-durability
reviewed: 2026-07-15T00:59:06Z
generated_at: 2026-07-15T00:59:06Z
depth: standard
status: issues_found
generated_by: gsd-code-reviewer
lifecycle_mode: yolo
phase_lifecycle_id: 33-2026-07-14T01-50-49
files_reviewed: 21
files_reviewed_list:
  - crates/bitaxe-api/src/boot_identity.rs
  - crates/bitaxe-api/src/lib.rs
  - crates/bitaxe-api/src/phase33_evidence.rs
  - crates/bitaxe-api/src/settings.rs
  - crates/bitaxe-api/src/v12_settings.rs
  - crates/bitaxe-config/src/lib.rs
  - crates/bitaxe-config/src/persistence.rs
  - docs/evidence/phase-33/hardware-summary.md
  - firmware/bitaxe/BUILD.bazel
  - firmware/bitaxe/src/boot_evidence.rs
  - firmware/bitaxe/src/http_api.rs
  - firmware/bitaxe/src/main.rs
  - firmware/bitaxe/src/rtc_boot_ordinal.rs
  - firmware/bitaxe/src/runtime_snapshot.rs
  - firmware/bitaxe/src/settings_adapter.rs
  - firmware/bitaxe/src/wifi_adapter.rs
  - scripts/phase33-confirmed-settings-durability-test.sh
  - scripts/phase33-confirmed-settings-durability.sh
  - tools/parity/BUILD.bazel
  - tools/parity/src/main.rs
  - tools/parity/src/phase33_source_guard.rs
findings:
  critical: 0
  warning: 5
  info: 0
  total: 5
---

# Phase 33: Code Review Report

**Reviewed:** 2026-07-15T00:59:06Z
**Depth:** standard
**Files Reviewed:** 21
**Status:** issues_found

## Summary

The exact hostname-only authority, NVS write/commit/reload/reconcile/publication order, typed reboot classifier, and committed redacted hardware summary are coherent on their successful paths. The scoped host tests and shell checks pass. Five warning-level issues remain in failure-path recovery, raw-evidence containment, simulation fidelity, confirmed-snapshot poison handling, and post-response worker failure semantics.

No critical issue or direct secret value was found in the reviewed committed artifacts. The review did not access hardware, credentials, or protected raw evidence.

Material guidance loaded for this review: `AGENTS.md` (including the detector, passive serial, protected evidence, timeout, and no-UART/pin rules), `AGENTS.bright-builds.md`, `standards-overrides.md`, `standards/index.md`, `standards/core/architecture.md`, `standards/core/code-shape.md`, `standards/core/operability.md`, `standards/core/testing.md`, `standards/core/verification.md`, and `standards/languages/rust.md`.

## Warning Findings

### WR-01: `--local-root` can place protected raw evidence in a tracked repository path

**File:** `/Users/peterryszkiewicz/Repos/bitaxe-esp-miner/scripts/phase33-confirmed-settings-durability.sh:40-42,171-190`

**Issue:** The default local root is correctly under ignored `scratch/`, but the public `--local-root` option accepts any path. The wrapper creates/chmods that path and writes detector output, flash logs, passive raw serial bytes, classifier JSON, and HTTP request/readback bodies without checking that a repository-local path is gitignored. File mode `0600` protects OS access but does not prevent a caller from selecting a tracked path and later committing raw device URLs, USB/device identity data, hostnames, or other local identifiers. This violates the repo-local requirement that detailed traces live under mode-0700 gitignored roots.

**Fix:** Canonicalize the requested root and either constrain it to the fixed ignored `scratch/phase33-settings-durability/` subtree or fail closed unless `git check-ignore` proves a repository-local root is ignored. Reject symlinked roots and add a regression that a tracked `--local-root` fails before detector, flash, or HTTP work.

### WR-02: Signal and unexpected-errexit paths can leave the generated hostname persisted

**File:** `/Users/peterryszkiewicz/Repos/bitaxe-esp-miner/scripts/phase33-confirmed-settings-durability.sh:192-207,247-266,306-310`

**Issue:** `fail_proof` attempts restoration, but the `EXIT INT TERM` trap calls only `cleanup_passive`. After the proof PATCH sets `hostname_changed=1`, a user cancellation, termination signal, or any unguarded `set -e` failure bypasses `fail_proof` and exits without calling `restore_hostname`. The device can therefore retain the generated proof hostname, and no `restoration_failed` category is emitted. The same trap body for `INT`/`TERM` also does not explicitly preserve the signal exit status.

**Fix:** Install one idempotent finalizer that reaps the passive reader, attempts confirmed hostname restoration whenever `hostname_changed=1`, records a category-only restoration result, and then exits with the original status/signal. Guard against recursive traps and add cancellation plus unexpected-command-failure tests after the hostname PATCH.

### WR-03: The simulation matrix asserts labels without exercising the hardware workflow

**File:** `/Users/peterryszkiewicz/Repos/bitaxe-esp-miner/scripts/phase33-confirmed-settings-durability.sh:94-138`

**File:** `/Users/peterryszkiewicz/Repos/bitaxe-esp-miner/scripts/phase33-confirmed-settings-durability-test.sh:27-62`

**Issue:** Simulation mode maps each scenario name directly to its expected failure category and exits before any detector, manifest, flash, HTTP, passive-monitor, identity, cleanup, or restoration code runs. The test then checks that the hard-coded category was printed. Consequently, the full scenario suite can remain green if the corresponding production gate is deleted, reordered, or broken. The later source-string checks prove that selected tokens exist, not that the fail-closed branches work. This does not provide the fake-backed behavioral simulation required by the Phase 33 plan.

**Fix:** Route hardware operations through injectable command functions or a fake command directory, and run the same orchestration in tests with deterministic detector, flash, curl, classifier, monitor, process, and holder fakes. Each scenario should cause the production branch to observe the injected fault and prove cleanup/restoration side effects, rather than returning a preselected label.

### WR-04: A poisoned confirmed-snapshot lock is projected as default settings

**File:** `/Users/peterryszkiewicz/Repos/bitaxe-esp-miner/firmware/bitaxe/src/settings_adapter.rs:118-127`

**File:** `/Users/peterryszkiewicz/Repos/bitaxe-esp-miner/firmware/bitaxe/src/runtime_snapshot.rs:315-320`

**Issue:** `current_settings_snapshot` returns `NvsSnapshot::new()` when the mutex is poisoned. `apply_settings_snapshot` then feeds that empty snapshot through normal reload/default rules and projects the default hostname and other settings as ordinary runtime truth. Phase 33 requires readers to retain the prior confirmed snapshot or represent confirmation loss explicitly; manufacturing a default snapshot does neither and can make immediate reads disagree with committed storage after a panic.

**Fix:** Recover and clone the poisoned mutex's inner last-confirmed snapshot while emitting a category-only degraded-state marker, or change the read API to return an explicit unavailable/confirmation-lost result that the system-info projection cannot mistake for confirmed defaults. Add a poison-path regression.

### WR-05: Worker spawn failures break the declared response/effect ordering

**File:** `/Users/peterryszkiewicz/Repos/bitaxe-esp-miner/firmware/bitaxe/src/http_api.rs:322,411-419,1021-1041,1121-1133`

**Issue:** On settings-effects thread creation failure, `schedule_settings_effects` applies the hostname inline. The code's restart comment states that the HTTP response completes only after the handler returns, so this fallback can execute the live effect before response completion despite the post-response contract. The restart path has the opposite failure: after serializing the response, `schedule_restart_after_response` propagates a spawn error and never reaches `esp_restart`, leaving a response potentially sent without its promised effect. These branches are reachable under thread/resource exhaustion and are not covered by behavioral tests.

**Fix:** Use a process-lifetime worker/queue created before serving requests, or otherwise establish worker availability before emitting success. Never apply the hostname inline from the request handler. For restart, ensure a successfully returned command has an already-owned delayed effect path, and add injected worker-unavailable tests for both settings and restart ordering.

## Verification

- `cargo test -p bitaxe-api settings` passed: 23 matching tests.
- `cargo test -p bitaxe-api phase33` passed: 7 tests.
- `cargo test -p bitaxe-api boot_identity` passed: 5 tests.
- `cargo test -p bitaxe-config persistence` passed: 9 tests.
- `cargo test -p bitaxe-parity phase33_source_guard` passed: 10 tests.
- `bash -n scripts/phase33-confirmed-settings-durability.sh scripts/phase33-confirmed-settings-durability-test.sh` passed.
- `bash scripts/phase33-confirmed-settings-durability-test.sh` passed.
- `shellcheck scripts/phase33-confirmed-settings-durability.sh scripts/phase33-confirmed-settings-durability-test.sh` passed.
- `git diff --check c1381a163cb19571b52d62e7d091d84eaaf0d38e..HEAD` passed.

***

_Reviewed: 2026-07-15T00:59:06Z_
_Reviewer: gsd-code-reviewer_
_Depth: standard_
