---
phase: 03
slug: bm1366-asic-protocol-and-safe-initialization
status: approved
nyquist_compliant: true
wave_0_complete: false
created: 2026-06-26
---

# Phase 03 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust unit tests through Cargo and Bazel `rust_test` |
| **Config file** | `crates/bitaxe-asic/BUILD.bazel`, workspace `Cargo.toml`, `MODULE.bazel` |
| **Quick run command** | `cargo test -p bitaxe-asic --all-features` |
| **Full suite command** | `cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo build --all-targets --all-features && cargo test --all-features` |
| **Estimated runtime** | ~2-5 minutes |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p bitaxe-asic --all-features` plus any touched crate test.
- **After every plan wave:** Run `bazel test //crates/bitaxe-asic:tests`; when firmware adapter files are touched, also run the relevant firmware build/package command from the plan.
- **Before phase verification:** Run the full Rust pre-commit sequence from `AGENTS.md`.
- **Max feedback latency:** 5 minutes for pure crate feedback; hardware smoke remains manual-only.

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 03-01-01 | 01 | 1 | ASIC-01 | T-03-01 | Malformed command/job frames reject before adapter use | unit/golden | `cargo test -p bitaxe-asic bm1366_crc --all-features` | no - Wave 0 module gap | pending |
| 03-01-02 | 01 | 1 | ASIC-01 | T-03-01 | CRC5 and CRC16-FALSE outputs match reference-derived fixtures | unit/golden | `cargo test -p bitaxe-asic bm1366_packet --all-features` | no - Wave 0 module gap | pending |
| 03-02-01 | 02 | 1 | ASIC-02 | T-03-02 | Work encoding and result parsing reject invalid job IDs and malformed frames | unit/golden | `cargo test -p bitaxe-asic bm1366_work --all-features` | no - Wave 0 module gap | pending |
| 03-02-02 | 02 | 1 | ASIC-02 | T-03-02 | Nonce-derived ASIC/core/domain decoding is typed and bounded | unit/golden | `cargo test -p bitaxe-asic bm1366_result --all-features` | no - Wave 0 module gap | pending |
| 03-03-01 | 03 | 2 | ASIC-03 | T-03-03 | BM1366 is the only active V1 dispatch path; other ASICs are deferred/not verified | unit | `cargo test -p bitaxe-asic dispatch --all-features` | no - Wave 0 module gap | pending |
| 03-03-02 | 03 | 2 | ASIC-04 | T-03-01 | Firmware receives typed adapter actions, not raw BM1366 command bytes | unit/adapter | `cargo test -p bitaxe-asic transcript --all-features` | no - Wave 0 module gap | pending |
| 03-04-01 | 04 | 2 | ASIC-05 | T-03-03 | Missing board/config/power/thermal gates fail closed with no mining/work submission | unit | `cargo test -p bitaxe-asic init_plan --all-features` | no - Wave 0 module gap | pending |
| 03-04-02 | 04 | 2 | ASIC-06 | T-03-03 | Frequency and voltage are range-checked as pure decisions, not verified effects | unit | `cargo test -p bitaxe-asic frequency_voltage --all-features` | partial in `bitaxe-config` | pending |
| 03-05-01 | 05 | 3 | ASIC-07 | T-03-04 | Live init, work-send, and result-receive claims remain below verified without Ultra 205 evidence | evidence/static | `rg -n "ASIC-00[2-7]|BM1366" docs/parity docs/parity/evidence` | checklist exists, Phase 3 evidence gap | pending |
| 03-05-02 | 05 | 3 | ASIC-08 | T-03-05 | Breadcrumbs point to pinned reference behavior without copying C expression into MIT files | static review | `rg -n "reference/esp-miner|ASIC-00|BM1366" crates/bitaxe-asic firmware/bitaxe docs/parity` | no Phase 3 breadcrumbs yet | pending |

---

## Wave 0 Requirements

- [ ] `crates/bitaxe-asic/src/error.rs` - typed protocol, init, and transcript failures.
- [ ] `crates/bitaxe-asic/src/bm1366.rs` plus `crates/bitaxe-asic/src/bm1366/` - CRC, packet, registers, work, result, init plan, and transcript modules.
- [ ] `crates/bitaxe-asic/fixtures/bm1366/` or an equivalent fixture path - metadata-rich reference-derived cases.
- [ ] `crates/bitaxe-asic/BUILD.bazel` - new source files and tests visible to Bazel.
- [ ] `docs/parity/checklist.md` - ASIC rows updated with pure fixture evidence and live evidence status boundaries.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Ultra 205 chip-detect smoke | ASIC-05, ASIC-07 | Requires connected Ultra 205 hardware, safe bench setup, serial port, and board-named log evidence | Run the plan-defined chip-detect command, record board, port, firmware commit, reference commit, logs, observed chip count, skipped gates, and fail-closed conclusion in `docs/parity/evidence/` |
| Full staged init smoke | ASIC-05, ASIC-07 | Safety-critical reset/UART/frequency/init behavior cannot be verified by unit tests | Run only after board/config/power/thermal preflight tokens exist; capture logs and leave checklist below `verified` if any gate is missing |
| Diagnostic work-send/result-receive smoke | ASIC-02, ASIC-07 | Production mining is Phase 4; any Phase 3 work path must be diagnostic-only and fail-closed | Run only if a plan explicitly adds diagnostic work mode; confirm no Stratum pool mining or accepted-share claim is made |

---

## Validation Sign-Off

- [x] All tasks have automated verify commands or Wave 0 dependencies.
- [x] Sampling continuity: no 3 consecutive tasks without automated verification.
- [x] Wave 0 covers all missing module and fixture references.
- [x] No watch-mode flags.
- [x] Feedback latency target is under 5 minutes for pure crate work.
- [x] `nyquist_compliant: true` set in frontmatter.

**Approval:** approved 2026-06-26
