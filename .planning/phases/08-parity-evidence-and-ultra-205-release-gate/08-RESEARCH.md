# Phase 8: Parity Evidence And Ultra 205 Release Gate - Research

**Researched:** 2026-06-28
**Domain:** Ultra 205 parity evidence governance, release-gate validation, HTTP/OTA/recovery hardware evidence
**Confidence:** HIGH for codebase and workflow findings; MEDIUM for live HTTP reachability until a Phase 8 hardware run establishes a device URL.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

## Implementation Decisions

### Evidence Governance And Claim Policy

- **D-01:** Treat `docs/parity/checklist.md` as the release audit ledger. Every V1 parity surface must keep observable behavior, reference breadcrumb, Rust-owned target, status, evidence, and notes aligned before release readiness is claimed.
- **D-02:** Preserve the project meaning of `verified`: evidence-backed parity, not implementation completion. Package, compile, unit, golden, and API compare evidence can support appropriate rows, but live hardware, release-sensitive, OTA, rollback, recovery, and interrupted-update claims need their required evidence class before `verified`.
- **D-03:** Keep safety-critical and hardware-control rows below `verified` unless they have `hardware-smoke` or `hardware-regression` evidence. This includes voltage, fan, thermal, power, ASIC initialization, self-test hardware, runtime input, and runtime display surfaces.
- **D-04:** Add or tighten automated parity guards where release claims could otherwise drift, rather than relying on prose review alone.

### Ultra 205 Release Evidence Workflow

- **D-05:** Use the repo-local Ultra 205 hardware workflow before live hardware checks: run `just detect-ultra205`, continue only when exactly one likely ESP USB serial port is found and `espflash board-info --chip esp32s3 --port <port> --non-interactive` succeeds, then record that detector output in evidence.
- **D-06:** Phase 8 should prioritize the deferred live release surfaces from Phase 7: HTTP reachability, `/`, `/assets/app.css.gz`, missing static redirect behavior, `/recovery`, valid firmware OTA, invalid OTA rejection, OTAWWW gap response, rollback or boot-validation evidence, large erase recovery, failed update recovery, and interrupted-update recovery.
- **D-07:** Destructive or fault-injection checks are allowed only through phase-gated procedures that document the recovery path and required evidence. Do not run ad hoc erase, rollback, interrupted-update, voltage/fan/mining stress, or raw write commands outside those documented procedures.
- **D-08:** If the board is not reachable over HTTP or a destructive recovery path cannot be made explicit, record the evidence as pending or not run with a concrete blocker. Do not promote the corresponding checklist row.

### Release Gate And Documentation Closure

- **D-09:** Extend the existing `tools/parity` release-gate/checklist validation path instead of creating a second release-readiness tool. `just parity` and `bazel run //tools/parity:report -- release-gate` should remain the canonical command evidence.
- **D-10:** Release readiness should be derived from the package manifest, license inventory, provenance manifest, release operator guide, parity checklist, and evidence records together. A final release summary may be added, but it must cite concrete artifacts and command results rather than restating goals.
- **D-11:** Package artifacts must be tied to source commit, reference commit, checksums, tool versions, and artifact paths before publication. Generated firmware images remain GPL-risk-reviewed release artifacts until provenance and license review are explicitly complete.
- **D-12:** Keep `docs/release/ultra-205.md`, `docs/release/license-inventory.md`, `docs/release/provenance-manifest.md`, and `docs/parity/evidence/phase-07-ota-filesystem-release.md` as the starting release evidence set; update them only where Phase 8 produces new evidence or closes known gaps.

### Deferred Scope And Gap Handling

- **D-13:** Non-205 boards, deferred ASIC families, Stratum v2, BAP completeness, all-board factory image matrices, and an Angular AxeOS rewrite remain outside V1 release closure unless a later roadmap phase adds their own evidence path.
- **D-14:** OTAWWW remains an explicit REL-03 gap unless Phase 8 implements and proves whole-`www` partition update behavior, recovery access, and interrupted-update recovery on Ultra 205 with hardware-regression evidence.
- **D-15:** It is acceptable for Phase 8 to ship with explicit V1 gaps only when the checklist, release docs, evidence files, and release gate all make the gap, owner, impact, and follow-up path visible.

### the agent's Discretion

The agent may choose the exact Phase 8 plan split, evidence document names, release summary shape, test helper names, and whether validation logic lands in `tools/parity/src/main.rs` or `tools/parity/src/release_gate.rs`. Those choices must preserve the existing functional-core/imperative-shell boundary, keep upstream reference files read-only, avoid overclaiming verified status, and keep hardware evidence records free of secrets, private endpoints, pool credentials, Wi-Fi credentials, or NVS secret values.

### Deferred Ideas (OUT OF SCOPE)

- Gamma 601/BM1370 and other non-205 board verification remain future-board phases.
- Stratum v2 completeness remains future protocol scope.
- BAP accessory parity remains future accessory scope.
- Angular AxeOS UI replacement remains out of V1; V1 keeps API and asset compatibility.
- All-board factory image matrices remain future release automation after each board has evidence.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
| --- | --- | --- |
| REL-08 | Rollback, recovery, large erase, failed update, and interrupted update cases have verification evidence before release parity is claimed. | Plan from `docs/release/ultra-205.md` procedures, Phase 7 deferred evidence, ESP-IDF OTA rollback semantics, `just detect-ultra205`, `just package`, `just flash-monitor`, and HTTP/OTA probes. [VERIFIED: .planning/REQUIREMENTS.md] [VERIFIED: docs/release/ultra-205.md] [CITED: https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/ota.html] |
| EVD-01 | Each V1 parity surface records observable behavior, reference breadcrumb, Rust implementation pointer, status, evidence, and notes. | Audit and update `docs/parity/checklist.md`; its table already has exactly those seven columns and `tools/parity` parses them. [VERIFIED: docs/parity/checklist.md] [VERIFIED: tools/parity/src/main.rs] |
| EVD-02 | `verified` means evidence-backed parity, not implemented code. | Preserve and extend `tools/parity` verified-row guards; current baseline rejects pending evidence, safety-critical overclaims, and release/OTA row overclaims. [VERIFIED: tools/parity/src/main.rs] [VERIFIED: just parity] |
| EVD-03 | Non-205 boards and ASICs stay unverified or deferred until each has its own evidence set. | Keep Gamma 601/BM1370 and other non-205 rows deferred/not-started, and do not inherit Ultra 205 evidence. [VERIFIED: docs/adr/0014-pivot-to-ultra-205-bm1366-first-parity.md] [VERIFIED: docs/parity/checklist.md] |
| EVD-04 | Rust modules porting reference behavior include module-level or behavior-level breadcrumbs without line-by-line translation comments. | Use `rg` breadcrumb audit across `crates/`, `firmware/`, and `tools/`; existing modules already show the intended pattern. [VERIFIED: docs/adr/0008-reference-breadcrumb-comments.md] [VERIFIED: codebase rg] |
| EVD-05 | Verification layers include unit tests, golden fixtures, API comparison, hardware smoke tests, and hardware regression or soak evidence where appropriate. | Preserve layered evidence classes in checklist and require hardware-smoke/regression for live release and safety-critical claims. [VERIFIED: docs/parity/checklist.md] [VERIFIED: docs/adr/0012-parity-verification-evidence.md] |
</phase_requirements>

