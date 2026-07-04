# Phase 23: Redacted Operator Evidence Workflow - Research

**Researched:** 2026-07-04  
**Domain:** Ultra 205 detector-gated production-mining evidence workflow, redaction, and parity evidence governance  
**Confidence:** HIGH for repo-local workflow, redaction, and validator patterns; MEDIUM for live hardware outcome planning because Phase 23 may truthfully leave later-share slots blocked or pending.

<user_constraints>
## User Constraints (from CONTEXT.md)

The following locked decisions, discretion areas, and deferred ideas are copied from `.planning/phases/23-redacted-operator-evidence-workflow/23-CONTEXT.md`. [VERIFIED: 23-CONTEXT.md]

### Locked Decisions

### Operator Evidence Flow

- **D-01:** Provide one documented repo-owned flow for board `205` that covers detection, package or flash, local credential input, bounded mining evidence, telemetry capture, safe-stop capture, redaction, redaction review, and evidence review.
- **D-02:** The flow must start with `just detect-ultra205` and stop when detection is absent, ambiguous, not board `205`, or board-info fails. No stale port, stale `DEVICE_URL`, network scan, mDNS, ARP, router, or unrelated evidence may substitute for the detector gate.
- **D-03:** Prefer a `just`-reachable command surface or repo-owned script wrapper over ad hoc instructions. If new hardware command shapes are needed, extend the repo validator and tests instead of bypassing allow-manifest style checks.
- **D-04:** The operator flow may accept local `wifi-credentials.json` and `pool-credentials*.json` files as runtime inputs only. The committed command summaries must record only category labels such as `pool_config: local-owner-supplied`, not the raw file contents or owner-identifying values.

### Single Redacted Evidence Root

- **D-05:** Phase 23 must define one v1.1 redacted evidence root shape for committed artifacts. Required slots are package, detector, board-info, command, log, API, WebSocket, share-outcome, redaction-review, safe-stop, and conclusion artifacts.
- **D-06:** Artifact slots that cannot be truthfully populated yet must still be represented with an explicit blocked, pending, or deferred outcome and the exact reason. In particular, share-outcome proof can remain a non-claim until Phase 25 observes a real accepted or rejected pool response to live ASIC-derived work.
- **D-07:** The evidence root must name board `205`, source commit, reference commit, package or firmware identity, detector evidence, command category, redaction status, observed behavior, safe-stop status, conclusion, and exact non-claims.
- **D-08:** Raw local artifacts may exist only in ignored/local paths when needed for diagnosis. The committed evidence root must contain redacted files or category summaries only and must declare `raw_artifacts_committed: no`.

### Redaction Contract

- **D-09:** Redaction must be designed as a first-class deliverable, not as a final manual cleanup step. Every new committed artifact type needs either automatic sanitization, an explicit review checklist, or both.
- **D-10:** Committed logs, command summaries, API captures, WebSocket captures, retained logs, NVS/settings values, Stratum fields, share payloads, safe-stop summaries, and conclusions must redact pool URLs, ports, users, workers, owner addresses, passwords, tokens, targets, extranonces, share payloads, socket errors, device URLs, IP addresses, MAC addresses, Wi-Fi values, NVS secrets, API tokens, and raw BM1366 frames.
- **D-11:** Add deterministic redaction tests or checks for the categories that existing Phase 17/21 tooling only partially covers, especially Stratum target values, extranonces, share payloads, socket errors, NVS/settings values, worker/address-like usernames, device URLs, IPs, MACs, Wi-Fi values, and pool credential fields.
- **D-12:** Operator-visible firmware and host logs should emit lifecycle/status markers with `redacted=true` or equivalent redaction-safe labels instead of raw Stratum messages, raw pool values, raw share payloads, raw socket errors, or raw ASIC frames.

### Reuse And Integration

- **D-13:** Reuse the strongest existing patterns from Phase 17 live API/WebSocket capture, Phase 21 mining evidence scripts and redaction tests, Phase 22 claim-ladder and blocker governance, `tools/flash` commit-redacted evidence, and `tools/parity` allow/evidence validators.
- **D-14:** The preferred implementation shape is a thin script or CLI shell around typed redaction/evidence decisions where practical. Pure redaction classifiers, evidence-root inventory validation, and claim/outcome modeling should live in tested Rust, Node, or shell-testable helpers rather than buried in long inline command strings.
- **D-15:** Preserve Phase 22 exact-claim language. Phase 23 can prove the operator workflow and redaction root, but it must not promote trusted BM1366 production work, live Stratum socket success, or accepted/rejected share outcomes unless those later-phase artifacts exist.
- **D-16:** Update parity and release/operator docs only to the exact level supported by Phase 23 artifacts. Evidence-governance rows may advance when the redacted workflow and review pass; production mining behavior rows remain pending or non-claimed until later phases.

### Verification Gate

- **D-17:** Verification must include tests for changed redaction/evidence helpers, script behavior, parity/evidence-root validation, `just parity`, `just verify-reference`, and lifecycle validation for this phase.
- **D-18:** If hardware is used, it must follow the Ultra 205 detector gate and evidence rules. If local credentials are used, the run must never print, summarize, commit, or copy secret values into evidence.
- **D-19:** The final phase verification may pass with blocked or pending hardware outcome slots only if those slots are explicitly marked as later-phase non-claims and the workflow, redaction, and evidence-root contract are otherwise proven.

### Claude's Discretion

Claude may choose exact file names, command names, script boundaries, validator names, artifact suffixes, test fixture values, and whether to extend existing Phase 21 helpers or create Phase 23-specific wrappers. Those choices must preserve detector gating, runtime-only secrets, redaction-safe committed artifacts, exact non-claim governance, rerunnable scripts, repo-native verification, and functional-core/imperative-shell separation.

### Deferred Ideas (OUT OF SCOPE)

