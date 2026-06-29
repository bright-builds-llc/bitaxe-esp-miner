---
phase: 10
slug: route-manifest-and-api-compare-unification
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-29
---

# Phase 10 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
| --- | --- |
| **Framework** | Bazel `rust_test` targets for `crates/bitaxe-api` and `tools/parity`; Rust unit tests live in `crates/bitaxe-api/src/route_shell.rs` and `tools/parity/src/api_compare.rs`. |
| **Config file** | `crates/bitaxe-api/BUILD.bazel`, `tools/parity/BUILD.bazel`, `tools/parity/fixtures/api/*.json`, and `Justfile`. |
| **Quick run command** | `bazel test //crates/bitaxe-api:tests //tools/parity:tests` |
| **API compare command** | `bazel run //tools/parity:report -- api-compare` |
| **Checklist guard command** | `just parity` |
| **Full suite command** | `just test`; before commit also run `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`. |
| **Estimated runtime** | Quick: under 45 seconds after warm cache; full suite varies with Bazel/Cargo cache state. |

## Sampling Rate

- **After every route manifest task:** Run `bazel test //crates/bitaxe-api:tests`.
- **After every API compare task:** Run `bazel test //tools/parity:tests` and `bazel run //tools/parity:report -- api-compare`.
- **After every checklist or evidence task:** Run `just parity`.
- **After every plan wave:** Run `bazel test //crates/bitaxe-api:tests //tools/parity:tests`, `bazel run //tools/parity:report -- api-compare`, and `just parity`.
- **Before phase verification:** Run `just test`, `bazel run //tools/parity:report -- api-compare`, `just parity`, and the Rust pre-commit sequence required by `AGENTS.md`.
- **Max feedback latency:** Keep targeted route/parity feedback under 45 seconds after warm cache; full-suite latency is acceptable only at wave/phase boundaries.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 10-01-01 | 01 | 1 | API-09/REL-01 | T-10-01 | `phase07_routes()` remains the manifest source for `/api/system/OTA`, `/api/system/OTAWWW`, `/recovery`, and `/*` with expected methods and `RouteKind` values. | unit | `bazel test //crates/bitaxe-api:tests` | Yes | pending |
| 10-01-02 | 01 | 1 | EVD-01 | T-10-02 | Firmware route reporting derives route count and ownership metadata from the Phase 7 manifest instead of `phase05_routes().len()`. | unit/build | `bazel test //crates/bitaxe-api:tests`; `bazel build //firmware/bitaxe:firmware` when firmware imports change | Yes | pending |
| 10-02-01 | 02 | 1 | API-10/REL-02/REL-03 | T-10-03 | API compare route checks consume Phase 7 typed routes and fail on missing route or `RouteKind` downgrade. | unit + API compare | `bazel test //tools/parity:tests`; `bazel run //tools/parity:report -- api-compare` | Yes | pending |
| 10-02-02 | 02 | 1 | EVD-01 | T-10-04 | API compare rejects fixture or policy overclaims that mark static, recovery, firmware OTA, OTAWWW, or release-sensitive routes verified from weak evidence. | unit + parity | `bazel test //tools/parity:tests`; `just parity` | Yes | pending |
| 10-03-01 | 03 | 2 | EVD-01/REL-01/REL-02/REL-03 | T-10-05 | Checklist and evidence record Phase 10 manifest/tooling proof while keeping live HTTP/static/recovery/OTA/release rows below verified. | docs/parity | `just parity` | Yes | pending |

*Status: pending, green, red, flaky.*

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. Add focused regression tests before or alongside behavior changes:

- [ ] `crates/bitaxe-api/src/route_shell.rs` tests for Phase 7 route count and ownership report metadata.
- [ ] `tools/parity/src/api_compare.rs` tests for missing Phase 7 route failure.
- [ ] `tools/parity/src/api_compare.rs` tests for firmware OTA, OTAWWW gap, recovery, and static wildcard kind downgrades.
- [ ] `tools/parity/src/api_compare.rs` tests for verified-overclaim failure on static/recovery/update route policy.

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
| --- | --- | --- | --- |
| Diff review for no live behavior overclaim | EVD-01 | The claim boundary is semantic and spans docs plus checklist wording. | Review the final diff and confirm Phase 10 evidence says manifest/tooling only; `API-007`, `FS-001`, `OTA-001`, `OTA-002`, `REL-001`, `REL-002`, and `REL-003` remain below verified unless later live evidence exists. |

## Validation Sign-Off

- [ ] All tasks have automated verify or documented manual review.
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify.
- [ ] Existing test infrastructure covers all missing references.
- [ ] No watch-mode flags.
- [ ] Feedback latency for quick checks is under 45 seconds after warm cache.
- [ ] `nyquist_compliant: true` set in frontmatter after execution evidence confirms the contract.

**Approval:** pending