## Summary

Phase 8 should be planned as evidence closure plus release-gate hardening, not as feature expansion. The project already has a release ledger (`docs/parity/checklist.md`), release documents, package manifest generation, `tools/parity` validation, and Ultra 205 detection/flash evidence paths; Phase 8 should connect those assets to live HTTP/OTA/recovery observations and tighten guards where a row could otherwise be overclaimed. [VERIFIED: 08-CONTEXT.md] [VERIFIED: docs/parity/checklist.md] [VERIFIED: tools/parity/src/main.rs] [VERIFIED: tools/parity/src/release_gate.rs]

The most important planning dependency is HTTP reachability. Phase 7 proved corrected factory flash, partition layout, SPIFFS mount, boot-validation entry, and HTTP route registration over serial, but it did not discover a reachable device URL because the captured firmware did not expose Wi-Fi, AP, DHCP, mDNS, hostname, or IP address information. [VERIFIED: docs/parity/evidence/phase-07-ultra-205-ota-hardware-smoke.md]

**Primary recommendation:** First establish or explicitly fail `DEVICE_URL` discovery, then collect live Ultra 205 evidence for static/recovery/OTA/rollback surfaces, then update checklist/release docs and `tools/parity` guards so release readiness is derived from evidence classes rather than implementation status. [VERIFIED: 08-CONTEXT.md] [VERIFIED: docs/release/ultra-205.md]

## Project Constraints (from AGENTS.md)

- Use GSD workflow artifacts for repo edits; this research is part of the GSD phase workflow and should write only the Phase 8 research file. [VERIFIED: AGENTS.md]
- Follow Bright Builds rules, including functional core / imperative shell, repo-native verification, and Rust testing conventions. [VERIFIED: AGENTS.md] [VERIFIED: AGENTS.bright-builds.md] [VERIFIED: standards/core/architecture.md] [VERIFIED: standards/core/testing.md] [VERIFIED: standards/languages/rust.md]
- Keep release-gate logic pure and host-testable where practical; isolate filesystem, CLI, HTTP, ESP-IDF, flash, and hardware effects in thin adapters. [VERIFIED: standards/core/architecture.md] [VERIFIED: standards/languages/rust.md]
- Use `foo.rs` plus `foo/` for new multi-file Rust modules, use early returns and `let...else` for guard extraction, prefix internal `Option<T>` names with `maybe_`, and avoid `unwrap()`. [VERIFIED: standards/languages/rust.md] [VERIFIED: AGENTS.md]
- Unit tests for pure logic must focus on one concern and use Arrange/Act/Assert comments unless the structure is trivial. [VERIFIED: standards/core/testing.md] [VERIFIED: AGENTS.md]
- Before any commit in this Rust repo, run `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`; for GSD execution plans, prefer repo-owned `just`/Bazel verification where it captures the intended workflow. [VERIFIED: AGENTS.md] [VERIFIED: standards/core/verification.md]
- Use `just detect-ultra205` before autonomous Ultra 205 hardware use; continue only with exactly one likely ESP USB serial port and a successful `espflash board-info --chip esp32s3 --port <port> --non-interactive`. [VERIFIED: AGENTS.md] [VERIFIED: scripts/detect-ultra205.sh]
- Destructive or fault-injection hardware verification requires documented recovery paths and required evidence before execution; do not run ad hoc erase, rollback, interrupted-update, voltage/fan/mining stress, or raw write commands outside the plan. [VERIFIED: AGENTS.md] [VERIFIED: docs/release/ultra-205.md]
- Hardware evidence must record board `205`, port, source commit, reference commit, package artifacts or manifest, exact commands, `board-info`, logs, observed behavior, and conclusion, without committing secrets, pool credentials, Wi-Fi credentials, private endpoints, or NVS secret values. [VERIFIED: AGENTS.md]
- GSD/frontmatter Markdown must not use standalone `---` body separators after frontmatter; use headings or `***` if a separator is needed. [VERIFIED: AGENTS.md] [VERIFIED: tasks/lessons.md]
- Upstream `reference/esp-miner` is read-only behavioral evidence; do not modify it or copy upstream source expression into MIT-only Rust files. [VERIFIED: AGENTS.md] [VERIFIED: PROVENANCE.md] [VERIFIED: docs/adr/0005-read-only-reference-implementation.md]

## Standard Stack

### Core

| Library / Tool | Version | Purpose | Why Standard |
| --- | --- | --- | --- |
| `tools/parity` (`bitaxe-parity`) | `0.1.0` | Checklist reporting, API compare, release-gate validation. | Existing canonical parity/release-gate path; Phase 8 context forbids a second release-readiness tool. [VERIFIED: cargo metadata] [VERIFIED: 08-CONTEXT.md] |
| `tools/flash` (`bitaxe-flash`) | `0.1.0` | Flash, monitor, flash-monitor, and flash evidence JSON/log capture. | Existing `just flash`, `just monitor`, and `just flash-monitor` backend with manifest-aware factory image selection. [VERIFIED: cargo metadata] [VERIFIED: tools/flash/src/main.rs] |
| `crates/bitaxe-api` | `0.1.0` | Pure HTTP route, OTA, OTAWWW, static, recovery, and API behavior decisions. | Existing functional-core home for host-testable route decisions. [VERIFIED: cargo metadata] [VERIFIED: crates/bitaxe-api/src/update_plan.rs] [VERIFIED: crates/bitaxe-api/src/static_plan.rs] |
| Bazel | `9.1.1` | Canonical automation graph. | `Justfile` routes build/test/package/parity through Bazel targets. [VERIFIED: command availability] [VERIFIED: Justfile] |
| `just` | `1.48.0` | Human command surface. | Repo-local workflow exposes `build`, `test`, `package`, `flash`, `monitor`, `flash-monitor`, `detect-ultra205`, `verify-reference`, and `parity`. [VERIFIED: command availability] [VERIFIED: Justfile] |
| `espflash` | `4.0.1` | ESP32-S3 board-info, flash, monitor, and image workflow backend. | Repo scripts and tools use `espflash list-ports`, `board-info`, `write-bin`, and `monitor`. [VERIFIED: command availability] [VERIFIED: scripts/detect-ultra205.sh] [VERIFIED: tools/flash/src/main.rs] |