- Trusted BM1366 production initialization, pool-derived work dispatch, live result parsing, stale-work invalidation, and fail-closed ASIC production errors belong to Phase 24.
- Real Stratum v1 socket lifecycle, deterministic fake-pool production tests, live accepted/rejected share outcome, watchdog under bounded production load, and bounded safe-stop runtime proof belong to Phase 25.
- API/WebSocket/statistics/scoreboard promotion and final v1.1 parity closure belong to Phase 26.
- Full active voltage/fan/thermal fault-stimulus, self-test hardware closure, OTA/recovery destructive or fault-injection evidence, runtime display/input, BAP, Stratum v2, non-205 boards, other ASIC families, and unbounded stress mining remain future work.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| EVD-07 | Committed v1.1 evidence records one redacted root with package, detector, board-info, command, log, API, WebSocket, share-outcome, redaction-review, safe-stop, and conclusion artifacts. [VERIFIED: REQUIREMENTS.md] | Use one Phase 23 evidence-root contract plus validator/test coverage for required slots, including blocked or pending slot artifacts when later phases own the outcome. [VERIFIED: 23-CONTEXT.md] |
| STR-10 | Production-mining evidence redacts pool URLs, ports, workers, owner addresses, passwords, targets, extranonces, share payloads, and socket errors. [VERIFIED: REQUIREMENTS.md] | Extend deterministic synthetic redaction fixtures beyond Phase 17/21 regex coverage and require redaction review before citation. [VERIFIED: 23-CONTEXT.md; VERIFIED: scripts/phase21-live-mining-evidence-test.sh] |
| REL-09 | Operator can run a documented repo-owned flow for detect, package or flash, local credential input, bounded production mining, telemetry capture, safe stop, redaction, and evidence review. [VERIFIED: REQUIREMENTS.md] | Make the flow `just`-reachable or script-owned, detector-gated, rerunnable, and validated by repo-owned tests rather than ad hoc instructions. [VERIFIED: Justfile; VERIFIED: AGENTS.md; VERIFIED: 23-CONTEXT.md] |
| CFG-07 | Local pool credentials are runtime inputs only and committed evidence records category labels instead of raw values. [VERIFIED: REQUIREMENTS.md] | Reuse the Phase 21 JSON parser and pool input bridge pattern, but keep committed summaries to `pool_config: local-owner-supplied` and `raw_pool_values_committed=no`. [VERIFIED: scripts/phase21-pool-credentials-json.mjs; VERIFIED: scripts/phase21-pool-input-bridge.sh] |
| EVD-09 | Redaction tests or review gates cover retained logs, command summaries, API captures, WebSocket captures, NVS/settings values, Stratum fields, share payloads, device URLs, IPs, MACs, Wi-Fi values, and pool secrets before evidence is committed. [VERIFIED: REQUIREMENTS.md] | Add Phase 23-specific synthetic fixture tests and a redaction-review inventory that covers every artifact type in the root. [VERIFIED: 23-CONTEXT.md; VERIFIED: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/redaction-review.md] |
</phase_requirements>

## Summary

Phase 23 should be planned as an evidence workflow and redaction-governance phase, not as a live production mining implementation phase. The repo already has the required detector gate (`just detect-ultra205`), package/flash evidence capture, Phase 17 API/WebSocket capture helpers, Phase 21 mining evidence wrappers and secret-bridge tests, and Phase 22 exact-claim language. [VERIFIED: Justfile; VERIFIED: tools/flash/src/main.rs; VERIFIED: scripts/phase17-live-http-api-smoke.sh; VERIFIED: scripts/phase21-live-mining-evidence.sh; VERIFIED: docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/claim-ladder.md]

The planner should create one redacted evidence-root contract and one repo-owned operator path that can populate or truthfully block the required slots. The implementation should not promote accepted/rejected share outcomes, trusted BM1366 production work, live Stratum socket success, or full safe-stop runtime proof unless those later-phase artifacts already exist. [VERIFIED: 23-CONTEXT.md; VERIFIED: docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/claim-ladder.md]

**Primary recommendation:** Build a thin Phase 23 shell around existing Phase 17/21/flash helpers, and put evidence-root validation, redaction inventory checks, and claim/outcome slot modeling in tested repo-owned helpers, preferably `tools/parity` when the decision is pure and cross-artifact. [VERIFIED: 23-CONTEXT.md; VERIFIED: standards/core/architecture.md; VERIFIED: standards/languages/rust.md]

## Project Constraints

- No `.cursor/rules/` files were present in this workspace during research, so there are no additional Cursor rule directives to copy into the plan. [VERIFIED: Glob .cursor/rules/**/*]
- No `.cursor/skills/` or `.agents/skills/` project skill indexes were present in this workspace during research. [VERIFIED: Glob .cursor/skills/**/SKILL.md; VERIFIED: Glob .agents/skills/**/SKILL.md]
- Repo-local hardware runs must start with `just detect-ultra205`, require exactly one likely ESP32-S3 port and passing `espflash board-info`, and stop on absent, ambiguous, wrong-board, or failed board-info detection. [VERIFIED: AGENTS.md; VERIFIED: scripts/detect-ultra205.sh]
- Local `wifi-credentials.json` and `pool-credentials*.json` may be runtime inputs only; their contents must not be read, printed, summarized, committed, copied into evidence, or exposed. [VERIFIED: AGENTS.md]
- Committed or shareable evidence must redact pool endpoints, ports, users, workers, owner addresses, passwords, tokens, NVS secrets, raw Stratum payloads, raw share payloads, raw BM1366 frames, device URLs, IPs, MACs, Wi-Fi values, and API tokens. [VERIFIED: AGENTS.md; VERIFIED: 23-CONTEXT.md]
- GSD and other frontmatter-parsed Markdown must not use standalone body `---` separators after frontmatter. [VERIFIED: AGENTS.md]
- Functional core / imperative shell is the default architecture: pure evidence decisions and redaction classification should be tested outside shell/hardware adapters. [VERIFIED: standards/core/architecture.md; VERIFIED: AGENTS.bright-builds.md]
- Pure business logic must have focused unit tests with Arrange, Act, Assert structure. [VERIFIED: standards/core/testing.md; VERIFIED: standards/languages/rust.md]

## Standard Stack

### Core

| Library / Tool | Version | Purpose | Why Standard |
|----------------|---------|---------|--------------|
| Just | 1.48.0 local | Human command surface for `detect-ultra205`, `package`, `flash-monitor`, `parity`, and `verify-reference`. [VERIFIED: local `just --version`; VERIFIED: Justfile] | Repo decisions require `just` as the operator-facing command surface. [VERIFIED: AGENTS.md] |
| Bazel / Bazelisk | 9.1.1 | Canonical automation graph and test runner. [VERIFIED: .bazelversion; VERIFIED: local `bazel --version`] | Repo commands route build/test/package through Bazel. [VERIFIED: Justfile; VERIFIED: AGENTS.md] |
| `rules_rust` | 0.70.0 | Rust Bazel rules for pure crates and tools. [VERIFIED: MODULE.bazel] | Current repo already builds Rust tools through Bazel with `rust_binary` and `rust_test`. [VERIFIED: tools/parity/BUILD.bazel; VERIFIED: tools/flash/BUILD.bazel] |
| Rust workspace crates/tools | Edition 2021 | Pure evidence, parity, flash, API, safety, Stratum, and firmware logic. [VERIFIED: Cargo.toml] | Existing pure validators live in Rust tools and crates, and new cross-artifact validation should follow that pattern. [VERIFIED: tools/parity/src/main.rs; VERIFIED: tools/parity/src/mining_allow.rs] |
| Bash scripts under `scripts/` | repo-owned | Thin hardware and evidence orchestration shell. [VERIFIED: scripts/BUILD.bazel] | Existing detector, package, API, WebSocket, pool bridge, and mining evidence flows are shell/Node wrappers registered in Bazel. [VERIFIED: scripts/BUILD.bazel] |
| Node.js | 24.13.0 local | JSON helper and WebSocket capture runtime. [VERIFIED: local `node --version`] | Existing helpers parse pool credentials and capture WebSocket frames with Node. [VERIFIED: scripts/phase21-pool-credentials-json.mjs; VERIFIED: scripts/phase17-websocket-capture.mjs] |
| `espflash` | 4.0.1 local | USB board-info, flash, monitor, and evidence capture backend. [VERIFIED: local `espflash --version`; VERIFIED: scripts/detect-ultra205.sh; VERIFIED: tools/flash/src/main.rs] | Repo-local guidance prefers `espflash` for flashing, monitoring, and board-info. [VERIFIED: AGENTS.md] |

