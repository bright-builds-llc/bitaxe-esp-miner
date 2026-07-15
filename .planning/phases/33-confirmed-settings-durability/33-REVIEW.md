---
phase: 33-confirmed-settings-durability
reviewed: 2026-07-15T01:55:03Z
generated_at: 2026-07-15T01:55:03Z
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
  warning: 1
  info: 0
  total: 1
---

# Phase 33: Code Review Report

**Reviewed:** 2026-07-15T01:55:03Z
**Depth:** standard
**Files Reviewed:** 21
**Status:** issues_found

## Summary

Commits `4dcf1b7`, `f307336`, `49c5bca`, `a0e4d19`, and `5e67223` resolve the finalizer, fake-backed flow, poisoned-snapshot retention, secret-safe wrapper formatting, confirmed-success versus best-effort-effect semantics, and response-before-effect ownership findings. Restart remains on its mandatory pre-response ownership path; dropped leases discard effects without inline execution, queue tests cover ownership/release/disconnection, and the Phase 33 heartbeat, redaction boundary, public response shapes, and committed hardware facts are preserved.

One warning remains in the original raw-root finding. Repository-local roots are required to match an ignore rule, but the check uses `git check-ignore --no-index`; that option intentionally ignores index membership and therefore accepts a tracked path whenever an ignore rule also matches it. The existing regression covers a tracked non-ignored path, not the tracked-and-ignored case.

No critical issue or direct secret value was found in the reviewed committed artifacts. This review did not access hardware, credentials, protected raw evidence, or any raw device trace. Supporting deferred-effect and confirmed-snapshot modules were inspected only as call-chain context; the explicit reviewed scope and count remain the requested 21 files.

Material guidance loaded for this review: `AGENTS.md` (including protected evidence, timeout, hardware, no-UART/pin, and frontmatter rules), `AGENTS.bright-builds.md`, `standards-overrides.md`, `standards/index.md`, `standards/core/architecture.md`, `standards/core/code-shape.md`, `standards/core/operability.md`, `standards/core/testing.md`, `standards/core/verification.md`, and `standards/languages/rust.md`.

## Finding Disposition

| Area | Disposition | Evidence |
| --- | --- | --- |
| Raw-root containment | Partially resolved; WR-01 remains | Canonicalization, symlink rejection, ignored-root gating, modes, and pre-command failure are present, but `--no-index` bypasses the tracked-path distinction. |
| Finalizer and restoration | Resolved | The idempotent finalizer preserves exit/signal status, reaps the passive reader, restores a changed hostname, and reports category-only outcomes; cancellation and errexit simulations pass. |
| Fake-backed production flow | Resolved | Detector, flash, classifier, identity, HTTP, monitor, and checkpoint fakes drive the production wrapper through success and failure branches. |
| Poison recovery and formatting | Resolved | Reads retain the poisoned mutex inner snapshot with degraded health, while custom `Debug` implementations expose health only; sentinel-secret regressions pass. |
| Confirmed success and best-effort settings effect | Resolved | Durable commit/reload/reconcile/publication remains authoritative for empty `200` success; worker acquisition/release failure only records a category-safe degraded marker. |
| Restart and deferred queue ownership | Resolved | Restart ownership is acquired before response serialization and released after it; there is no inline fallback, and dropped leases discard effects. |
| Public shapes, heartbeat, and evidence | Resolved/preserved | Hostname authority stays closed and redacted, response bodies are unchanged, boot identity/origin/heartbeat checks pass, and the committed hardware summary is byte-unchanged. |

## Warning Finding

### WR-01: `--no-index` allows a tracked path to satisfy the raw-root ignore gate

**File:** `/Users/peterryszkiewicz/Repos/bitaxe-esp-miner/scripts/phase33-confirmed-settings-durability.sh:156-159`

**Issue:** `prepare_local_root` accepts an in-repository root when `git check-ignore -q --no-index` succeeds. Git documents and implements `--no-index` specifically so tracked files are also checked against ignore rules. A tracked path that is later covered by an ignore rule, or was force-added while ignored, therefore passes this gate and can receive detector logs, HTTP bodies, classifier JSON, and raw serial evidence. A read-only probe against tracked `AGENTS.md` with an injected ignore rule returned success with `--no-index` and failure without it. The current test rejects `.planning/...` only because that tracked path is not ignored, so it does not guard the tracked-and-ignored case.

**Fix:** Remove `--no-index` so ordinary `git check-ignore` fails for tracked paths, or add an explicit `git ls-files --error-unmatch` rejection before the ignore check. Add a regression using a temporary Git fixture containing a force-added ignored directory and assert rejection occurs before detector, flash, or HTTP calls.

## Verification

- `cargo fmt --all -- --check` passed.
- `cargo clippy --all-targets --all-features -- -D warnings` passed.
- `cargo test -p bitaxe-api -p bitaxe-config` passed: 167 API tests and 49 config tests.
- `cargo test -p bitaxe-parity phase33_source_guard` passed: 11 matching tests.
- `bash -n`, `shellcheck`, and `shfmt -d` passed for the wrapper, test, and fake fixture scripts.
- `bash scripts/phase33-confirmed-settings-durability-test.sh` passed, including cancellation, errexit, restoration, cleanup, fake-backed failures, and the current unsafe-root cases.
- Bazel tests passed for `//scripts:phase33_confirmed_settings_durability_test`, `//crates/bitaxe-api:tests`, `//crates/bitaxe-config:tests`, and `//tools/parity:tests` with the Phase 33 filter.
- Read-only Git proof passed: a tracked path with an injected ignore rule is reported by `git check-ignore --no-index` and rejected by ordinary `git check-ignore`.
- `git diff --exit-code 323e5e4..HEAD -- docs/evidence/phase-33/hardware-summary.md` passed; the recorded hardware summary is unchanged.
- The Phase 33 hardware summary passed the sensitive-output denylist.
- `git diff --check 4dcf1b7^..HEAD` passed.

***

_Reviewed: 2026-07-15T01:55:03Z_
_Reviewer: gsd-code-reviewer_
_Depth: standard_