### Supporting

| Library / Tool | Version | Purpose | When to Use |
| --- | --- | --- | --- |
| `anyhow` | `1.0.103` resolved | CLI and release-gate error propagation. | Keep using it in `tools/parity` and `tools/flash` CLI surfaces. [VERIFIED: cargo metadata] [VERIFIED: tools/parity/Cargo.toml] |
| `camino` | `1.2.3` | UTF-8 paths for CLI/document validation. | Continue for release document, manifest, and evidence paths. [VERIFIED: cargo metadata] [VERIFIED: tools/parity/Cargo.toml] |
| `clap` | `4.6.1` | CLI parsing for `tools/parity` and `tools/flash`. | Add any Phase 8 `tools/parity` subcommand or option through existing Clap patterns. [VERIFIED: cargo metadata] [VERIFIED: tools/parity/src/main.rs] |
| `serde` / `serde_json` | `1.0.228` / `1.0.150` | JSON serialization/deserialization for reports, manifests, and evidence records. | Use for machine-readable release evidence or manifest checks instead of ad hoc string handling. [VERIFIED: cargo metadata] [VERIFIED: tools/flash/src/main.rs] [VERIFIED: tools/xtask/src/package_manifest.rs] |
| `curl` | `8.7.1` | Live HTTP probes for `/`, `/assets/app.css.gz`, `/recovery`, `/api/system/OTA`, and `/api/system/OTAWWW`. | Use after `DEVICE_URL` is established; record commands, statuses, headers, bodies, and sanitized output. [VERIFIED: command availability] [VERIFIED: docs/release/ultra-205.md] |
| `cargo-about` | `0.9.0` | Cargo dependency license report input. | Preserve as a release-gate input; regenerate only when Cargo inputs change. [VERIFIED: command availability] [VERIFIED: docs/release/license-inventory.md] |
| `jq` | `1.7.1-apple` | Local manifest/report inspection in scripts or manual verification. | Use for evidence extraction when it keeps scripts clear; avoid making jq-only logic the release source of truth. [VERIFIED: command availability] |
| `python3` | `3.14.4` | Existing package script helper path. | Existing packaging uses Python for erased otadata fallback bytes. [VERIFIED: command availability] [VERIFIED: scripts/package-firmware.sh] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
| --- | --- | --- |
| Extend `tools/parity` | A new `tools/release-ready` binary | Rejected because Phase 8 locked decision D-09 says to extend the existing parity release-gate path, not create a second release-readiness tool. [VERIFIED: 08-CONTEXT.md] |
| Typed JSON/Markdown validation in Rust | Shell-only `grep` checks over checklist and release docs | Rejected for guard logic because existing `tools/parity` already parses checklist rows and release doc sections with tests. [VERIFIED: tools/parity/src/main.rs] [VERIFIED: tools/parity/src/release_gate.rs] |
| Existing Rust-owned static/recovery assets | Copy upstream AxeOS or recovery HTML to close evidence | Rejected because V1 avoids an Angular UI rewrite and provenance policy forbids copying GPL expression into MIT-only files without explicit review. [VERIFIED: docs/adr/0010-axeos-api-and-asset-compatibility.md] [VERIFIED: PROVENANCE.md] |
| Keep OTAWWW as explicit REL-03 gap | Implement whole-`www` partition update now | Default recommendation is to keep the gap unless a Phase 8 plan explicitly adds size, erase/write, recovery, and interrupted-update hardware-regression evidence. [VERIFIED: 08-CONTEXT.md] [VERIFIED: docs/parity/evidence/phase-07-ota-filesystem-release.md] |

**Installation:**

```bash
# No new dependencies recommended for Phase 8.
# Use the existing repo toolchain and verify with:
just doctor
just detect-ultra205
```

**Version verification:** Versions above were verified with `cargo metadata`, `just --version`, `bazel --version`, `espflash --version`, `cargo-about --version`, `curl --version`, `jq --version`, and `python3 --version` during this research session. [VERIFIED: command availability]

## Architecture Patterns

### Recommended Project Structure

```text
docs/
  parity/
    checklist.md                         # Release audit ledger.
    evidence/
      phase-08-ultra-205-release-gate.md # Recommended Phase 8 rollup.
      phase-08-ultra-205-http-ota-recovery/ # Optional raw logs and JSON if needed.
  release/
    ultra-205.md                         # Operator guide, updated only for new evidence/gaps.
    license-inventory.md                 # Existing release-gate input.
    provenance-manifest.md               # Existing release-gate input.
tools/
  parity/
    src/
      main.rs                            # CLI and existing checklist parser; avoid major growth.
      release_gate.rs                    # Preferred home for new release-readiness guards.
      release_evidence.rs                # Add only if Phase 8 needs typed evidence parsing.
scripts/
  # Add a small helper only if live probe commands become too repetitive.
```

This structure keeps the checklist as the ledger, release-gate validation inside `tools/parity`, and raw live evidence under `docs/parity/evidence/`. [VERIFIED: docs/parity/checklist.md] [VERIFIED: tools/parity/src/main.rs] [VERIFIED: tools/parity/src/release_gate.rs]

### Pattern 1: Evidence-First Checklist Promotion

**What:** Promote a checklist row only after the evidence file contains the exact evidence class and concrete observations the row needs. [VERIFIED: docs/parity/checklist.md] [VERIFIED: tools/parity/src/main.rs]

**When to use:** Use for FS-001, OTA-001, OTA-002, REL-001, REL-002, REL-003, and any safety-critical or release-sensitive row. [VERIFIED: docs/parity/checklist.md] [VERIFIED: 08-CONTEXT.md]

**Example:**

```rust
// Source: tools/parity/src/main.rs
fn validate_release_ota_verified_row(row: &ChecklistRow) -> Vec<ValidationError> {
    match row.id.as_str() {
        "FS-001" => validate_filesystem_verified_row(row),
        "OTA-001" => validate_firmware_ota_verified_row(row),
        "OTA-002" => validate_otawww_verified_row(row),
        "REL-001" | "REL-002" => validate_release_sensitive_verified_row(row),
        "REL-003" => validate_release_image_verified_row(row),
        _ => Vec::new(),
    }
}
```

### Pattern 2: Pure Guard Logic, Thin CLI/Filesystem Shell

**What:** Keep release readiness decisions in pure functions that accept parsed document or manifest data; keep file reads and CLI parsing in the surrounding command code. [VERIFIED: standards/core/architecture.md] [VERIFIED: tools/parity/src/release_gate.rs]