### Supporting

| Library / Tool | Version | Purpose | When to Use |
|----------------|---------|---------|-------------|
| `serde` / `serde_json` | `serde` 1.0.228 requested, 1.0.228 locked; `serde_json` 1.0.150 requested, locked through Cargo | Typed JSON parsing and artifact validation. [VERIFIED: Cargo.toml; VERIFIED: Cargo.lock] | Use for evidence-root manifests, command evidence JSON, and validator fixtures instead of ad hoc parsing in Rust. [VERIFIED: tools/parity/src/release_evidence.rs; VERIFIED: tools/flash/src/main.rs] |
| `clap` | 4.6.1 | CLI parsing for Rust tools. [VERIFIED: Cargo.toml; VERIFIED: Cargo.lock] | Use for new `tools/parity` subcommands or options if Phase 23 adds a validator. [VERIFIED: tools/parity/src/main.rs] |
| `camino` | 1.2.3 | UTF-8 path types in tools. [VERIFIED: Cargo.toml; VERIFIED: Cargo.lock] | Use for evidence-root path validation in Rust tools. [VERIFIED: tools/parity/src/release_evidence.rs; VERIFIED: tools/parity/src/mining_allow.rs] |
| `tempfile` | 3.27.0 | Temporary test/output files. [VERIFIED: Cargo.toml; VERIFIED: Cargo.lock] | Use in Rust tests or flash-related helpers that need local scratch space. [VERIFIED: tools/flash/src/main.rs] |
| `curl` | 8.7.1 local | HTTP API capture shell dependency. [VERIFIED: local `curl --version`; VERIFIED: scripts/phase17-live-http-api-smoke.sh] | Use only inside repo-owned capture scripts with redaction and explicit target gates. [VERIFIED: scripts/phase17-live-http-api-smoke.sh] |
| `ripgrep` | 15.1.0 local | Deterministic redaction scans. [VERIFIED: local `rg --version`] | Use in review gates to scan committed evidence roots for forbidden categories. [VERIFIED: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/redaction-review.md] |
| Python 3 | 3.14.4 local | Existing script-local JSON helpers. [VERIFIED: local `python3 --version`; VERIFIED: scripts/phase17-live-http-api-smoke.sh; VERIFIED: scripts/phase21-live-mining-package.sh] | Reuse only where existing scripts already do; prefer Rust/Node for new typed helpers when possible. [VERIFIED: standards/core/code-shape.md] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Repo-owned `just`/script wrapper | Ad hoc terminal procedure | Ad hoc commands cannot be validated by allow-manifest style checks and are not repeatable evidence. [VERIFIED: 23-CONTEXT.md; VERIFIED: .planning/research/PITFALLS.md] |
| `tools/parity` validator extension | Manual evidence review only | Manual review is still needed, but pure slot and redaction-inventory checks are testable and should not be hidden in prose. [VERIFIED: 23-CONTEXT.md; VERIFIED: standards/core/testing.md] |
| Phase 21 credential parser and bridge | New pool credential parser | A new parser risks inconsistent accepted fields and leaks; the existing helper already validates `poolURL`, `poolPort`, `poolUser`, and `poolPassword` without printing raw values on success. [VERIFIED: scripts/phase21-pool-credentials-json.mjs; VERIFIED: scripts/phase21-live-mining-evidence-test.sh] |
| Phase 17 WebSocket capture helper | Hand-written one-off `curl`/socket snippets | The existing helper enforces allowed paths, bounded duration, max frames, origin-only device URLs, and redaction. [VERIFIED: scripts/phase17-websocket-capture.mjs] |
| `tools/flash` commit-redacted evidence | Direct `espflash` in docs | The flash tool already models `redact-evidence`, evidence directories, trusted boot markers, and commit-ready metadata. [VERIFIED: tools/flash/src/main.rs] |

**Installation:** No new package installation is recommended for Phase 23. The planner should reuse existing workspace dependencies and add files to existing Bazel targets. [VERIFIED: Cargo.toml; VERIFIED: MODULE.bazel; VERIFIED: scripts/BUILD.bazel]

**Version verification:** Versions above were verified from repo manifests and local command output during research; no npm packages are part of the recommended stack. [VERIFIED: Cargo.toml; VERIFIED: MODULE.bazel; VERIFIED: .bazelversion; VERIFIED: local tool commands]

## Architecture Patterns

### Recommended Project Structure

```text
scripts/
  phase23-redacted-operator-evidence.sh        # thin operator workflow shell [RECOMMENDED: based on scripts/phase21-live-mining-evidence.sh]
  phase23-redacted-operator-evidence-test.sh   # synthetic secret/redaction and blocked-slot tests [RECOMMENDED: based on scripts/phase21-live-mining-evidence-test.sh]

tools/parity/src/
  operator_evidence.rs                         # pure evidence-root slot and redaction inventory validation if new Rust validation is needed [RECOMMENDED: based on tools/parity/src/mining_allow.rs]
  main.rs                                      # subcommand wiring if a validator subcommand is added [RECOMMENDED: based on tools/parity/src/main.rs]

docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/
  evidence-contract.md                         # root shape and slot contract [RECOMMENDED: based on phase-21 evidence-contract.md]
  package/                                     # package identity or blocked artifact [RECOMMENDED]
  detector/                                    # detector and board-info evidence or blocked artifact [RECOMMENDED]
  board-info/                                  # board-info summary, redacted for commit [RECOMMENDED]
  command/                                     # command category and allow validation summary [RECOMMENDED]
  log/                                         # redacted log evidence or blocked artifact [RECOMMENDED]
  api/                                         # redacted API capture or blocked artifact [RECOMMENDED]
  websocket/                                   # redacted WebSocket capture or blocked artifact [RECOMMENDED]
  share-outcome/                               # explicit non-claim, blocked, pending, accepted, or rejected status [RECOMMENDED]
  safe-stop/                                   # safe-stop summary or blocked artifact [RECOMMENDED]
  redaction-review.md                          # deterministic scan and reviewer checklist [RECOMMENDED]
  summary.md                                   # conclusion and exact claims/non-claims [RECOMMENDED]
```

