---
phase: 12
slug: asic-and-mining-hardware-evidence
status: passed
nyquist_compliant: true
wave_0_complete: true
created: 2026-06-30T00:14:49Z
generated_by: gsd-plan-phase
lifecycle_mode: yolo
phase_lifecycle_id: 12-2026-06-30T00-13-19
---

# Phase 12 - Validation Strategy

Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
| --- | --- |
| Framework | Rust unit tests through Cargo and Bazel, existing shell/script tests where repo scripts own behavior, parity validation, and detector-gated hardware evidence when available |
| Config file | `Cargo.toml`, `MODULE.bazel`, `Justfile`, `tools/flash/BUILD.bazel`, `tools/parity/BUILD.bazel`, relevant crate `BUILD.bazel` files |
| Quick run command | `cargo test -p bitaxe-asic -p bitaxe-stratum -p bitaxe-safety -p bitaxe-api --all-features` |
| Full suite command | `cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo build --all-targets --all-features && cargo test --all-features && just parity` |
| Hardware gate command | `just detect-ultra205` |
| Hardware evidence command | `just flash-monitor board=205 port=<path> evidence-dir=docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence` or a Phase 12 probe command documented by the active plan |
| Estimated runtime | Targeted host checks: 2-5 minutes. Full Rust pre-commit checks and hardware smoke/soak are environment-dependent. |

## Sampling Rate

- After every task commit: run the narrow automated command named in that task's acceptance criteria.
- After every plan wave: run the affected crate tests plus `just parity` when checklist or evidence rows changed.
- Before live hardware: run `just detect-ultra205`; continue only with exactly one successful `port=<path>`.
- After live hardware: perform and record redaction review before staging generated logs or JSON.
- Before verification: run the full suite command, lifecycle validation, and any hardware evidence commands allowed by the plan and detector result.
- Max feedback latency: no three consecutive implementation tasks may proceed without automated feedback or an explicit manual-only evidence note.

## Per-Requirement Verification Map

| Requirement | Behavior | Test Type | Automated Command | Manual Or Hardware Evidence | Status |
| --- | --- | --- | --- | --- | --- |
| ASIC-07 | BM1366 initialization, work-send, and result-receive have exact hardware-smoke evidence before release parity is claimed. | unit plus hardware smoke | `cargo test -p bitaxe-asic --all-features` | Detector-gated safe boot passed; chip-detect emitted fail-closed markers but wrapper trust failed; diagnostic work/result did not run. Checklist rows remain below verified. | passed with hardware evidence pending |
| STR-06 | First Ultra 205 mining loop connects config, Stratum v1, BM1366 dispatch/result parsing, state, and safety gates. | unit/integration plus hardware smoke | `cargo test -p bitaxe-stratum -p bitaxe-asic -p bitaxe-safety -p bitaxe-api --all-features` | Trusted safe boot and preflight showed `mining_loop_status=blocked` and `work_submission=disabled`; controlled first-loop mining did not run. | passed with hardware evidence pending |
| STR-07 | Mining smoke/soak records command, board, port, commits, logs, result, redaction, and conclusion. | artifact validation plus hardware smoke/soak | `just parity` plus Phase 12 evidence checks | Criteria, redaction, and preflight artifacts exist; bounded smoke/soak did not run and no share/no-share outcome exists. | passed with hardware evidence pending |
| EVD-05 | Verification layers include unit tests, golden fixtures, API comparison, hardware smoke, and hardware regression or soak evidence where appropriate. | full verification | Full suite command | Final verification passed; unsupported hardware claims remain explicitly pending and unpromoted. | passed |

## Wave 0 Requirements

- [x] `docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md` - create the Phase 12 ledger/runbook/claim matrix for ASIC-07, STR-06, STR-07, and EVD-05.
- [x] Redaction contract - define how generated serial logs, command JSON, pool information, NVS/config values, and worker identifiers are reviewed before commit.
- [x] Hardware gate - run `just detect-ultra205` before any live board interaction and record detector output when live evidence is attempted.
- [x] Checklist-promotion rule - document exact claim-to-evidence mapping before any `docs/parity/checklist.md` row is promoted.

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
| --- | --- | --- | --- |
| Ultra 205 hardware presence and board-info capture | ASIC-07, STR-07 | Requires connected board over USB. | Run `just detect-ultra205`; if exactly one port is confirmed, record port, board-info, source commit, and reference commit. |
| BM1366 live chip-detect and staged init | ASIC-07 | Requires live ASIC/UART/reset path. | Run the plan-approved wrapper/probe after detector and recovery gates; record logs, observations, fail-closed behavior, and conclusion. |
| Diagnostic work-send/result-receive | ASIC-07 | Requires live ASIC work/result behavior or a controlled block. | Use typed bounded diagnostic work when available; record expected and observed result/timeout behavior. |
| Controlled mining smoke/soak | STR-06, STR-07 | Requires hardware, controlled pool or fake-pool condition, safety monitoring, and redaction. | Use bounded duration and stop conditions; record accepted/rejected shares or controlled no-share rationale. |
| Redaction review | STR-07, EVD-05 | Generated logs/configs may include secrets or private endpoints. | Inspect all generated evidence before commit; record result in the ledger or `redaction-review.md`. |

## Security Threat Coverage

| Threat Ref | Surface | Secure Behavior | Validation |
| --- | --- | --- | --- |
| T-12-01 | Evidence files | Do not record pool credentials, worker secrets, Wi-Fi credentials, private endpoints, NVS secrets, or unredacted environment values. | Redaction review before verification and commit. |
| T-12-02 | Live mining commands | Live mining and soak stay bounded by detector, recovery path, stop conditions, and safety gates. | Plan review, command review, and hardware evidence log. |
| T-12-03 | Checklist promotion | ASIC/mining rows cannot become verified unless exact evidence supports the exact claim. | `just parity` and any added parity tests. |

## Validation Sign-Off

- [x] All tasks have automated verification or a documented manual-only evidence path.
- [x] Sampling continuity: no three consecutive implementation tasks without automated verification or explicit evidence-gated pending status.
- [x] Wave 0 covers all missing evidence references from research.
- [x] No watch-mode flags in verification commands.
- [x] Hardware interaction follows the Ultra 205 detector and evidence rules.
- [x] `nyquist_compliant: true` set in frontmatter only when the executed phase satisfies this contract.

Approval: approved 2026-06-30