**When to use:** Use when adding new requirements such as "manifest artifact review must no longer say awaiting package output evidence" or "phase-08 evidence must mention `DEVICE_URL`, valid OTA, invalid OTA, rollback, recovery, and interrupted-update conclusions before REL-003 can be verified." [VERIFIED: docs/release/provenance-manifest.md] [VERIFIED: 08-CONTEXT.md]

**Example:**

```rust
// Source: tools/parity/src/release_gate.rs
pub(crate) fn validate_release_gate(documents: &ReleaseGateDocuments) -> ReleaseGateReport {
    let license_sections = parse_h2_sections(&documents.license_inventory_markdown);
    let provenance_sections = parse_h2_sections(&documents.provenance_markdown);
    let mut errors = Vec::new();

    validate_required_sections(&mut errors, "license inventory", &documents.license_inventory_path, &license_sections, &[
        "Cargo crates",
        "Bazel and rules",
        "ESP-IDF and esp-rs",
        "Flashing tools",
        "Static assets",
        "Release artifacts",
    ]);

    ReleaseGateReport { errors }
}
```

### Pattern 3: Phase-Gated Hardware Evidence

**What:** Start live hardware work with `just detect-ultra205`, package current artifacts, flash/monitor with evidence capture, establish `DEVICE_URL`, then run HTTP/OTA probes and destructive recovery steps only from a documented plan. [VERIFIED: AGENTS.md] [VERIFIED: scripts/detect-ultra205.sh] [VERIFIED: docs/release/ultra-205.md]

**When to use:** Use for live `/`, `/assets/app.css.gz`, missing static redirect, `/recovery`, valid OTA, invalid OTA, OTAWWW gap, rollback/boot validation, large erase recovery, failed update recovery, and interrupted-update recovery. [VERIFIED: 08-CONTEXT.md]

**Example sequence:**

```bash
# Source: AGENTS.md, Justfile, docs/release/ultra-205.md
just detect-ultra205
just package
just flash-monitor board=205 port=/dev/cu.usbmodem1101 evidence-dir=docs/parity/evidence/phase-08-ultra-205-release-gate
# Establish DEVICE_URL from logs, operator-supplied address, or a Phase 8 network setup step before curl probes.
```

### Pattern 4: Explicit Gap Closure

**What:** It is valid for V1 to ship with explicit gaps only when checklist, release docs, evidence files, and release gate all show owner, impact, follow-up, and no overclaimed status. [VERIFIED: 08-CONTEXT.md] [VERIFIED: docs/parity/evidence/phase-07-ota-filesystem-release.md]

**When to use:** Use for OTAWWW unless Phase 8 implements and proves whole-`www` partition replacement with hardware-regression and interrupted-update recovery evidence. [VERIFIED: 08-CONTEXT.md]

**Example expected current OTAWWW response:**

```text
Wrong API input
```

Source: `crates/bitaxe-api/src/update_plan.rs`, `firmware/bitaxe/src/http_api.rs`, and `docs/release/ultra-205.md`. [VERIFIED: codebase rg]

### Anti-Patterns to Avoid

- **Release by implementation status:** Package, compile, unit, golden, or API-compare evidence alone must not verify live OTA, rollback, recovery, or interrupted-update rows. [VERIFIED: docs/adr/0012-parity-verification-evidence.md] [VERIFIED: tools/parity/src/main.rs]
- **Parallel release-readiness tool:** Do not create a second readiness CLI; extend `tools/parity` guards and report paths. [VERIFIED: 08-CONTEXT.md]
- **Unchecked destructive tests:** Do not run erase, rollback, or interruption tests until the plan records recovery path, current package manifest, factory image, exact commands, and evidence fields. [VERIFIED: AGENTS.md] [VERIFIED: docs/release/ultra-205.md]
- **Scope inheritance:** Do not let Ultra 205 evidence verify Gamma 601, BM1370, Stratum v2, BAP, all-board images, or UI replacement rows. [VERIFIED: docs/adr/0014-pivot-to-ultra-205-bm1366-first-parity.md] [VERIFIED: 08-CONTEXT.md]
- **Line-by-line translation comments:** Add missing breadcrumbs at module or behavior boundaries only. [VERIFIED: docs/adr/0008-reference-breadcrumb-comments.md]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
| --- | --- | --- | --- |
| Checklist parsing and verified-row policy | A second Markdown parser or prose-only review checklist | Existing `tools/parity` row parser and validators | It already parses seven-column rows and fails invalid verified claims. [VERIFIED: tools/parity/src/main.rs] |
| Release readiness | A standalone release-readiness binary | `tools/parity release-gate` plus `just parity` | Phase 8 locked decision D-09 makes this canonical. [VERIFIED: 08-CONTEXT.md] |
| Manifest validation | Manual JSON grep in shell | `tools/xtask/src/package_manifest.rs` and `serde_json`-backed checks | Manifest v2 already validates required fields, artifact kinds, offsets, and SHA-256 shape. [VERIFIED: tools/xtask/src/package_manifest.rs] |
| USB detection and board identity | Manual port guesses | `just detect-ultra205` | It enforces exactly one likely ESP port and successful `espflash board-info`. [VERIFIED: scripts/detect-ultra205.sh] |
| Flash evidence | Manually pasted flash logs only | `just flash-monitor ... evidence-dir=...` plus a Phase 8 evidence doc | `tools/flash` already writes monitor logs and `flash-command-evidence.json`. [VERIFIED: tools/flash/src/main.rs] |
| Firmware OTA mechanics | Custom flash/partition writers | ESP-IDF OTA APIs through existing adapter | `firmware/bitaxe/src/ota_update.rs` already streams request bytes through ESP-IDF OTA begin/write/end/set-boot calls. [VERIFIED: firmware/bitaxe/src/ota_update.rs] [CITED: https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/ota.html] |
| Static path resolution | Direct string concatenation to filesystem paths | `crates/bitaxe-api/src/static_plan.rs` | The pure resolver rejects traversal before filesystem access and models gzip/static/recovery decisions. [VERIFIED: crates/bitaxe-api/src/static_plan.rs] |
| Upstream UI/recovery content | Copy upstream Angular or recovery HTML | Rust-owned fallback static/recovery assets plus provenance docs | Copying upstream expression risks GPL contamination and expands V1 scope. [VERIFIED: PROVENANCE.md] [VERIFIED: docs/release/provenance-manifest.md] |

**Key insight:** The hard problem is not adding more code; it is proving that each release claim has the right class of evidence and that gaps remain visible instead of becoming accidental claims. [VERIFIED: docs/adr/0006-parity-checklist-as-audit-evidence.md] [VERIFIED: docs/adr/0012-parity-verification-evidence.md]

## Common Pitfalls

### Pitfall 1: Treating Route Registration as HTTP Reachability