### Pattern 1: Detector-Gated Operator Shell

**What:** A shell wrapper should begin with a detector evidence requirement and should record blocked artifacts rather than proceed when detector, board-info, package identity, or redaction gates are missing. [VERIFIED: AGENTS.md; VERIFIED: scripts/detect-ultra205.sh; VERIFIED: scripts/phase21-live-mining-evidence.sh]

**When to use:** Use this for any Phase 23 path that can touch hardware, consume local credentials, derive a device target, or cite operator evidence. [VERIFIED: AGENTS.md; VERIFIED: 23-CONTEXT.md]

**Planning guidance:** The shell should orchestrate detector, package/flash, optional pool bridge, API/WebSocket capture, slot inventory, redaction scan, and summary rendering; it should not own complex redaction classifiers or claim decisions. [VERIFIED: standards/core/architecture.md; VERIFIED: 23-CONTEXT.md]

### Pattern 2: One Evidence Root With Required Slots

**What:** Every required slot should have a file or directory with `passed`, `blocked`, `pending`, or `deferred` status plus a reason and claim boundary. [VERIFIED: 23-CONTEXT.md; VERIFIED: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/evidence-contract.md]

**When to use:** Use this for EVD-07 and REL-09 so the planner can close the workflow even when share outcome, live Stratum, or safe-stop runtime proof belongs to later phases. [VERIFIED: REQUIREMENTS.md; VERIFIED: 23-CONTEXT.md]

**Planning guidance:** Treat missing later-phase artifacts as first-class blocked or deferred slot outcomes, not absent files. [VERIFIED: 23-CONTEXT.md]

### Pattern 3: Redaction Contract Before Citation

**What:** The workflow should produce redacted committed artifacts and an explicit redaction review before any evidence is cited by parity docs. [VERIFIED: 23-CONTEXT.md; VERIFIED: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/redaction-review.md]

**When to use:** Use this for command summaries, logs, API captures, WebSocket captures, NVS/settings output, Stratum fields, share payloads, safe-stop summaries, and conclusions. [VERIFIED: 23-CONTEXT.md]

**Planning guidance:** Tests should use synthetic sentinel values for pool endpoints, ports, owner-address-like workers, passwords, device URLs, IPs, MACs, Wi-Fi values, Stratum targets, extranonces, share payloads, socket errors, and raw ASIC frames. [VERIFIED: 23-CONTEXT.md; VERIFIED: scripts/phase21-live-mining-evidence-test.sh; VERIFIED: firmware/bitaxe/src/controlled_mining_runtime.rs]

### Pattern 4: Exact Claim Ladder Integration

**What:** Phase 23 can prove the workflow and evidence-root/redaction governance while leaving trusted BM1366 production work and live share outcomes as explicit non-claims. [VERIFIED: 23-CONTEXT.md; VERIFIED: docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/claim-ladder.md]

**When to use:** Use this whenever checklist rows or release/operator docs are updated. [VERIFIED: 23-CONTEXT.md; VERIFIED: docs/parity/checklist.md]

**Planning guidance:** Evidence-governance rows may advance only when the redacted workflow and review pass; mining behavior rows should remain pending or non-claimed until their owning phases produce artifacts. [VERIFIED: 23-CONTEXT.md]

### Anti-Patterns to Avoid

- **Bypassing `just detect-ultra205`:** This violates the hardware evidence gate and makes the evidence untrustworthy. [VERIFIED: AGENTS.md; VERIFIED: 23-CONTEXT.md]
- **Using stale target discovery:** mDNS, ARP, router state, scans, stale logs, and unrelated evidence are not allowed substitutes for same-session target derivation. [VERIFIED: AGENTS.md; VERIFIED: .planning/research/PITFALLS.md]
- **Logging raw protocol data then redacting later:** Redaction is a deliverable and should be designed into each artifact type. [VERIFIED: 23-CONTEXT.md]
- **Promoting missing share outcomes:** Share-outcome proof can remain a non-claim until Phase 25 observes a real pool response to live ASIC-derived work. [VERIFIED: 23-CONTEXT.md; VERIFIED: docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/claim-ladder.md]
- **Embedding substantial Node/Python/Rust logic inside shell strings:** Keep complex helper logic in checked-in language-aware files. [VERIFIED: standards/core/code-shape.md]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Ultra 205 detection | Custom serial-port scan or remembered port | `just detect-ultra205` and `scripts/detect-ultra205.sh` | Existing detector enforces exactly one likely port and ESP32-S3 board-info. [VERIFIED: scripts/detect-ultra205.sh] |
| Flash/monitor evidence | Direct `espflash` docs without evidence metadata | `just flash-monitor ... redact-evidence=true evidence-dir=...` through `tools/flash` | Existing tool records command kind, board, commits, trusted output, monitor log, redaction mode, and commit-ready status. [VERIFIED: tools/flash/src/main.rs] |
| Pool credential parsing | New parser or environment-only inputs | `scripts/phase21-pool-credentials-json.mjs` | Existing helper validates the committed example shape and shell-quotes exports; tests cover missing/invalid fields without leaking values. [VERIFIED: scripts/phase21-pool-credentials-json.mjs; VERIFIED: scripts/phase21-live-mining-evidence-test.sh] |
| Pool settings application | Raw `curl` snippets | `scripts/phase21-pool-input-bridge.sh` | Existing bridge validates origin-only target, PATCHes settings, polls redacted logs for consumption, and records category labels. [VERIFIED: scripts/phase21-pool-input-bridge.sh] |
| WebSocket capture | One-off socket code | `scripts/phase17-websocket-capture.mjs` | Existing helper bounds duration and frames, validates paths, derives WebSocket URLs from origin-only device URLs, and redacts frames/errors. [VERIFIED: scripts/phase17-websocket-capture.mjs] |
| Allow/command validation | Manual command review only | `tools/parity` mining/evidence validators | Existing mining allow logic validates claim tiers, surfaces, package identity, abort conditions, safe-state markers, redaction reviewer completion, live-pool inputs, and bounded soak duration. [VERIFIED: tools/parity/src/mining_allow.rs] |
| Evidence redaction scan | Hand inspection only | Deterministic scan plus redaction-review checklist | Existing Phase 21 review uses `rg` scan and artifact inventory before citation. [VERIFIED: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/redaction-review.md] |

**Key insight:** Phase 23 is risky because it combines hardware, live credentials, runtime targets, and share language; repo-owned validators and redaction fixtures are cheaper than recovering from leaked evidence or overclaimed parity rows. [VERIFIED: .planning/research/PITFALLS.md; VERIFIED: 23-CONTEXT.md]

## Common Pitfalls

### Pitfall 1: Secret Leakage Across Secondary Artifacts

**What goes wrong:** Main logs are redacted, but command summaries, API bodies, WebSocket frames, retained logs, NVS/settings output, socket errors, or final summaries still contain sensitive values. [VERIFIED: 23-CONTEXT.md; VERIFIED: .planning/research/PITFALLS.md]

**Why it happens:** Phase 17/21 redaction is implemented in multiple shell/Node/Rust surfaces, so new Phase 23 artifact types can miss a category. [VERIFIED: scripts/phase17-live-http-api-smoke.sh; VERIFIED: scripts/phase17-websocket-capture.mjs; VERIFIED: scripts/phase21-live-mining-evidence.sh; VERIFIED: tools/flash/src/main.rs]

**How to avoid:** Add a required root inventory plus synthetic sentinel tests for every forbidden category before artifacts are cited. [VERIFIED: 23-CONTEXT.md]

**Warning signs:** `redaction_status` is missing, pending, blocked, or reviewer-less; `raw_artifacts_committed` is not `no`; redaction scan returns raw endpoints, workers, device URLs, IPs, MACs, Wi-Fi values, targets, extranonces, or share payloads. [VERIFIED: 23-CONTEXT.md; VERIFIED: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/redaction-review.md]

### Pitfall 2: Treating Blocked Hardware Slots As Missing Evidence

**What goes wrong:** The evidence root lacks share-outcome, safe-stop, API, or WebSocket slots because the run could not truthfully populate them. [VERIFIED: 23-CONTEXT.md]

**Why it happens:** It is tempting to skip unsupported slots instead of modeling blocked or deferred outcomes. [VERIFIED: 23-CONTEXT.md]

**How to avoid:** Require every slot in the evidence root to exist with `passed`, `blocked`, `pending`, or `deferred` status and an exact non-claim. [VERIFIED: 23-CONTEXT.md]

**Warning signs:** Final summaries say "not applicable" without owner, reason, later phase, and claim boundary. [VERIFIED: docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/claim-ladder.md]

### Pitfall 3: Using Stale `DEVICE_URL` Or Network Discovery

**What goes wrong:** API/WebSocket probes hit the wrong device or stale firmware and produce false evidence. [VERIFIED: .planning/research/PITFALLS.md]

**Why it happens:** HTTP/WebSocket capture needs an origin, and network discovery is convenient. [VERIFIED: .planning/research/PITFALLS.md]

**How to avoid:** Accept explicit origin-only `--device-url` only when allowed by the workflow, or derive it from same-session trusted flash-monitor evidence when the repo rules are satisfied; never scan. [VERIFIED: AGENTS.md; VERIFIED: scripts/phase17-live-http-api-smoke.sh; VERIFIED: scripts/phase17-websocket-capture.mjs]

**Warning signs:** Evidence mentions mDNS, ARP, router state, network scan, stale monitor log, or "last known" target. [VERIFIED: AGENTS.md; VERIFIED: .planning/research/PITFALLS.md]

### Pitfall 4: Allow-Manifest Bypass

**What goes wrong:** A direct mining, raw Stratum, raw BM1366, voltage/fan, erase, or flash command produces artifacts outside repo-owned validation. [VERIFIED: .planning/research/PITFALLS.md; VERIFIED: tools/parity/src/mining_allow.rs]

**Why it happens:** New procedure shapes can be faster to run directly than to teach the validator. [VERIFIED: .planning/research/PITFALLS.md]

**How to avoid:** Extend `tools/parity` and its tests for new Phase 23 surfaces or command categories before using them in evidence. [VERIFIED: 23-CONTEXT.md; VERIFIED: tools/parity/src/mining_allow.rs]

**Warning signs:** `allowed_command` contains prohibited tokens or a new wrapper is not validated by `mining-allow` or a Phase 23 equivalent. [VERIFIED: tools/parity/src/mining_allow.rs]

### Pitfall 5: Overclaiming Phase 24/25/26 Behavior

**What goes wrong:** The workflow proves redaction and evidence packaging, but docs imply trusted BM1366 production work, live Stratum success, accepted/rejected share outcome, final runtime telemetry, or full safe-stop runtime proof. [VERIFIED: 23-CONTEXT.md; VERIFIED: docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/claim-ladder.md]

**Why it happens:** Required Phase 23 slots include share-outcome, API, WebSocket, and safe-stop, which can sound like behavior proof unless status and non-claim language are explicit. [VERIFIED: 23-CONTEXT.md]

**How to avoid:** Use Phase 22 claim tiers and blocker reasons in all summaries and checklist notes. [VERIFIED: docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/claim-ladder.md; VERIFIED: docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/blocker-reasons.md]

**Warning signs:** Parity rows become `verified` for production mining behavior without hardware-smoke/soak evidence and exact outcome artifacts. [VERIFIED: tools/parity/src/main.rs; VERIFIED: docs/parity/checklist.md]

## Code Examples

Verified patterns from the repository:

### Detector Gate Command Shape

```bash
just detect-ultra205
```

This command is the required first hardware gate and runs the repo-owned detector script. [VERIFIED: Justfile; VERIFIED: scripts/detect-ultra205.sh; VERIFIED: AGENTS.md]

### Commit-Redacted Flash Evidence Shape

```bash
just flash-monitor board=205 port=<detected-port> redact-evidence=true evidence-dir=<phase-23-root>/package
```

This shape follows the existing `tools/flash` aliases for `redact-evidence` and `evidence-dir`; the planner must keep the actual detected port as runtime evidence and must not hardcode stale values. [VERIFIED: Justfile; VERIFIED: tools/flash/src/main.rs; VERIFIED: AGENTS.md]

### Runtime-Only Pool Credential Shape

```json
{
  "poolURL": "public-pool.io",
  "poolPort": 3333,
  "poolUser": "bc1q-your-address-here.bitaxe",
  "poolPassword": "x"
}
```

This is the committed example shape only; the real local credential file must not be read, printed, summarized, or committed. [VERIFIED: pool-credentials.json.example; VERIFIED: AGENTS.md]

### Redaction Review Scan Pattern

```bash
rg -n -i "ssid|wifi|password|pool|worker|token|device_url|nvs|stratum|https?://|([0-9]{1,3}\.){3}[0-9]{1,3}|([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}|secret|credential" docs/parity/evidence/phase-23-redacted-operator-evidence-workflow
```