**What goes wrong:** A plan promotes FS-001 or OTA rows because serial logs show HTTP routes registered. [VERIFIED: docs/parity/evidence/phase-07-ultra-205-ota-hardware-smoke.md]

**Why it happens:** Phase 7 reached route registration but did not expose a reachable `DEVICE_URL`. [VERIFIED: docs/parity/evidence/phase-07-ultra-205-ota-hardware-smoke.md]

**How to avoid:** Make `DEVICE_URL` discovery the first live-evidence task and record "not run - no reachable device URL" if it cannot be established. [VERIFIED: 08-CONTEXT.md]

**Warning signs:** Evidence says route registered, but has no HTTP status codes, response headers, response bodies, device URL, or `curl` commands. [VERIFIED: docs/release/ultra-205.md]

### Pitfall 2: Verifying OTA from Success Response Alone

**What goes wrong:** OTA-001 or REL-08 is treated as verified after `/api/system/OTA` returns `Firmware update complete, rebooting now!`. [VERIFIED: docs/release/ultra-205.md]

**Why it happens:** Upload success does not prove selected next partition, reboot identity, boot validation, rollback, invalid-image rejection, or operable recovery. [VERIFIED: docs/release/ultra-205.md] [CITED: https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/ota.html]

**How to avoid:** Capture upload response, reboot logs, running partition, `ota_boot_validation` lines, invalid-image rejection, and recovery outcome before promotion. [VERIFIED: docs/release/ultra-205.md] [VERIFIED: firmware/bitaxe/src/boot_validation.rs]

**Warning signs:** Evidence lacks `ota_boot_validation=marked_valid`, `marked_invalid_reboot`, running partition, or post-reboot identity. [VERIFIED: firmware/bitaxe/src/boot_validation.rs] [VERIFIED: docs/parity/evidence/phase-07-ultra-205-ota-hardware-smoke.md]

### Pitfall 3: Accidentally Expanding V1

**What goes wrong:** Planning drifts into Gamma 601/BM1370, additional ASICs, Stratum v2, BAP, all-board release matrices, or Angular UI replacement. [VERIFIED: 08-CONTEXT.md]

**Why it happens:** The checklist contains future surfaces, but Phase 8 is a release gate for Ultra 205 V1 evidence. [VERIFIED: docs/parity/checklist.md] [VERIFIED: .planning/ROADMAP.md]

**How to avoid:** Keep future surfaces deferred/not-started unless a later roadmap phase adds an evidence path; do not touch upstream reference files. [VERIFIED: docs/adr/0014-pivot-to-ultra-205-bm1366-first-parity.md] [VERIFIED: docs/adr/0005-read-only-reference-implementation.md]

**Warning signs:** A plan proposes non-205 hardware evidence, Stratum v2 implementation, BAP parity, Angular assets, or all-board factory artifacts. [VERIFIED: 08-CONTEXT.md]

### Pitfall 4: Destructive Evidence Without Recovery Proof

**What goes wrong:** An erase, interrupted update, rollback fault, or raw write leaves the board unrecoverable or produces evidence that cannot be safely repeated. [VERIFIED: AGENTS.md] [VERIFIED: docs/release/ultra-205.md]

**Why it happens:** The recovery path depends on a current package manifest, factory image, port, exact command, and known-good flash/monitor procedure. [VERIFIED: docs/release/ultra-205.md]

**How to avoid:** Require package manifest path, factory image checksum, board-info, exact command, and post-recovery boot/static/recovery observations before destructive steps. [VERIFIED: AGENTS.md] [VERIFIED: docs/release/ultra-205.md]

**Warning signs:** A task says "erase flash" or "interrupt upload" but does not name `bitaxe-ultra205-factory.bin`, `just flash board=205 port=...`, and expected evidence fields. [VERIFIED: docs/release/ultra-205.md]

### Pitfall 5: Checklist Drift

**What goes wrong:** Evidence docs are updated but the checklist status/evidence/notes stay stale, or rows are promoted without tool coverage. [VERIFIED: docs/adr/0006-parity-checklist-as-audit-evidence.md]

**Why it happens:** The checklist is the ledger; release docs and evidence docs are supporting artifacts. [VERIFIED: docs/adr/0006-parity-checklist-as-audit-evidence.md]

**How to avoid:** Run `just parity` after each checklist or evidence change and add tests for new guard logic. [VERIFIED: Justfile] [VERIFIED: tools/parity/src/main.rs]

**Warning signs:** `docs/parity/evidence/*` says passed but checklist still says pending, or checklist says verified but `tools/parity` has no guard for the release-sensitive condition. [VERIFIED: docs/parity/checklist.md] [VERIFIED: tools/parity/src/main.rs]

## Code Examples

### Verified-Row Guard Pattern

```rust
// Source: tools/parity/src/main.rs
fn validate_rows(rows: &[ChecklistRow]) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    for row in rows {
        if normalize(&row.status) != "verified" {
            continue;
        }

        if normalize(&row.evidence) == "pending" {
            errors.push(ValidationError {
                id: row.id.clone(),
                message: "verified rows require non-pending evidence".to_owned(),
            });
        }

        if is_safety_critical(row) && !has_hardware_evidence(row) {
            errors.push(ValidationError {
                id: row.id.clone(),
                message: "safety-critical verified rows require hardware-smoke or hardware-regression evidence".to_owned(),
            });
        }

        errors.extend(validate_release_ota_verified_row(row));
    }

    errors
}
```

### Release-Gate Test Pattern

```rust
// Source: tools/parity/src/release_gate.rs
#[test]
fn release_gate_fails_when_unknown_lacks_owner_and_follow_up() {
    // Arrange
    let mut documents = complete_documents();
    documents.license_inventory_markdown =
        LICENSE_INVENTORY.replacen("- Owner: release tooling", "- Status: unknown", 1);

    // Act
    let report = validate_release_gate(&documents);

    // Assert
    assert!(!report.passed());
    assert!(report.errors.iter().any(|error| {
        error.contains("unknown") && error.contains("owner") && error.contains("follow-up")
    }));
}
```

### Static/Recovery Pure Decision Pattern

```rust
// Source: crates/bitaxe-api/src/static_plan.rs
if request.path == "/recovery" {
    if request.filesystem == FilesystemAvailability::Unavailable {
        return StaticRouteDecision::RecoveryFallback(recovery_fallback());
    }

    return StaticRouteDecision::ServeRecovery(serve_recovery());
}
```

### Firmware OTA Adapter Boundary