This adapts the existing Phase 21 deterministic scan pattern; Phase 23 should add sentinel-specific tests for targets, extranonces, share payloads, socket errors, and raw BM1366 frames. [VERIFIED: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/redaction-review.md; VERIFIED: 23-CONTEXT.md]

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Controlled no-share evidence could be mistaken for production mining proof. [VERIFIED: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/summary.md] | Phase 22 claim ladder distinguishes controlled no-share, prerequisite readiness, live socket/runtime, live ASIC-derived share outcome, and deferred non-claims. [VERIFIED: docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/claim-ladder.md] | Phase 22 completed 2026-07-04. [VERIFIED: 22-03-SUMMARY.md] | Phase 23 must package evidence without promoting later mining behavior. [VERIFIED: 23-CONTEXT.md] |
| Evidence packs were phase-specific and spread across Phase 17/21/22 directories. [VERIFIED: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/evidence-contract.md; VERIFIED: docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/summary.md] | Phase 23 requires one v1.1 redacted evidence root with fixed slots. [VERIFIED: 23-CONTEXT.md] | Phase 23 scope. [VERIFIED: ROADMAP.md] | Planner should add a root inventory validator or checklist so every slot exists. [VERIFIED: 23-CONTEXT.md] |
| Redaction checks existed for selected Phase 17/21 paths. [VERIFIED: scripts/phase21-live-mining-evidence-test.sh; VERIFIED: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/redaction-review.md] | Phase 23 must cover retained logs, command summaries, API captures, WebSocket captures, NVS/settings values, Stratum fields, share payloads, device URLs, IPs, MACs, Wi-Fi values, and pool secrets before commit. [VERIFIED: REQUIREMENTS.md] | Phase 23 scope. [VERIFIED: ROADMAP.md] | Planner should require synthetic sentinel fixtures for categories not already covered. [VERIFIED: 23-CONTEXT.md] |

**Deprecated/outdated:**

- Treating Phase 21 controlled no-share evidence as accepted/rejected share proof is invalid for v1.1 parity. [VERIFIED: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/summary.md; VERIFIED: docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/claim-ladder.md]
- Using network scans or stale target inference for `DEVICE_URL` is prohibited for evidence. [VERIFIED: AGENTS.md]
- Committing raw local artifacts as evidence is prohibited; committed evidence must be redacted or category-only and declare `raw_artifacts_committed: no`. [VERIFIED: AGENTS.md; VERIFIED: 23-CONTEXT.md]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|

All claims in this research were verified from repository files, local tool probes, or repo-owned GSD context. No `[ASSUMED]` claims are required for planning. [VERIFIED: files_to_read; VERIFIED: local tool commands]

## Open Questions (RESOLVED)

1. **RESOLVED: No user-blocking questions for planning.**
   - What we know: Phase 23 may pass with blocked or pending hardware outcome slots when those slots are explicit later-phase non-claims and workflow/redaction/root validation passes. [VERIFIED: 23-CONTEXT.md]
   - What's unclear: Whether execution will produce live detector-gated hardware artifacts during this phase or only static/synthetic workflow proof is execution-time state. [VERIFIED: local `just detect-ultra205`; VERIFIED: 23-CONTEXT.md]
   - Recommendation: Plan both paths: hardware-success slots when detector-gated commands run, and blocked/pending slot artifacts when a gate cannot truthfully complete. [VERIFIED: 23-CONTEXT.md]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|-------------|-----------|---------|----------|
| Ultra 205 over USB | Optional hardware evidence path | Yes, exactly one detector-gated ESP32-S3 board-info check passed during research; raw MAC and exact port are intentionally not recorded here. [VERIFIED: local `just detect-ultra205`] | ESP32-S3 board-info success [VERIFIED: local `just detect-ultra205`] | If unavailable during execution, write blocked detector/board-info/hardware slots with exact reason. [VERIFIED: AGENTS.md; VERIFIED: 23-CONTEXT.md] |
| `espflash` | Detector, board-info, flash/monitor | Yes [VERIFIED: local `espflash --version`] | 4.0.1 [VERIFIED: local `espflash --version`] | Block hardware execution and keep static workflow validation. [VERIFIED: AGENTS.md] |
| Just | Operator commands | Yes [VERIFIED: local `just --version`] | 1.48.0 [VERIFIED: local `just --version`] | Use direct repo scripts only for tests; user-facing docs should still be `just`-reachable. [VERIFIED: Justfile; VERIFIED: 23-CONTEXT.md] |
| Bazel | Build/test/parity commands | Yes [VERIFIED: local `bazel --version`] | 9.1.1 [VERIFIED: .bazelversion; VERIFIED: local `bazel --version`] | No good fallback for canonical verification; planner should treat missing Bazel as blocking. [VERIFIED: AGENTS.md; VERIFIED: Justfile] |
| Cargo/Rust | Rust tests and tools | Yes [VERIFIED: local `cargo --version`; VERIFIED: local `rustc --version`] | cargo/rustc 1.88.0-nightly local [VERIFIED: local tool commands] | Use Bazel Rust targets when Cargo direct checks are not needed. [VERIFIED: Justfile; VERIFIED: tools/parity/BUILD.bazel] |
| Node.js | Pool JSON and WebSocket helpers | Yes [VERIFIED: local `node --version`] | 24.13.0 [VERIFIED: local `node --version`] | WebSocket/pool helper tests block if Node is missing. [VERIFIED: scripts/BUILD.bazel; VERIFIED: scripts/phase17-websocket-capture.mjs] |
| `curl` | API and settings bridge capture | Yes [VERIFIED: local `curl --version`] | 8.7.1 [VERIFIED: local `curl --version`] | API/WebSocket hardware slots become blocked or pending if missing. [VERIFIED: scripts/phase17-live-http-api-smoke.sh; VERIFIED: scripts/phase21-pool-input-bridge.sh] |
| `rg` | Redaction scan | Yes [VERIFIED: local `rg --version`] | 15.1.0 [VERIFIED: local `rg --version`] | Manual review alone should not replace deterministic scan; missing `rg` should block final redaction review. [VERIFIED: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/redaction-review.md] |
| Local pool credentials | Runtime-only pool input | Not probed; contents must not be read or summarized. [VERIFIED: AGENTS.md] | Runtime local file, if operator supplies one [VERIFIED: AGENTS.md] | Use blocked `pool_config` / live-prerequisite slots and synthetic tests. [VERIFIED: scripts/phase21-live-mining-evidence.sh; VERIFIED: 23-CONTEXT.md] |
| Local Wi-Fi credentials | Runtime-only flash input | Not probed; contents must not be read or summarized. [VERIFIED: AGENTS.md] | Runtime local file, if operator supplies one [VERIFIED: AGENTS.md] | Omit Wi-Fi seeding or mark slot blocked if networked capture requires it. [VERIFIED: AGENTS.md; VERIFIED: tools/flash/src/main.rs] |

**Missing dependencies with no fallback:**

- None for research and static planning. [VERIFIED: local tool commands]
- Bazel/Just/Node/curl/rg/espflash availability should be rechecked during execution because verification and hardware flows depend on the current shell environment. [VERIFIED: standards/core/verification.md]

**Missing dependencies with fallback:**

- Real local credential files are optional runtime inputs; if absent, the evidence root must use blocked or pending live/pool slots and still prove redaction/workflow behavior with synthetic fixtures. [VERIFIED: AGENTS.md; VERIFIED: 23-CONTEXT.md]

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Bazel `rust_test` for Rust tools and crates, Bazel `sh_test` for shell workflows. [VERIFIED: tools/parity/BUILD.bazel; VERIFIED: tools/flash/BUILD.bazel; VERIFIED: scripts/BUILD.bazel] |
| Config file | `MODULE.bazel`, `Cargo.toml`, `scripts/BUILD.bazel`, `tools/parity/BUILD.bazel`, `tools/flash/BUILD.bazel`. [VERIFIED: MODULE.bazel; VERIFIED: Cargo.toml; VERIFIED: scripts/BUILD.bazel; VERIFIED: tools/parity/BUILD.bazel; VERIFIED: tools/flash/BUILD.bazel] |
| Quick run command | `bazel test //scripts:phase21_live_mining_evidence_test //scripts:phase21_pool_input_bridge_test //tools/parity:tests //tools/flash:tests` for current reused surfaces; Phase 23 should add its own script test target. [VERIFIED: scripts/BUILD.bazel; VERIFIED: tools/parity/BUILD.bazel; VERIFIED: tools/flash/BUILD.bazel] |
| Full suite command | `just test` runs `bazel test //...`. [VERIFIED: Justfile] |

### Phase Requirements To Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| EVD-07 | Evidence root requires package, detector, board-info, command, log, API, WebSocket, share-outcome, redaction-review, safe-stop, and conclusion slots. [VERIFIED: REQUIREMENTS.md] | Rust unit or shell fixture test for root inventory | `bazel test //tools/parity:tests //scripts:phase23_redacted_operator_evidence_test` [RECOMMENDED] | `tools/parity:tests` exists; Phase 23 target does not exist yet. [VERIFIED: tools/parity/BUILD.bazel; VERIFIED: scripts/BUILD.bazel] |
| STR-10 | Committed logs/evidence redact pool, worker, owner-address, password, target, extranonce, share payload, and socket error values. [VERIFIED: REQUIREMENTS.md] | Synthetic redaction fixture test | `bazel test //scripts:phase23_redacted_operator_evidence_test //tools/parity:tests` [RECOMMENDED] | Existing Phase 21 redaction tests exist; Phase 23 expanded tests do not exist yet. [VERIFIED: scripts/phase21-live-mining-evidence-test.sh] |
| REL-09 | Operator flow covers detect, package or flash, credential input, bounded mining evidence, telemetry, safe stop, redaction, and review. [VERIFIED: REQUIREMENTS.md] | Shell workflow test with fake detector/package/curl/WebSocket helpers and blocked-path cases | `bazel test //scripts:phase23_redacted_operator_evidence_test` [RECOMMENDED] | Missing Wave 0 target. [VERIFIED: scripts/BUILD.bazel] |
| CFG-07 | Pool credentials stay runtime-only and committed evidence records only category labels. [VERIFIED: REQUIREMENTS.md] | Synthetic JSON credential test with forbidden-value assertions | `bazel test //scripts:phase21_live_mining_evidence_test //scripts:phase21_pool_input_bridge_test //scripts:phase23_redacted_operator_evidence_test` [RECOMMENDED] | Existing Phase 21 tests exist; Phase 23 target missing. [VERIFIED: scripts/BUILD.bazel] |
| EVD-09 | Redaction review gates all root artifact types before commit. [VERIFIED: REQUIREMENTS.md] | Deterministic scan/review fixture plus validator test | `bazel test //tools/parity:tests //scripts:phase23_redacted_operator_evidence_test` [RECOMMENDED] | Existing parity tests exist; Phase 23 root validator missing. [VERIFIED: tools/parity/BUILD.bazel] |

### Sampling Rate

- **Per task commit:** Run the narrow Bazel targets for changed scripts/Rust tools plus any new Phase 23 test target. [VERIFIED: standards/core/verification.md; VERIFIED: scripts/BUILD.bazel]
- **Per wave merge:** Run `just parity`, `just verify-reference`, and relevant targeted tests before updating evidence/checklist docs. [VERIFIED: 23-CONTEXT.md; VERIFIED: Justfile]
- **Phase gate:** Run `just test` or justified narrower repo-owned checks, `just parity`, `just verify-reference`, deterministic redaction scan/review, and GSD lifecycle validation for Phase 23. [VERIFIED: 23-CONTEXT.md; VERIFIED: Justfile; VERIFIED: .planning/config.json]

### Wave 0 Gaps

- [ ] `scripts/phase23-redacted-operator-evidence.sh` - operator workflow shell for REL-09 and EVD-07. [RECOMMENDED: 23-CONTEXT.md]
- [ ] `scripts/phase23-redacted-operator-evidence-test.sh` - synthetic redaction and blocked-slot coverage for STR-10, CFG-07, and EVD-09. [RECOMMENDED: 23-CONTEXT.md; VERIFIED: scripts/phase21-live-mining-evidence-test.sh]
- [ ] `tools/parity/src/operator_evidence.rs` or extension to existing validator - evidence-root inventory and redaction-review validation for EVD-07/EVD-09. [RECOMMENDED: tools/parity/src/mining_allow.rs; VERIFIED: 23-CONTEXT.md]
- [ ] `docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/evidence-contract.md` - committed root contract for EVD-07. [RECOMMENDED: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/evidence-contract.md; VERIFIED: 23-CONTEXT.md]
- [ ] `docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/redaction-review.md` - review checklist and deterministic scan for EVD-09. [RECOMMENDED: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/redaction-review.md; VERIFIED: 23-CONTEXT.md]

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V2 Authentication | No new authentication surface is planned for Phase 23. [VERIFIED: 23-CONTEXT.md] | Preserve existing API/firmware behavior and do not add credential exposure. [VERIFIED: AGENTS.md] |
| V3 Session Management | No browser/session surface is planned for Phase 23. [VERIFIED: 23-CONTEXT.md] | Not applicable beyond keeping API/WebSocket captures redacted. [VERIFIED: 23-CONTEXT.md] |
| V4 Access Control | Yes for hardware/evidence command authorization. [VERIFIED: AGENTS.md; VERIFIED: tools/parity/src/mining_allow.rs] | Enforce detector gate, board `205`, package identity, allow-manifest or equivalent validator, abort conditions, and safe-state markers. [VERIFIED: AGENTS.md; VERIFIED: tools/parity/src/mining_allow.rs] |
| V5 Input Validation | Yes for credential JSON, device URL, command args, and evidence manifests. [VERIFIED: scripts/phase21-pool-credentials-json.mjs; VERIFIED: scripts/phase17-websocket-capture.mjs; VERIFIED: tools/parity/src/mining_allow.rs] | Use typed parsers, origin-only URL checks, bounded durations/frame counts, and JSON schema-like validation in repo-owned helpers. [VERIFIED: scripts/phase17-websocket-capture.mjs; VERIFIED: scripts/phase21-pool-credentials-json.mjs] |
| V6 Cryptography | No new cryptographic primitive is planned. [VERIFIED: 23-CONTEXT.md] | Do not hand-roll crypto and do not claim credential-at-rest security or NVS encryption unless separately verified. [VERIFIED: .planning/research/PITFALLS.md] |
| V8 Data Protection | Yes for secrets and private runtime values in evidence. [VERIFIED: AGENTS.md; VERIFIED: 23-CONTEXT.md] | Runtime-only credential inputs, redacted committed artifacts, category labels only, `raw_artifacts_committed: no`, deterministic scan, and explicit redaction review. [VERIFIED: AGENTS.md; VERIFIED: 23-CONTEXT.md] |