```rust
// Source: firmware/bitaxe/src/http_api.rs
let result = crate::ota_update::stream_firmware_ota(raw_request, record_firmware_ota_status);
match result {
    FirmwareOtaApplyResult::Complete { bytes_written } => {
        log::info!("firmware_ota_update=complete bytes_written={bytes_written}");
        send_public_response(request, plan.success_response)?;
        schedule_firmware_ota_restart();
        Ok(())
    }
    FirmwareOtaApplyResult::ValidationError { esp_err } => {
        log::warn!("firmware_ota_update=validation_error esp_err={esp_err}");
        send_public_response(request, plan.validation_error_response)
    }
    _ => send_text_error(request, 500, "Protocol Error"),
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
| --- | --- | --- | --- |
| Treat implementation completion as parity. | Treat `verified` as explicit evidence-backed parity. | Established in ADR-0006 and ADR-0012; reinforced by Phase 8 context. [VERIFIED: docs/adr/0006-parity-checklist-as-audit-evidence.md] [VERIFIED: docs/adr/0012-parity-verification-evidence.md] | Plans must include evidence capture and `tools/parity` guards, not only code tasks. |
| Package-only release confidence. | Release readiness derives from package manifest, license inventory, provenance manifest, operator guide, checklist, and hardware evidence. | Phase 7 and Phase 8 decisions. [VERIFIED: 07-CONTEXT.md] [VERIFIED: 08-CONTEXT.md] | REL rows stay below verified until live release behavior and review records exist. |
| Broad first-target hardware scope. | Ultra 205/BM1366 first, with non-205 surfaces deferred until each has its own evidence set. | ADR-0014. [VERIFIED: docs/adr/0014-pivot-to-ultra-205-bm1366-first-parity.md] | Do not reuse Ultra 205 evidence for other boards or ASICs. |
| Copy upstream UI/recovery assets to match appearance. | Rust-owned fallback/static/recovery assets with explicit provenance; no Angular rewrite in V1. | Phase 7 release/provenance decisions. [VERIFIED: docs/release/provenance-manifest.md] [VERIFIED: docs/adr/0010-axeos-api-and-asset-compatibility.md] | Release evidence should inspect behavior and provenance, not visual UI replacement. |
| Ad hoc destructive testing. | Phase-gated destructive/fault-injection procedures with recovery paths and evidence fields. | Repo-local Ultra 205 hardware guidance and Phase 8 context. [VERIFIED: AGENTS.md] [VERIFIED: 08-CONTEXT.md] | Plans must include recovery commands before erase/interruption tasks. |

**Deprecated/outdated:**

- `cargo-espmonitor` is not the normal workflow here; use `espflash monitor` through `tools/flash` and `just monitor`/`just flash-monitor`. [VERIFIED: AGENTS.md] [VERIFIED: Justfile] [VERIFIED: tools/flash/src/main.rs]
- ESP-IDF 6 is not the first baseline; the project pins ESP-IDF `v5.5.4` through `esp-idf-sys` metadata for the current Rust firmware stack. [VERIFIED: AGENTS.md] [VERIFIED: Cargo.toml]
- Full OTAWWW parity is not the default Phase 8 recommendation; the current route is an explicit REL-03 gap unless whole-partition update and interrupted recovery are implemented and proven. [VERIFIED: 08-CONTEXT.md] [VERIFIED: crates/bitaxe-api/src/update_plan.rs]

## Assumptions Log

> List all claims tagged `[ASSUMED]` in this research. The planner and discuss-phase use this section to identify decisions that need user confirmation before execution.

| # | Claim | Section | Risk if Wrong |
| --- | --- | --- | --- |

All claims in this research were verified from local project files, local commands, or cited official documentation; no `[ASSUMED]` claims were intentionally used.

## Open Questions

1. **How will Phase 8 establish `DEVICE_URL`?**
   - What we know: Phase 7 serial evidence registered HTTP routes but found no reachable URL because the firmware did not expose Wi-Fi/AP/DHCP/mDNS/IP logs. [VERIFIED: docs/parity/evidence/phase-07-ultra-205-ota-hardware-smoke.md]
   - What's unclear: Whether Phase 8 should add a minimal network setup/logging step, use a known operator-supplied address, or record HTTP evidence pending. [VERIFIED: 08-CONTEXT.md]
   - Recommendation: Plan `DEVICE_URL` discovery as Wave 1 before any HTTP/OTA row promotion. [VERIFIED: docs/release/ultra-205.md]

2. **Will Phase 8 keep OTAWWW as a gap or implement full static update?**
   - What we know: Current `OTAWWW` returns `Wrong API input` and is documented as REL-03 gap. [VERIFIED: crates/bitaxe-api/src/update_plan.rs] [VERIFIED: docs/release/ultra-205.md]
   - What's unclear: Whether full whole-`www` partition update can be implemented and proven safely in this phase without violating no-expansion intent. [VERIFIED: 08-CONTEXT.md]
   - Recommendation: Keep OTAWWW as explicit gap and capture the gap response unless the planner creates a dedicated recovery-safe implementation plan with hardware-regression evidence. [VERIFIED: 08-CONTEXT.md]

3. **What exact destructive/fault-injection procedure should be used for interrupted update and large erase?**
   - What we know: Operator docs list required evidence fields and recovery sequence. [VERIFIED: docs/release/ultra-205.md]
   - What's unclear: Exact interruption point, command sequence, and acceptable timeout/recovery thresholds are not yet encoded as a runnable repo command. [VERIFIED: codebase rg]
   - Recommendation: Plan destructive checks only after packaging and non-destructive HTTP/OTA smoke pass, and record "not run" if recovery cannot be made explicit. [VERIFIED: AGENTS.md] [VERIFIED: 08-CONTEXT.md]

4. **Should release artifact review rows in `docs/release/provenance-manifest.md` be closed in Phase 8?**
   - What we know: Current artifact review table still says "Awaiting package output evidence" for generated release artifacts. [VERIFIED: docs/release/provenance-manifest.md]
   - What's unclear: Whether Phase 8 will update the table with concrete package manifest values or add a separate release summary that cites them. [VERIFIED: 08-CONTEXT.md]
   - Recommendation: Update provenance/license docs only where Phase 8 produces concrete manifest/checksum/review evidence, and add a factual release summary if it reduces checklist ambiguity. [VERIFIED: 08-CONTEXT.md]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
| --- | --- | --- | --- | --- |
| `cargo` | Rust tests and local metadata | yes | `1.88.0-nightly` | none needed. [VERIFIED: command availability] |
| `rustc` | Rust builds/tests | yes | `1.88.0-nightly` | none needed. [VERIFIED: command availability] |
| `just` | Human command surface | yes | `1.48.0` | Direct Bazel commands exist but should not be the default UX. [VERIFIED: command availability] [VERIFIED: Justfile] |
| `bazel` | Build/test/package/parity graph | yes | `9.1.1` | none needed. [VERIFIED: command availability] |
| `espflash` | Board detection, flash, monitor | yes | `4.0.1` | none for hardware evidence. [VERIFIED: command availability] |
| `cargo-about` | Release license input | yes | `0.9.0` | Regenerate only when Cargo inputs change. [VERIFIED: command availability] [VERIFIED: docs/release/license-inventory.md] |
| `curl` | Live HTTP/OTA probes | yes | `8.7.1` | Use another HTTP client only if curl fails locally; keep commands recorded. [VERIFIED: command availability] |
| `jq` | Local JSON extraction | yes | `1.7.1-apple` | Use Rust validators for release gate decisions. [VERIFIED: command availability] |
| `python3` | Existing package script helper | yes | `3.14.4` | Existing script already uses Python. [VERIFIED: command availability] [VERIFIED: scripts/package-firmware.sh] |
| Ultra 205 over USB | Live hardware evidence | yes | `/dev/cu.usbmodem1101`, ESP32-S3 rev v0.2, 16 MB flash | Stop if a later run finds zero or multiple ports. [VERIFIED: just detect-ultra205] |
| Reachable `DEVICE_URL` | HTTP/static/OTA/recovery probes | not yet established | - | First Phase 8 plan must establish URL or record evidence pending. [VERIFIED: docs/parity/evidence/phase-07-ultra-205-ota-hardware-smoke.md] |

**Missing dependencies with no fallback:**

- Reachable `DEVICE_URL` for live HTTP/OTA/recovery evidence is not yet established; without it, FS-001, OTA-001, REL-001, REL-002, REL-003, and REL-08 live evidence must remain pending. [VERIFIED: docs/parity/evidence/phase-07-ultra-205-ota-hardware-smoke.md] [VERIFIED: 08-CONTEXT.md]

**Missing dependencies with fallback:**

- None for local CLI tooling; all required local commands checked during research are available. [VERIFIED: command availability]

## Validation Architecture

### Test Framework

| Property | Value |
| --- | --- |
| Framework | Bazel `rust_test` over Rust unit tests; Cargo tests for focused crate-level checks. [VERIFIED: tools/parity/BUILD.bazel] [VERIFIED: Cargo.toml] |
| Config file | `tools/parity/BUILD.bazel`, root `Cargo.toml`, and package `Cargo.toml` files. [VERIFIED: codebase rg] |
| Quick run command | `bazel test //tools/parity:tests` for release/checklist guard changes. [VERIFIED: command run] |
| Full suite command | `just test`; release gate add-ons should also run `just package`, `just parity`, and `bazel run //tools/parity:report -- release-gate`. [VERIFIED: Justfile] [VERIFIED: command run] |