### Known Threat Patterns For Phase 23

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Pool credential or owner identity leakage in committed evidence | Information Disclosure | Use runtime-only local credential files, synthetic fixtures for tests, category labels in summaries, and deterministic scan/review before citation. [VERIFIED: AGENTS.md; VERIFIED: 23-CONTEXT.md] |
| Stale or spoofed device target produces false API/WebSocket evidence | Spoofing / Tampering | Require detector-gated same-session target provenance or explicit origin-only URL rules; never use scans or stale logs. [VERIFIED: AGENTS.md; VERIFIED: scripts/phase17-live-http-api-smoke.sh] |
| Ad hoc command bypasses safety/evidence controls | Elevation of Privilege / Tampering | Keep commands in `just`/repo scripts and extend `tools/parity` validation when command shapes change. [VERIFIED: 23-CONTEXT.md; VERIFIED: tools/parity/src/mining_allow.rs] |
| Overclaiming unproven mining behavior | Repudiation / Integrity | Use Phase 22 claim ladder terms and explicit non-claims in every summary/checklist update. [VERIFIED: docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/claim-ladder.md] |
| Raw local artifacts accidentally committed | Information Disclosure | Store raw artifacts only in ignored/local paths and make committed root declare `raw_artifacts_committed: no`. [VERIFIED: AGENTS.md; VERIFIED: 23-CONTEXT.md] |
| Shell quoting or parser drift around local credential inputs | Tampering / Information Disclosure | Reuse existing JSON helper and keep generated command summaries redacted; avoid new `eval` paths unless constrained to trusted helper output. [VERIFIED: scripts/phase21-pool-credentials-json.mjs; VERIFIED: scripts/phase21-live-mining-evidence.sh] |

## Sources

### Primary (HIGH confidence)

- `.planning/phases/23-redacted-operator-evidence-workflow/23-CONTEXT.md` - locked decisions, discretion, deferred scope, reusable assets, and integration points. [VERIFIED: ReadFile]
- `.planning/REQUIREMENTS.md` - EVD-07, STR-10, REL-09, CFG-07, and EVD-09 requirement text. [VERIFIED: ReadFile]
- `.planning/ROADMAP.md` - Phase 23 goal, success criteria, dependency, and phase boundaries. [VERIFIED: ReadFile]
- `.planning/STATE.md` - v1.1 decisions, Phase 22 completion state, and current blockers. [VERIFIED: ReadFile]
- `AGENTS.md` and `AGENTS.bright-builds.md` - Ultra 205 hardware gate, local credential handling, redaction rules, workflow, and verification expectations. [VERIFIED: ReadFile]
- `standards/core/architecture.md`, `standards/core/code-shape.md`, `standards/core/testing.md`, `standards/core/verification.md`, `standards/languages/rust.md` - functional core, script shape, testing, verification, and Rust guidance. [VERIFIED: ReadFile]
- `Justfile`, `scripts/detect-ultra205.sh`, `scripts/phase21-live-mining-evidence.sh`, `scripts/phase21-live-mining-evidence-test.sh`, `scripts/phase21-pool-input-bridge.sh`, `scripts/phase21-pool-credentials-json.mjs`, `scripts/phase17-live-http-api-smoke.sh`, `scripts/phase17-websocket-capture.mjs` - existing command and evidence implementation patterns. [VERIFIED: ReadFile]
- `tools/parity/src/mining_allow.rs`, `tools/parity/src/release_evidence.rs`, `tools/parity/src/main.rs`, `tools/flash/src/main.rs` - existing Rust validators and flash/evidence redaction. [VERIFIED: rg; VERIFIED: ReadFile]
- `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/*` and `docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/*` - prior evidence contracts, redaction reviews, claim ladder, blocker reasons, and final summaries. [VERIFIED: ReadFile]

### Secondary (MEDIUM confidence)

- Local tool availability probes for Node, Bazel, Just, Cargo, Rust, Python, espflash, ripgrep, curl, and the read-only detector. [VERIFIED: local shell commands]

### Tertiary (LOW confidence)

- None. This research is codebase-internal and did not require web search or unverified ecosystem claims. [VERIFIED: tool strategy decision]

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH - the phase should reuse the repo's existing Rust/Bazel/Just/scripts/espflash/Node stack and no new external package is recommended. [VERIFIED: Cargo.toml; VERIFIED: MODULE.bazel; VERIFIED: Justfile; VERIFIED: scripts/BUILD.bazel]
- Architecture: HIGH - the repo already has the functional-core/imperative-shell split, parity validators, flash evidence redaction, and script-owned hardware orchestration patterns needed for Phase 23. [VERIFIED: standards/core/architecture.md; VERIFIED: tools/parity/src/mining_allow.rs; VERIFIED: tools/flash/src/main.rs; VERIFIED: scripts/phase21-live-mining-evidence.sh]
- Pitfalls: HIGH - the main pitfalls are directly documented in Phase 23 context, repo-local rules, and v1.1 pitfalls research. [VERIFIED: 23-CONTEXT.md; VERIFIED: AGENTS.md; VERIFIED: .planning/research/PITFALLS.md]
- Hardware availability: MEDIUM - `just detect-ultra205` succeeded during research, but execution must rerun it because hardware state is session-local. [VERIFIED: local `just detect-ultra205`; VERIFIED: AGENTS.md]

**Research date:** 2026-07-04  
**Valid until:** 2026-08-03 for codebase-local workflow guidance; revalidate local tool versions and detector status immediately before hardware execution. [VERIFIED: standards/core/verification.md; VERIFIED: AGENTS.md]