### Phase Requirements -> Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
| --- | --- | --- | --- | --- |
| REL-08 | Rollback/recovery/large erase/failed update/interrupted update cannot be claimed without live evidence. | unit + hardware smoke/regression | `bazel test //tools/parity:tests --test_filter=release` after adding guards; hardware run is phase-gated manual/agent evidence. | Partial: `tools/parity/src/main.rs`, `tools/parity/src/release_gate.rs` exist. [VERIFIED: codebase rg] |
| EVD-01 | Checklist rows retain seven required fields. | unit + report command | `just parity` | yes. [VERIFIED: command run] |
| EVD-02 | Invalid `verified` rows fail validation. | unit + report command | `bazel test //tools/parity:tests`; `just parity` | yes. [VERIFIED: command run] |
| EVD-03 | Non-205/deferred rows remain unverified/deferred. | checklist audit + parity guard if added | `just parity`; optional new `tools/parity` test for non-205 overclaim rows | Partial: checklist exists; specific non-205 overclaim guard may need Wave 0 test. [VERIFIED: docs/parity/checklist.md] |
| EVD-04 | Breadcrumbs exist at module/behavior boundaries. | grep/audit + targeted tests where local modules already test breadcrumbs | `rg -n "Reference breadcrumb|Reference breadcrumbs|reference/esp-miner" crates firmware tools` | yes for audit; only some module-specific tests exist. [VERIFIED: codebase rg] |
| EVD-05 | Verification layers are represented and not overclaimed. | checklist audit + release gate + hardware evidence review | `just parity`; `bazel run //tools/parity:report -- release-gate`; phase-gated hardware evidence commands | yes for existing guards; Phase 8 hardware evidence file does not exist yet. [VERIFIED: command run] |

### Sampling Rate

- **Per task commit:** Run the narrow affected command: usually `bazel test //tools/parity:tests` for guard logic, `cargo test -p bitaxe-api --all-features update_plan static_plan` for pure route decision changes, or `just parity` for checklist-only changes. [VERIFIED: command run] [VERIFIED: Cargo.toml]
- **Per wave merge:** Run `just parity`, `bazel run //tools/parity:report -- release-gate`, and all affected crate tests. [VERIFIED: Justfile] [VERIFIED: command run]
- **Phase gate:** Run `just test`, `just package`, `just parity`, `bazel run //tools/parity:report -- release-gate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` if manifest exists, and the Phase 8 hardware evidence protocol when `DEVICE_URL` and recovery path are established. [VERIFIED: Justfile] [VERIFIED: tools/parity/src/main.rs] [VERIFIED: docs/release/ultra-205.md]

### Wave 0 Gaps

- [ ] `tools/parity/src/release_gate.rs` or a small sibling module - add focused tests for any new Phase 8 release-readiness guards before checklist promotion. [VERIFIED: tools/parity/src/release_gate.rs]
- [ ] `docs/parity/evidence/phase-08-ultra-205-release-gate.md` - create the Phase 8 evidence ledger before live hardware actions so every run has a destination and expected fields. [VERIFIED: 08-CONTEXT.md]
- [ ] `DEVICE_URL` discovery procedure - document how the URL is obtained, or record the blocker before HTTP/OTA probes. [VERIFIED: docs/parity/evidence/phase-07-ultra-205-ota-hardware-smoke.md]

## Security Domain

Security enforcement is enabled because `.planning/config.json` does not set `security_enforcement` to `false`. [VERIFIED: .planning/config.json]

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
| --- | --- | --- |
| Authentication / access control for update routes | yes | Preserve `plan_http_access`, private-network/origin checks, and AP-mode update rejection before OTA effects. [VERIFIED: crates/bitaxe-api/src/update_plan.rs] [VERIFIED: firmware/bitaxe/src/http_api.rs] |
| Session management | limited | No browser login/session state is introduced in Phase 8; do not add session state for evidence tooling. [VERIFIED: codebase rg] |
| Input validation | yes | Keep upload route planning typed, reject unsafe static paths before filesystem access, and validate manifests with structured JSON. [VERIFIED: crates/bitaxe-api/src/static_plan.rs] [VERIFIED: tools/xtask/src/package_manifest.rs] |
| Cryptography / integrity | yes | Use SHA-256 checksums already present in package manifest and evidence docs; do not hand-roll cryptography. [VERIFIED: tools/xtask/src/package_manifest.rs] |
| Secure update / supply chain | yes | Tie artifacts to source commit, reference commit, tool versions, checksums, license inventory, and provenance manifest. [VERIFIED: docs/release/provenance-manifest.md] [VERIFIED: tools/xtask/src/package_manifest.rs] |
| Logging and privacy | yes | Hardware evidence must exclude secrets, Wi-Fi credentials, pool credentials, private endpoints, and NVS secret values. [VERIFIED: AGENTS.md] |

OWASP ASVS should be used as the application security vocabulary if detailed controls are added, but the Phase 8 actionable controls are the repo-local release/update/evidence controls above. [CITED: https://owasp.org/www-project-application-security-verification-standard/] [VERIFIED: AGENTS.md]

### Known Threat Patterns for Ultra 205 Release Gate

| Pattern | STRIDE | Standard Mitigation |
| --- | --- | --- |
| Unauthorized firmware update request | Elevation of privilege / tampering | Preserve private-network/origin/AP-mode gates before OTA effects. [VERIFIED: crates/bitaxe-api/src/update_plan.rs] |
| Static path traversal | Tampering / information disclosure | Use `resolve_static_request` path rejection before filesystem access. [VERIFIED: crates/bitaxe-api/src/static_plan.rs] |
| Corrupt or interrupted update bricks device | Denial of service | Require rollback/boot-validation evidence, recovery procedure, package manifest, and factory image before claims. [VERIFIED: docs/release/ultra-205.md] [CITED: https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/ota.html] |
| Evidence leaks credentials or private endpoints | Information disclosure | Sanitize evidence and do not commit secrets, pool credentials, Wi-Fi credentials, private endpoints, or NVS secret values. [VERIFIED: AGENTS.md] |
| Supply-chain/provenance ambiguity | Tampering / repudiation | Release gate must include manifest checksums, source/reference commits, cargo-about, license inventory, and provenance manifest. [VERIFIED: tools/parity/src/release_gate.rs] [VERIFIED: docs/release/provenance-manifest.md] |
| Non-205 evidence reuse | Repudiation / safety | Keep non-205 rows deferred or unverified until each board/ASIC has its own evidence. [VERIFIED: docs/adr/0014-pivot-to-ultra-205-bm1366-first-parity.md] |

## Sources

### Primary (HIGH confidence)

- `.planning/phases/08-parity-evidence-and-ultra-205-release-gate/08-CONTEXT.md` - locked Phase 8 decisions, no-expansion boundary, release-gate policy, hardware workflow, and deferred scope. [VERIFIED: file read]
- `.planning/REQUIREMENTS.md` - REL-08 and EVD-01 through EVD-05 requirement definitions. [VERIFIED: file read]
- `.planning/STATE.md` - current phase state, prior decisions, and Phase 8 blockers. [VERIFIED: file read]
- `.planning/ROADMAP.md` - Phase 8 goal, success criteria, verification expectations, and research flags. [VERIFIED: codebase rg]
- `AGENTS.md`, `AGENTS.bright-builds.md`, `standards/*`, `tasks/lessons.md` - repo instructions, Bright Builds rules, Ultra 205 hardware workflow, Rust verification rules, and Markdown frontmatter lesson. [VERIFIED: file read]
- `docs/parity/checklist.md` - parity ledger, status/evidence definitions, rows, and current release/OTA status. [VERIFIED: file read]
- `tools/parity/src/main.rs`, `tools/parity/src/release_gate.rs`, `tools/parity/BUILD.bazel` - existing checklist parser, verified-row guards, release-gate validation, and tests. [VERIFIED: file read]
- `docs/parity/evidence/phase-07-ota-filesystem-release.md`, `docs/parity/evidence/phase-07-ultra-205-ota-hardware-smoke.md` - deferred live release surfaces and current hardware evidence gap. [VERIFIED: file read]
- `docs/release/ultra-205.md`, `docs/release/license-inventory.md`, `docs/release/provenance-manifest.md`, `PROVENANCE.md` - operator procedures, release-gate inputs, artifact review, and GPL guardrails. [VERIFIED: file read]
- `scripts/detect-ultra205.sh`, `tools/flash/src/main.rs`, `tools/xtask/src/package_manifest.rs`, `firmware/bitaxe/src/http_api.rs`, `firmware/bitaxe/src/ota_update.rs`, `firmware/bitaxe/src/boot_validation.rs`, `crates/bitaxe-api/src/update_plan.rs`, `crates/bitaxe-api/src/static_plan.rs` - implementation and evidence integration points. [VERIFIED: file read]
- Local command outputs: `just detect-ultra205`, `just parity`, `bazel test //tools/parity:tests`, `bazel run //tools/parity:report -- release-gate`, `cargo metadata`, and CLI version probes. [VERIFIED: command run]

### Secondary (MEDIUM confidence)

- Espressif ESP-IDF OTA API documentation - rollback and OTA API semantics for evidence expectations. [CITED: https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/ota.html]
- Espressif ESP-IDF HTTP server documentation - HTTP server handler context for route-serving behavior. [CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-reference/protocols/esp_http_server.html]
- esp-rs `espflash` project documentation - command family context for flashing/monitoring workflows; local version and usage were verified from repo scripts and command output. [CITED: https://github.com/esp-rs/espflash]
- OWASP ASVS project page - security vocabulary reference; actionable controls come from repo-local firmware/update constraints. [CITED: https://owasp.org/www-project-application-security-verification-standard/]

### Tertiary (LOW confidence)

- None used as authoritative support.

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH - all recommended tools and crate versions are existing repo dependencies or locally installed CLIs verified in this session. [VERIFIED: cargo metadata] [VERIFIED: command availability]
- Architecture: HIGH - patterns come from existing `tools/parity`, `tools/flash`, `tools/xtask`, firmware adapters, ADRs, and Bright Builds architecture rules. [VERIFIED: codebase rg] [VERIFIED: standards/core/architecture.md]
- Pitfalls: HIGH - pitfalls are directly documented by Phase 7 deferred evidence, Phase 8 locked decisions, checklist policy, and release operator docs. [VERIFIED: docs/parity/evidence/phase-07-ultra-205-ota-hardware-smoke.md] [VERIFIED: 08-CONTEXT.md]
- Hardware availability: MEDIUM - USB board detection passed during research, but HTTP reachability is not established until a Phase 8 run finds or configures `DEVICE_URL`. [VERIFIED: just detect-ultra205] [VERIFIED: docs/parity/evidence/phase-07-ultra-205-ota-hardware-smoke.md]

**Research date:** 2026-06-28
**Valid until:** 2026-07-12 for repo-local planning assumptions; re-run command availability and `just detect-ultra205` before live hardware execution.
