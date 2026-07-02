# Phase 17: Live HTTP API And Static Evidence - Research

**Researched:** 2026-07-02 [VERIFIED: system/developer date]
**Domain:** Ultra 205 live HTTP/static/API/WebSocket evidence capture [VERIFIED: .planning/phases/17-live-http-api-and-static-evidence/17-CONTEXT.md]
**Confidence:** HIGH for route/evidence architecture; MEDIUM for live `DEVICE_URL` availability [VERIFIED: local code inspection, `just detect-ultra205`, and 17-CONTEXT.md]

<user_constraints>
## User Constraints (from CONTEXT.md)

All bullets in this section are copied from `.planning/phases/17-live-http-api-and-static-evidence/17-CONTEXT.md`. [VERIFIED: .planning/phases/17-live-http-api-and-static-evidence/17-CONTEXT.md]

### Locked Decisions

### Target Gate And Current-Device Identity

- **D-01:** Accept a live target only from an explicit `--device-url`, explicit
  `DEVICE_URL`, or a phase target-lock manifest created from explicit input. Do
  not scan the network, infer a target from serial logs, parse mDNS, inspect ARP
  tables, or guess from router state.
- **D-02:** Operator observations from a serial console, AP UI, router UI, or
  bench note may be recorded as target provenance, but helpers must not treat
  those observations as automatic discovery.
- **D-03:** Before live probes, require the repo hardware gate: `just
  detect-ultra205` finds exactly one likely ESP32-S3 port and `espflash
  board-info --chip esp32s3 --port <port> --non-interactive` succeeds for board
  `205`.
- **D-04:** Bind live evidence to current package and flash identity. Record
  source commit, reference commit, package manifest, selected board, selected
  port, detector output, flash/serial evidence dependency, exact commands, and
  conclusion. If the package/flash identity is stale, mismatched, or unknown,
  write blocked evidence instead of probing.
- **D-05:** Treat `DEVICE_URL` as an origin-only `http://` or `https://` URL.
  Reject or block userinfo, query, fragment, and path-bearing targets unless a
  future plan deliberately documents why a non-origin URL is safe.
- **D-06:** Run bounded reachability and `/api/system/info` board/device sanity
  before promoting any live route claim. Missing, invalid, unreachable, or
  wrong-device targets produce blocked evidence with no network scan fallback.

### Live Probe Set And Pass Criteria

- **D-07:** Start from the existing Phase 16 route helper behavior, but harden
  Phase 17 pass criteria where needed instead of treating logged headers as
  proof. Keep helper tests for missing URL, invalid URL, successful fake route
  coverage, response redaction, and blocked route outcomes.
- **D-08:** The bounded live route set is:
  `/`, `/assets/app.css.gz`, a representative missing static path, `/recovery`,
  `/api/system/info`, an unknown `/api/*` route, `/api/ws`, `/api/ws/live`,
  `POST /api/system/OTA` as route-presence evidence only, and `POST
  /api/system/OTAWWW` as the expected fail-closed gap response.
- **D-09:** Passing HTTP/static evidence requires route-specific markers:
  root returns 200 with release/static entry markers; `/assets/app.css.gz`
  returns 200 with CSS/gzip/cache headers; missing static returns 302 to `/`
  with the captive-portal redirect body; `/recovery` returns 200 with recovery
  page markers; `/api/system/info` returns JSON for the current device; unknown
  API returns the expected JSON 404 body.
- **D-10:** `GET /api/ws` and `GET /api/ws/live` returning 400 or 426 without
  WebSocket upgrade is route-coexistence evidence only. It proves those paths do
  not fall through to static wildcard handling; it does not verify WebSocket
  framing.
- **D-11:** `POST /api/system/OTA` in Phase 17 proves route presence and
  validation-path reachability only. It must not be cited as valid OTA upload,
  invalid image rejection, reboot, rollback, selected partition, or
  boot-validation evidence.
- **D-12:** `POST /api/system/OTAWWW` may record the live fail-closed public
  response such as `Wrong API input`, but OTAWWW whole-`www` update parity
  remains deferred until Phase 19-style hardware-regression evidence exists.

### WebSocket Proof Depth

- **D-13:** Use both no-upgrade route checks and bounded real WebSocket capture
  before making any verified WebSocket claim. The no-upgrade checks prove route
  precedence; WebSocket frame capture proves actual upgrade/message behavior.
- **D-14:** `/api/ws/live` may be promoted only when a bounded capture records a
  redacted connect or cadence frame from the just-flashed device.
- **D-15:** `/api/ws` may be promoted only when a bounded capture records a
  redacted log frame or other accepted raw-log stream evidence. If the WebSocket
  opens but no log frame appears before timeout, record the open/timeout result
  and keep `/api/ws` frame evidence pending.
- **D-16:** WebSocket capture must stay bounded and evidence-focused. It should
  not introduce pool credentials, mining smoke, production soak, or long-running
  telemetry collection into Phase 17.

### Evidence Artifacts And Redaction

- **D-17:** Write Phase 17 live evidence under
  `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/`.
  Expected artifacts include a summary ledger, helper logs, per-route sanitized
  header/body/error artifacts, WebSocket frame metadata or redacted snippets,
  detector and package/flash identity references, and a redaction review.
- **D-18:** Prefer committed, redacted micro-artifacts plus a summary ledger for
  replayable proof. Do not commit raw live response dumps or full WebSocket
  payloads unless a plan introduces a quarantine/retention contract and the
  public ledger cites only redacted or allowlisted values.
- **D-19:** Store only typed allowlisted fields and sanitized snippets for API
  responses and WebSocket frames where practical. Redact `DEVICE_URL`, private
  endpoints, IP addresses, MAC addresses, Wi-Fi credentials, pool credentials,
  worker secrets, API tokens, NVS secret values, and local terminal secrets.
- **D-20:** Mark absent artifacts explicitly as `absent - not cited` in the
  redaction review. Do not cite body/header/frame artifacts that were not
  generated or reviewed.
- **D-21:** Use standalone `---` only for YAML frontmatter delimiters at the top
  of parsed Markdown artifacts. Use headings or `***` for body separation.

### Checklist, Release Docs, And Traceability

- **D-22:** Promote only rows and notes supported by exact Phase 17 artifacts.
  Guard-aligned promotion is allowed for live HTTP/API/static/recovery/WebSocket
  surfaces when evidence contains the required live terms and no blocker
  language.
- **D-23:** `FS-001` can become `verified` only with hardware evidence that
  names live static behavior, `/assets/app.css.gz`, missing static redirect,
  and `/recovery`, with no blocked/pending language.
- **D-24:** API rows such as `API-004`, `API-005`, `API-006`, `API-007`, and
  `API-008` can be updated or promoted only for the precise live route, frame,
  recovery page, and static asset behaviors captured. Do not let route
  registration or no-upgrade responses stand in for frame-level WebSocket proof.
- **D-25:** Keep `OTA-001`, `OTA-002`, and `REL-003` below verified in Phase 17.
  Keep valid OTA, invalid image rejection, rollback, boot validation, large
  erase, failed-update, interrupted-update, and whole-`www` OTAWWW behavior as
  explicit non-claims.
- **D-26:** Release docs, parity checklist, and requirements traceability should
  cite commands and artifacts, not goals or implementation existence. If
  `DEVICE_URL` or WebSocket capture is unavailable, update docs with blocked or
  pending evidence instead of promoting rows.

### the agent's Discretion

The agent may choose exact helper names, whether Phase 17 wraps or copies the
Phase 16 HTTP/static helper, the target-lock manifest schema, JSON field names,
timeout values, test fixture layout, and whether WebSocket capture uses Node,
Rust host tooling, or another repo-owned bounded client. Those choices must
preserve explicit target input, no network scanning, current package/flash
identity, redaction, test coverage, and exact-claim checklist boundaries.

### Deferred Ideas (OUT OF SCOPE)

- Valid firmware OTA upload, invalid image rejection, reboot identity, rollback,
  and boot-validation evidence belong to Phase 18.
- Recovery fault-injection regressions, failed-update recovery, large-erase
  recovery, interrupted-update recovery, and OTAWWW whole-`www` update behavior
  belong to Phase 19.
- Active safety hardware telemetry evidence belongs to Phase 20.
- Live production mining, accepted/rejected share behavior, watchdog
  responsiveness under mining, and bounded soak evidence belong to Phase 21.
- Network discovery, mDNS-based target selection, router scraping, AP scan
  automation, non-205 boards, all-board release images, Stratum v2, BAP,
  Angular AxeOS replacement, and production mining tuning remain out of Phase 17.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
| --- | --- | --- |
| API-09 | Static AxeOS assets and recovery page behavior remain compatible enough for device administration without requiring an Angular rewrite in V1. [VERIFIED: .planning/REQUIREMENTS.md] | Use live `/`, `/assets/app.css.gz`, missing-static redirect, `/recovery`, API coexistence, and WebSocket frame artifacts before promoting API/static rows. [VERIFIED: 17-CONTEXT.md, scripts/phase16-http-static-smoke.sh, firmware/bitaxe/src/static_files.rs, firmware/bitaxe/src/http_api.rs] |
| REL-01 | Partition layout, filesystem layout, SPIFFS/static assets, and recovery assets support upstream-like flash and administration flows. [VERIFIED: .planning/REQUIREMENTS.md] | Tie live static/recovery evidence to package manifest, serial SPIFFS evidence, and same board/port flash identity. [VERIFIED: docs/parity/evidence/phase-16-current-commit-release-evidence-completion.md, firmware/bitaxe/BUILD.bazel] |
| REL-07 | Build, flash, monitor, OTA, and recovery documentation is sufficient for a developer with a connected Ultra 205 to operate safely. [VERIFIED: .planning/REQUIREMENTS.md] | Update `docs/release/ultra-205.md` with exact Phase 17 commands/artifacts and non-claims instead of goals. [VERIFIED: docs/release/ultra-205.md, 17-CONTEXT.md] |
| EVD-05 | Verification layers include unit tests, golden fixtures, API comparison, hardware smoke tests, and hardware regression or soak evidence where appropriate. [VERIFIED: .planning/REQUIREMENTS.md] | Add Phase 17 helper tests, live hardware smoke artifacts, redaction review, `just parity`, and `just verify-reference`; leave destructive/regression evidence to later phases. [VERIFIED: .planning/config.json, scripts/BUILD.bazel, 17-CONTEXT.md] |
</phase_requirements>

## Summary

Phase 17 is primarily an evidence and helper-hardening phase, not a firmware feature phase, because the route shell, static/recovery planner, OTA/OTAWWW route decisions, and WebSocket state bridge already exist in the repo. [VERIFIED: firmware/bitaxe/src/http_api.rs, firmware/bitaxe/src/static_files.rs, crates/bitaxe-api/src/static_plan.rs, crates/bitaxe-api/src/update_plan.rs, firmware/bitaxe/src/websocket_api.rs]

The plan should create Phase 17-owned helpers and artifacts rather than mutating Phase 16 history, then run a fresh package/detect/flash evidence chain before any live probes. [VERIFIED: 17-CONTEXT.md, docs/parity/evidence/phase-16-current-commit-release-evidence-completion.md] The current `bazel-bin` package manifest records source commit `3e1a64089fb63432c9995308ee46aec17899b21d`, while current `HEAD` is `1d664cb1a779ef366a709a50512e2b2766bdfbec`; therefore planning must rebuild/package before claiming current live evidence. [VERIFIED: bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json, `git rev-parse HEAD`]

**Primary recommendation:** build `scripts/phase17-live-http-api-smoke.sh` plus a generalized `scripts/phase17-websocket-capture.mjs`, reusing Phase 16 route/redaction behavior and Phase 15 Node WebSocket capture patterns, and gate all live claims on explicit `DEVICE_URL`, detector success, fresh package/flash identity, bounded route/frame proof, and redaction review. [VERIFIED: scripts/phase16-http-static-smoke.sh, scripts/phase15-websocket-capture.mjs, 17-CONTEXT.md]

## Project Constraints (from AGENTS.md)

- Read `AGENTS.md`, `AGENTS.bright-builds.md`, relevant standards pages, and `standards-overrides.md` before planning or implementation. [VERIFIED: AGENTS.md, AGENTS.bright-builds.md, standards/index.md]
- Use ESP-IDF/esp-rs tooling first for firmware build, package, flash, monitor, partition, OTA, SPIFFS, NVS, FreeRTOS, and logging workflows. [VERIFIED: AGENTS.md]
- Treat `.embuild/` as local generated ESP-IDF/esp-rs state; do not commit or hand-edit it. [VERIFIED: AGENTS.md]
- Run `just detect-ultra205` before autonomous hardware use, and proceed only when exactly one ESP32-S3 candidate is found and `espflash board-info --chip esp32s3 --port <port> --non-interactive` succeeds. [VERIFIED: AGENTS.md]
- Stop or record evidence pending when no port, multiple ports, failed board-info, non-205 target, or missing recovery/evidence instructions occur. [VERIFIED: AGENTS.md]
- Every hardware run must record board `205`, selected port, source commit, reference commit, package manifest/artifacts when applicable, exact commands, board-info output, captured logs, observed behavior, and conclusion. [VERIFIED: AGENTS.md]
- Do not commit secrets, pool credentials, Wi-Fi credentials, private endpoints, or NVS secret values in evidence. [VERIFIED: AGENTS.md]
- Use standalone `---` only for YAML frontmatter delimiters at the top of parsed Markdown artifacts; do not use body separators that are standalone `---`. [VERIFIED: AGENTS.md]
- Prefer functional core / imperative shell, parse boundary data into domain types, and make illegal states unrepresentable where practical. [VERIFIED: standards/core/architecture.md]
- Unit-test pure/business logic and structure non-trivial unit tests with Arrange, Act, Assert. [VERIFIED: standards/core/testing.md]
- Prefer repo-owned verification entrypoints before commits; Rust commits require `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` when Rust paths change. [VERIFIED: AGENTS.md, standards/core/verification.md, standards/languages/rust.md]
- Project-local skills were not found under `.claude/skills/` or `.agents/skills/`. [VERIFIED: `find .claude/skills .agents/skills -maxdepth 2 -name SKILL.md`]

## Standard Stack

### Core

| Library / Tool | Version | Purpose | Why Standard |
| --- | --- | --- | --- |
| `scripts/phase16-http-static-smoke.sh` pattern | repo-owned | Existing explicit-`DEVICE_URL` HTTP/static/recovery/API/OTA route probe and redaction pattern. [VERIFIED: scripts/phase16-http-static-smoke.sh] | Reuse its per-route headers/body/error artifact pattern, route status matching, no-scan blocker output, and redaction tests instead of inventing a second route probe style. [VERIFIED: scripts/phase16-http-static-smoke.sh, scripts/phase16-http-static-smoke-test.sh] |
| `curl` | 8.7.1 | Live HTTP status/header/body capture for bounded route set. [VERIFIED: `curl --version`] | Existing helpers use `curl --dump-header`, `--write-out "%{http_code}"`, and bounded `--max-time` for replayable route evidence. [VERIFIED: scripts/phase16-http-static-smoke.sh] |
| Node.js built-in WebSocket client | v24.13.0, `globalThis.WebSocket` present | Bounded WebSocket frame capture for `/api/ws/live` and `/api/ws`. [VERIFIED: `node --version` and `node -e 'console.log(typeof globalThis.WebSocket)'`] | Node's official docs state the built-in client can connect with `new WebSocket()` and avoids client-side `ws`/`socket.io` dependencies. [CITED: https://nodejs.org/learn/getting-started/websocket] |
| `scripts/phase15-websocket-capture.mjs` pattern | repo-owned | Existing bounded, redacting WebSocket capture client. [VERIFIED: scripts/phase15-websocket-capture.mjs] | Generalize this helper to accept an allowlisted WebSocket path instead of adding a package dependency or long-running telemetry process. [VERIFIED: scripts/phase15-websocket-capture.mjs, scripts/phase15-controlled-mining-test.sh] |
| `just detect-ultra205` / `espflash board-info` | `just` 1.48.0, `espflash` 4.0.1 | Required read-only Ultra 205 hardware gate. [VERIFIED: `just --version`, `espflash --version`, `just detect-ultra205`] | Repo-local guidance requires this gate before autonomous hardware use, and local detection found exactly one ESP32-S3 candidate on `/dev/cu.usbmodem1101`. [VERIFIED: AGENTS.md, `just detect-ultra205`] |
| Bazel + `rules_shell` | Bazel 9.1.1, `rules_shell` 0.8.0 | Add Phase 17 helper binaries/tests under the canonical automation graph. [VERIFIED: `bazel --version`, MODULE.bazel, scripts/BUILD.bazel] | Existing shell helpers are exposed through `sh_binary`/`sh_test`, and `just test` routes through `bazel test //...`. [VERIFIED: Justfile, scripts/BUILD.bazel] |

### Supporting

| Library / Tool | Version | Purpose | When to Use |
| --- | --- | --- | --- |
| `tools/parity` | repo-owned Rust binary | `just parity`, checklist guards, release/static/WebSocket evidence semantics, and optional release evidence validation. [VERIFIED: tools/parity/src/main.rs, tools/parity/src/release_evidence.rs] | Use before finalizing docs/checklist changes, and extend only if Phase 17 needs new guard logic. [VERIFIED: Justfile, tools/parity/src/main.rs] |
| `tools/flash` | repo-owned Rust binary | `just flash-monitor board=205 ... evidence-dir=...` package-backed flash/serial evidence. [VERIFIED: Justfile, tools/flash/src/main.rs] | Use to bind the live HTTP evidence to the just-flashed package identity. [VERIFIED: 17-CONTEXT.md, .planning/phases/16-current-commit-release-evidence-completion/16-VERIFICATION.md] |
| Python 3 | 3.14.4 | Existing helper reads package manifest fields. [VERIFIED: `python3 --version`, scripts/phase16-http-static-smoke.sh] | Keep current small manifest-field use if wrapping Phase 16; avoid adding a new substantial Python script for Phase 17 helper logic. [VERIFIED: scripts/phase16-http-static-smoke.sh, standards/core/code-shape.md] |
| `crates/bitaxe-api` | workspace crate | Route/static/update/WebSocket pure contracts. [VERIFIED: Cargo.toml, crates/bitaxe-api/BUILD.bazel] | Reference for expected route bodies, markers, and claim boundaries; do not duplicate route policy in docs by hand. [VERIFIED: crates/bitaxe-api/src/route_shell.rs, crates/bitaxe-api/src/static_plan.rs, crates/bitaxe-api/src/update_plan.rs] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
| --- | --- | --- |
| Node built-in WebSocket client | npm `ws` or `wscat` | Avoid new dependency management because local Node v24 has a built-in client and the task only needs client capture, not a server. [VERIFIED: local Node probe; CITED: https://nodejs.org/learn/getting-started/websocket] |
| `curl` for HTTP probes | ad hoc `nc`, browser automation, or raw sockets | `curl` already gives bounded status/header/body artifacts and is used by existing route helpers. [VERIFIED: scripts/phase16-http-static-smoke.sh] |
| Explicit `DEVICE_URL` input | mDNS, ARP, router UI scraping, serial-log parsing | Locked out of scope; no network discovery or inferred target is allowed. [VERIFIED: 17-CONTEXT.md] |
| Phase 16 evidence files | Mutate historical Phase 16 artifacts | Phase 17 must write under its own evidence directory and cite exact Phase 17 artifacts. [VERIFIED: 17-CONTEXT.md] |

**Installation:**

```bash
# No npm package install is recommended for Phase 17. [VERIFIED: no package.json; Node v24 global WebSocket probe]
just doctor
just bootstrap-esp
```

**Version verification:** Tool versions were verified with `bash --version`, `curl --version`, `node --version`, `node -e 'console.log(typeof globalThis.WebSocket)'`, `espflash --version`, `bazel --version`, `just --version`, and `python3 --version`. [VERIFIED: local command probes]

## Architecture Patterns

### Recommended Project Structure

```text
scripts/
├── phase17-live-http-api-smoke.sh        # Phase-owned HTTP/static/API probe wrapper. [VERIFIED: scripts/BUILD.bazel pattern]
├── phase17-live-http-api-smoke-test.sh   # Shell tests for no-scan, URL validation, route matching, redaction, stale identity. [VERIFIED: scripts/phase16-http-static-smoke-test.sh pattern]
├── phase17-websocket-capture.mjs         # Generalized bounded WS frame capture for /api/ws/live and /api/ws. [VERIFIED: scripts/phase15-websocket-capture.mjs pattern]
└── BUILD.bazel                           # sh_binary/sh_test wiring for the helpers. [VERIFIED: scripts/BUILD.bazel]

docs/parity/evidence/phase-17-live-http-api-and-static-evidence/
├── summary.md                            # Final ledger with exact claims and non-claims. [VERIFIED: 17-CONTEXT.md]
├── target-lock.json                      # Sanitized explicit-input target/package/flash identity, if split commands need it. [VERIFIED: 17-CONTEXT.md]
├── http-static-api/                      # Per-route sanitized headers/body/error artifacts. [VERIFIED: scripts/phase16-http-static-smoke.sh]
├── websocket/                            # Redacted frame metadata/snippets. [VERIFIED: scripts/phase15-websocket-capture.mjs]
└── redaction-review.md                   # Artifact-by-artifact review, including absent artifacts. [VERIFIED: 17-CONTEXT.md]
```

### Pattern 1: Explicit Target Lock Before Live Probes

**What:** Validate `DEVICE_URL` as an origin-only `http://` or `https://` URL, store only redacted/sanitized target provenance, and bind it to package manifest, detector output, selected port, serial/flash evidence, source commit, and reference commit. [VERIFIED: 17-CONTEXT.md]

**When to use:** Use when HTTP and WebSocket captures are separate commands or when a retry should prove it is still tied to the same explicit target and flashed package. [VERIFIED: 17-CONTEXT.md]

**Example:**

```javascript
// Source: Node URL API and Phase 17 D-05. [CITED: https://nodejs.org/learn/getting-started/websocket] [VERIFIED: 17-CONTEXT.md]
export function parseOriginDeviceUrl(value) {
  const url = new URL(value);
  if (url.protocol !== "http:" && url.protocol !== "https:") {
    throw new Error("DEVICE_URL must use http or https");
  }
  if (url.username || url.password || url.search || url.hash || (url.pathname && url.pathname !== "/")) {
    throw new Error("DEVICE_URL must be an origin without userinfo, path, query, or fragment");
  }
  return `${url.protocol}//${url.host}`;
}
```

### Pattern 2: HTTP Route Evidence Uses Per-Route Micro-Artifacts

**What:** For each route, write a bounded header file, body file, curl error file, sanitized body snippet, selected headers, expected marker, actual status, and route conclusion. [VERIFIED: scripts/phase16-http-static-smoke.sh]

**When to use:** Use for `/`, `/assets/app.css.gz`, a Phase 17 missing path such as `/phase17-missing-static`, `/recovery`, `/api/system/info`, unknown `/api/phase17-unknown`, no-upgrade `/api/ws`, no-upgrade `/api/ws/live`, `POST /api/system/OTA`, and `POST /api/system/OTAWWW`. [VERIFIED: 17-CONTEXT.md, scripts/phase16-http-static-smoke.sh]

**Example:**

```bash
# Source: Phase 16 helper route pattern. [VERIFIED: scripts/phase16-http-static-smoke.sh]
probe_route "app-css-gz" "GET" "/assets/app.css.gz" "200" "" \
  "200 with static CSS headers and gzip/cache evidence"
```

### Pattern 3: Split WebSocket Route Coexistence From Frame Proof

**What:** First use `curl` without upgrade headers to show `/api/ws` and `/api/ws/live` do not fall through to static wildcard routing; then use a real WebSocket client for bounded frame capture. [VERIFIED: 17-CONTEXT.md]

**When to use:** Always for WebSocket claims; a 400/426 HTTP result is route-coexistence proof only. [VERIFIED: 17-CONTEXT.md]

**Example:**

```javascript
// Source: existing Phase 15 WebSocket helper generalized with an allowlisted path. [VERIFIED: scripts/phase15-websocket-capture.mjs]
const socket = new globalThis.WebSocket(wsUrl);
socket.addEventListener("message", (event) => {
  frames += 1;
  lines.push(`websocket_frame_${frames}=${redactText(event.data)}`);
});
```

### Pattern 4: Conservative Checklist Promotion

**What:** Promote only rows whose notes cite exact Phase 17 artifacts and contain no blocker language for the claimed behavior. [VERIFIED: 17-CONTEXT.md, tools/parity/src/main.rs]

**When to use:** Use after redaction review passes and before final `just parity`. [VERIFIED: 17-CONTEXT.md, Justfile]

### Anti-Patterns to Avoid

- **Treating route registration as live proof:** Serial route-registration logs do not prove HTTP status/body/header behavior. [VERIFIED: docs/parity/evidence/phase-16-current-commit-release-evidence-completion.md]
- **Treating 400/426 no-upgrade responses as WebSocket frame proof:** These responses show route precedence only. [VERIFIED: 17-CONTEXT.md]
- **Committing raw `DEVICE_URL` or response dumps:** Phase 17 requires redacted micro-artifacts and sanitized snippets. [VERIFIED: 17-CONTEXT.md]
- **Using `DEVICE_URL` from environment as implicit authorization in all helpers:** Phase 15 tests explicitly reject env-only authorization for live mining; Phase 17 allows explicit env input but must still record it as explicit provenance and sanitize it. [VERIFIED: scripts/phase15-controlled-mining-test.sh, 17-CONTEXT.md]
- **Adding long-running telemetry or mining behavior:** Phase 17 WebSocket capture must be bounded and evidence-focused. [VERIFIED: 17-CONTEXT.md]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
| --- | --- | --- | --- |
| HTTP route capture | Raw sockets or custom HTTP parser | `curl` with bounded timeouts and per-route artifacts. [VERIFIED: scripts/phase16-http-static-smoke.sh] | Existing helper already handles status/header/body/error capture and redaction. [VERIFIED: scripts/phase16-http-static-smoke-test.sh] |
| WebSocket client protocol | Manual Upgrade headers, `nc`, or homemade frame parser | Node v24 built-in `WebSocket` client. [VERIFIED: local Node probe; CITED: https://nodejs.org/learn/getting-started/websocket] | WebSocket framing, close/error events, and message callbacks are protocol details the phase should not reimplement. [CITED: https://nodejs.org/learn/getting-started/websocket] |
| Target discovery | mDNS parsing, ARP scans, router scraping, serial-log inference | Explicit `--device-url`, explicit `DEVICE_URL`, or target-lock manifest from explicit input. [VERIFIED: 17-CONTEXT.md] | Network discovery is locked out of scope and would weaken evidence provenance. [VERIFIED: 17-CONTEXT.md] |
| Firmware route source of truth | A second hardcoded route manifest in the helper | `phase07_routes()` and existing firmware/static/update code as source references; keep helper route list aligned with D-08. [VERIFIED: crates/bitaxe-api/src/route_shell.rs, 17-CONTEXT.md] | The repo already has route ownership and parity guards; duplicating policy increases drift risk. [VERIFIED: tools/parity/src/api_compare.rs] |
| Redaction review | Unreviewed grep output or raw dumps | Artifact-specific redaction review with absent artifacts marked `absent - not cited`. [VERIFIED: 17-CONTEXT.md] | Checklist promotion depends on reviewed, cited artifacts. [VERIFIED: 17-CONTEXT.md, docs/parity/evidence/phase-16-current-commit-release-evidence-completion/redaction-review.md] |
| Package/flash identity | Manual commit notes without manifest/flash evidence | `just package`, `just flash-monitor ... evidence-dir=...`, package manifest copy, detector log, flash JSON. [VERIFIED: Justfile, tools/flash/src/main.rs, docs/parity/evidence/phase-16-current-commit-release-evidence-completion.md] | Phase 17 claims must be tied to the just-flashed package, not a stale or inferred source. [VERIFIED: 17-CONTEXT.md] |

**Key insight:** The hard part is evidence integrity, not route implementation; custom discovery, custom protocol code, or broad artifacts would make the evidence less trustworthy. [VERIFIED: 17-CONTEXT.md, scripts/phase16-http-static-smoke.sh, scripts/phase15-websocket-capture.mjs]

## Common Pitfalls

### Pitfall 1: Existing Package Manifest Is Not Current HEAD

**What goes wrong:** Live evidence cites a package manifest whose `source_commit` does not match the code intended for Phase 17. [VERIFIED: bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json, `git rev-parse HEAD`]

**Why it happens:** Package artifacts in `bazel-bin` can predate the current repository `HEAD`. [VERIFIED: local package manifest and git probe]

**How to avoid:** Run `just package`, copy the manifest into the Phase 17 evidence directory, run `just detect-ultra205`, then flash-monitor that manifest before HTTP/WebSocket probes. [VERIFIED: Justfile, 17-CONTEXT.md]

**Warning signs:** Package `source_commit` differs from `git rev-parse HEAD`, or flash JSON points to a different manifest. [VERIFIED: tools/parity/src/release_evidence.rs]

### Pitfall 2: `DEVICE_URL` Is Present But Not Origin-Only

**What goes wrong:** A URL with path, query, fragment, or userinfo causes route probes to target the wrong base or leaks secrets into artifacts. [VERIFIED: 17-CONTEXT.md]

**Why it happens:** Phase 16 only rejects non-HTTP schemes; Phase 17 D-05 requires stricter origin-only validation. [VERIFIED: scripts/phase16-http-static-smoke.sh, 17-CONTEXT.md]

**How to avoid:** Validate with a URL parser before probing and store only sanitized provenance. [VERIFIED: 17-CONTEXT.md]

**Warning signs:** `DEVICE_URL` includes `@`, `?`, `#`, or a path other than `/`. [VERIFIED: 17-CONTEXT.md]

### Pitfall 3: No-Upgrade WebSocket Response Is Overclaimed

**What goes wrong:** `/api/ws` or `/api/ws/live` returning 400/426 is cited as WebSocket behavior. [VERIFIED: 17-CONTEXT.md]

**Why it happens:** ESP-IDF WebSocket routes are HTTP server URI handlers, and a non-upgrade GET can still prove route precedence. [CITED: https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/protocols/esp_http_server.html] [VERIFIED: firmware/bitaxe/src/http_api.rs]

**How to avoid:** Record no-upgrade responses as coexistence evidence only and require a bounded `WebSocket` frame artifact for verified WebSocket claims. [VERIFIED: 17-CONTEXT.md]

**Warning signs:** Checklist notes say WebSocket verified but cite only `curl` status/header artifacts. [VERIFIED: 17-CONTEXT.md, tools/parity/src/main.rs]

### Pitfall 4: Raw `/api/ws` May Open But Emit No Frame

**What goes wrong:** `/api/ws` capture times out and is mistaken for a failure of route registration or static routing. [VERIFIED: crates/bitaxe-api/src/logs.rs, firmware/bitaxe/src/http_api.rs]

**Why it happens:** The raw log stream starts its cursor at the current retained-log end for active clients and does not replay old retained history. [VERIFIED: crates/bitaxe-api/src/logs.rs]

**How to avoid:** Treat an open/no-frame result as pending raw-log frame evidence unless a bounded route action produces a redacted raw log frame. [VERIFIED: 17-CONTEXT.md, crates/bitaxe-api/src/logs.rs]

**Warning signs:** `/api/ws/live` passes with an `event=update` frame, while `/api/ws` records open/timeout with no frame. [VERIFIED: crates/bitaxe-api/src/telemetry.rs, crates/bitaxe-api/src/logs.rs]

### Pitfall 5: Empty Firmware OTA POST Is Overclaimed

**What goes wrong:** The empty `POST /api/system/OTA` route-presence probe is cited as valid OTA, invalid image rejection, rollback, selected partition, or boot-validation evidence. [VERIFIED: 17-CONTEXT.md]

**Why it happens:** The existing route helper uses an empty POST to exercise the validation path, and the firmware OTA adapter can emit validation/protocol status without a valid firmware upload. [VERIFIED: scripts/phase16-http-static-smoke.sh, firmware/bitaxe/src/ota_update.rs]

**How to avoid:** Cite it only as route presence and validation-path reachability. [VERIFIED: 17-CONTEXT.md]

**Warning signs:** `OTA-001`, `OTA-002`, or `REL-003` move to verified during Phase 17. [VERIFIED: 17-CONTEXT.md]

### Pitfall 6: Redaction Review Cites Missing Artifacts

**What goes wrong:** Final ledger cites body/header/frame artifacts that were not generated or reviewed. [VERIFIED: 17-CONTEXT.md]

**Why it happens:** Blocked paths create logs but not route artifacts; Phase 16 handled this by marking absent artifacts as not cited. [VERIFIED: docs/parity/evidence/phase-16-current-commit-release-evidence-completion/http-static-recovery.md]

**How to avoid:** Make redaction review enumerate generated artifacts and explicitly mark missing ones `absent - not cited`. [VERIFIED: 17-CONTEXT.md]

**Warning signs:** Redaction review has generic `passed` text but no artifact matrix. [VERIFIED: 17-CONTEXT.md]

## Code Examples

### Origin-Only Target Validation

```javascript
// Source: Phase 17 D-05 and Node built-in URL support. [VERIFIED: 17-CONTEXT.md] [CITED: https://nodejs.org/learn/getting-started/websocket]
function sanitizedOrigin(value) {
  const url = new URL(value);
  if (!["http:", "https:"].includes(url.protocol)) {
    throw new Error("invalid scheme");
  }
  if (url.username || url.password || url.search || url.hash || (url.pathname && url.pathname !== "/")) {
    throw new Error("not an origin URL");
  }
  return `${url.protocol}//[redacted]`;
}
```

### Generalized WebSocket Capture

```javascript
// Source: Phase 15 helper pattern. [VERIFIED: scripts/phase15-websocket-capture.mjs]
function websocketUrlFromDeviceUrl(deviceUrl, path) {
  if (path !== "/api/ws" && path !== "/api/ws/live") {
    throw new Error("unsupported WebSocket path");
  }
  const parsed = new URL(deviceUrl);
  parsed.protocol = parsed.protocol === "https:" ? "wss:" : "ws:";
  parsed.pathname = path;
  parsed.search = "";
  parsed.hash = "";
  return parsed;
}
```

### Redaction Scan Command

```bash
# Source: Phase 16 redaction-review pattern and Phase 17 D-19. [VERIFIED: docs/parity/evidence/phase-16-current-commit-release-evidence-completion/redaction-review.md] [VERIFIED: 17-CONTEXT.md]
rg -n -i 'ssid|wifi|password|pool|token|device_url|nvs|stratum|https?://|wss?://|([0-9]{1,3}\.){3}[0-9]{1,3}|([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}|secret' \
  docs/parity/evidence/phase-17-live-http-api-and-static-evidence
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
| --- | --- | --- | --- |
| Route registration, package artifacts, and API compare evidence were enough to mark implementation status. [VERIFIED: docs/parity/checklist.md] | Verified live static/recovery/API/WebSocket claims require explicit `DEVICE_URL` and live artifacts. [VERIFIED: 17-CONTEXT.md] | Phase 17 context after Phase 16 verification on 2026-07-02. [VERIFIED: 17-CONTEXT.md] | Planner must separate implemented status from verified parity status. [VERIFIED: docs/adr/0012-parity-verification-evidence.md, 17-CONTEXT.md] |
| No-upgrade `/api/ws` and `/api/ws/live` checks could show route precedence. [VERIFIED: scripts/phase16-http-static-smoke.sh] | Frame-level WebSocket proof requires a real bounded WebSocket client. [VERIFIED: 17-CONTEXT.md] | Phase 17 context. [VERIFIED: 17-CONTEXT.md] | Add or generalize Node WebSocket capture instead of relying on `curl`. [VERIFIED: scripts/phase15-websocket-capture.mjs] |
| Phase 16 wrote blocked HTTP evidence because `DEVICE_URL` was absent. [VERIFIED: docs/parity/evidence/phase-16-current-commit-release-evidence-completion/http-static-recovery.md] | Phase 17 must either capture live evidence at an explicit URL or write blocked/pending evidence without scanning. [VERIFIED: 17-CONTEXT.md] | Phase 17 context. [VERIFIED: 17-CONTEXT.md] | Planning must include a blocked evidence path, not only a happy path. [VERIFIED: 17-CONTEXT.md] |

**Deprecated/outdated:**

- Treating `DEVICE_URL` absence as a soft skip is outdated for Phase 17; it must produce blocked evidence and keep rows below verified. [VERIFIED: 17-CONTEXT.md]
- Treating `POST /api/system/OTAWWW` public response as whole-`www` update parity is explicitly deferred. [VERIFIED: 17-CONTEXT.md, crates/bitaxe-api/src/update_plan.rs]
- Treating Node as unable to do built-in WebSocket client capture is outdated on this machine because Node v24.13.0 exposes `globalThis.WebSocket`, and Node docs describe the stable built-in client. [VERIFIED: local Node probe] [CITED: https://nodejs.org/learn/getting-started/websocket]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
| --- | --- | --- | --- |
| A1 | The research remains useful for about 30 days for stable repo-local tooling. [ASSUMED] | Metadata | Planner may need to re-run tool/version probes sooner if the repo, Node, ESP-IDF tooling, or hardware setup changes. [VERIFIED: local command probes] |

## Open Questions

1. **What explicit `DEVICE_URL` will the operator provide at execution time?** [VERIFIED: 17-CONTEXT.md]
   - What we know: hardware detection succeeded locally for one ESP32-S3 USB candidate. [VERIFIED: `just detect-ultra205`]
   - What's unclear: no reachable network origin was provided during research, and scanning is forbidden. [VERIFIED: 17-CONTEXT.md]
   - Recommendation: planner should include a blocked evidence path and require `--device-url` or explicit `DEVICE_URL` before live probes. [VERIFIED: 17-CONTEXT.md]

2. **Can `/api/ws` produce a raw log frame within the Phase 17 bounded window?** [VERIFIED: 17-CONTEXT.md]
   - What we know: `/api/ws/live` sends a connect frame; `/api/ws` starts from current log-buffer end and may time out without new log lines. [VERIFIED: firmware/bitaxe/src/http_api.rs, crates/bitaxe-api/src/logs.rs]
   - What's unclear: whether the live route sequence naturally generates a raw retained-log frame after `/api/ws` connects. [VERIFIED: firmware/bitaxe/src/log_buffer.rs, crates/bitaxe-api/src/logs.rs]
   - Recommendation: capture `/api/ws` separately; if it opens but produces no frame, record open/timeout and keep raw-log frame evidence pending. [VERIFIED: 17-CONTEXT.md]

3. **Should Phase 17 extend `release-evidence` validation for post-source docs commits?** [VERIFIED: tools/parity/src/release_evidence.rs]
   - What we know: current release-evidence post-source allowlist is Phase 16-specific. [VERIFIED: tools/parity/src/release_evidence.rs]
   - What's unclear: Phase 17 verification expectations do not require `release-evidence`, but exact package/flash identity still matters. [VERIFIED: .planning/ROADMAP.md, 17-CONTEXT.md]
   - Recommendation: planner should either avoid using `release-evidence` for Phase 17 final validation or add a narrow Phase 17 allowlist/test if that validator becomes part of the phase gate. [VERIFIED: tools/parity/src/release_evidence.rs]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
| --- | --- | --- | --- | --- |
| Bash | Shell helpers and sh_tests. [VERIFIED: scripts/BUILD.bazel] | yes [VERIFIED: `bash --version`] | GNU bash 3.2.57 [VERIFIED: `bash --version`] | None needed. [VERIFIED: scripts use bash shebangs] |
| `curl` | HTTP/static/API route probes. [VERIFIED: scripts/phase16-http-static-smoke.sh] | yes [VERIFIED: `curl --version`] | 8.7.1 [VERIFIED: `curl --version`] | Block HTTP evidence if unavailable. [VERIFIED: 17-CONTEXT.md] |
| Node.js | WebSocket frame capture. [VERIFIED: scripts/phase15-websocket-capture.mjs] | yes [VERIFIED: `node --version`] | v24.13.0, global `WebSocket` is a function [VERIFIED: local Node probe] | Add npm `ws` only if Node global WebSocket is absent and planner explicitly accepts a new dependency. [CITED: https://nodejs.org/learn/getting-started/websocket] |
| `espflash` | Board-info and flash/monitor backend. [VERIFIED: scripts/detect-ultra205.sh, tools/flash/src/main.rs] | yes [VERIFIED: `espflash --version`] | 4.0.1 [VERIFIED: `espflash --version`] | Missing `espflash` blocks hardware evidence; use `just bootstrap-esp` to install tooling. [VERIFIED: AGENTS.md] |
| `just` | Human command surface. [VERIFIED: Justfile] | yes [VERIFIED: `just --version`] | 1.48.0 [VERIFIED: `just --version`] | Direct Bazel commands exist but should not be the default user surface. [VERIFIED: Justfile, AGENTS.md] |
| Bazel | Canonical automation graph. [VERIFIED: MODULE.bazel, Justfile] | yes [VERIFIED: `bazel --version`] | 9.1.1 [VERIFIED: `bazel --version`] | Missing Bazel blocks repo-native tests/package; use `just doctor`. [VERIFIED: AGENTS.md] |
| Python 3 | Existing manifest-field helper code. [VERIFIED: scripts/phase16-http-static-smoke.sh] | yes [VERIFIED: `python3 --version`] | 3.14.4 [VERIFIED: `python3 --version`] | Prefer Node/Rust helper code for new non-trivial JSON/URL logic. [VERIFIED: standards/core/code-shape.md] |
| Ultra 205 USB candidate | Detector gate. [VERIFIED: AGENTS.md] | yes [VERIFIED: `just detect-ultra205`] | ESP32-S3 board-info succeeded on one likely port. [VERIFIED: `just detect-ultra205`] | If absent/multiple/failing, write blocked hardware evidence. [VERIFIED: AGENTS.md, 17-CONTEXT.md] |
| Explicit `DEVICE_URL` | Live HTTP/WebSocket probes. [VERIFIED: 17-CONTEXT.md] | not provided during research [VERIFIED: user prompt and local environment did not include explicit target] | none | No scan fallback; write blocked evidence until explicit origin is provided. [VERIFIED: 17-CONTEXT.md] |

**Missing dependencies with no fallback:**

- Explicit reachable `DEVICE_URL` is required for live route/WebSocket evidence; network scanning is forbidden. [VERIFIED: 17-CONTEXT.md]

**Missing dependencies with fallback:**

- None for local tooling; all core CLI tools probed as available. [VERIFIED: local command probes]

## Validation Architecture

### Test Framework

| Property | Value |
| --- | --- |
| Framework | Bazel `sh_test` for shell helpers, Bazel `rust_test`/Cargo tests for Rust changes, and Node `--check` for `.mjs` syntax. [VERIFIED: scripts/BUILD.bazel, tools/parity/BUILD.bazel, Cargo.toml] |
| Config file | `scripts/BUILD.bazel`, `Cargo.toml`, per-crate `BUILD.bazel`; Nyquist validation is enabled. [VERIFIED: .planning/config.json] |
| Quick run command | `bash -n scripts/phase17-live-http-api-smoke.sh && node --check scripts/phase17-websocket-capture.mjs && bazel test //scripts:phase17_live_http_api_smoke_test` after Wave 0 creates those targets. [VERIFIED: scripts/BUILD.bazel pattern] |
| Full suite command | `just test` plus `just parity` and `just verify-reference` for final phase verification. [VERIFIED: Justfile, 17-CONTEXT.md] |

### Phase Requirements -> Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
| --- | --- | --- | --- | --- |
| API-09 | Explicit target validation, route set coverage, static/recovery markers, and WebSocket proof split. [VERIFIED: 17-CONTEXT.md] | shell + Node unit-style helper tests; live hardware smoke. [VERIFIED: scripts/phase16-http-static-smoke-test.sh, scripts/phase15-controlled-mining-test.sh] | `bazel test //scripts:phase17_live_http_api_smoke_test` and bounded live helper command with `--device-url`. [VERIFIED: scripts/BUILD.bazel pattern] | no, Wave 0. [VERIFIED: `rg --files`] |
| REL-01 | Package/flash identity tied to SPIFFS/static/recovery live evidence. [VERIFIED: 17-CONTEXT.md] | workflow + live hardware smoke. [VERIFIED: Justfile] | `just package`, `just detect-ultra205`, `just flash-monitor board=205 port=<port> manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json evidence-dir=...`, then Phase 17 smoke. [VERIFIED: Justfile, AGENTS.md] | existing commands yes; Phase 17 ledger no. [VERIFIED: Justfile, `rg --files`] |
| REL-07 | Release docs cite exact commands, artifacts, non-claims, and recovery boundaries. [VERIFIED: 17-CONTEXT.md] | docs review + parity. [VERIFIED: docs/release/ultra-205.md, Justfile] | `just parity` and manual diff review of `docs/release/ultra-205.md`. [VERIFIED: Justfile] | docs exist; Phase 17 updates pending. [VERIFIED: docs/release/ultra-205.md] |
| EVD-05 | Evidence layering includes helper tests, package/flash, live HTTP/WebSocket, redaction, parity, and reference guard. [VERIFIED: 17-CONTEXT.md] | workflow + live hardware smoke + redaction review. [VERIFIED: AGENTS.md] | `just test`, `just parity`, `just verify-reference`, redaction scan command. [VERIFIED: Justfile, 17-CONTEXT.md] | existing commands yes; Phase 17 redaction file no. [VERIFIED: `rg --files`] |

### Sampling Rate

- **Per task commit:** Run helper-specific syntax/tests for changed helper files; run Rust checks only when Rust files change. [VERIFIED: AGENTS.md, standards/core/verification.md]
- **Per wave merge:** Run `just test` when helper/Rust/Bazel changes are complete. [VERIFIED: Justfile]
- **Phase gate:** Run `just detect-ultra205`, fresh package/flash evidence, explicit `DEVICE_URL` smoke, WebSocket capture, redaction review, `just parity`, and `just verify-reference`. [VERIFIED: 17-CONTEXT.md]

### Wave 0 Gaps

- [ ] `scripts/phase17-live-http-api-smoke.sh` - Phase-owned wrapper with origin-only target validation, stale identity checks, and D-08 route list. [VERIFIED: 17-CONTEXT.md]
- [ ] `scripts/phase17-live-http-api-smoke-test.sh` - Tests missing URL, invalid scheme, userinfo/path/query/fragment rejection, successful fake route coverage, WebSocket no-upgrade not overclaimed, stale package/flash blocking, and redaction. [VERIFIED: scripts/phase16-http-static-smoke-test.sh, 17-CONTEXT.md]
- [ ] `scripts/phase17-websocket-capture.mjs` - Generalized allowlisted capture for `/api/ws/live` and `/api/ws`, with bounded duration/max frames and redaction. [VERIFIED: scripts/phase15-websocket-capture.mjs, 17-CONTEXT.md]
- [ ] `scripts/BUILD.bazel` entries for the Phase 17 helper(s) and tests. [VERIFIED: scripts/BUILD.bazel]
- [ ] `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/redaction-review.md` template with artifact matrix and absent-artifact handling. [VERIFIED: 17-CONTEXT.md]

## Security Domain

### Applicable ASVS Categories

OWASP ASVS is a web application security verification standard; this section maps the template categories to Phase 17 controls. [CITED: https://owasp.org/www-project-application-security-verification-standard/]

| ASVS Category | Applies | Standard Control |
| --- | --- | --- |
| V2 Authentication | no new auth implementation. [VERIFIED: 17-CONTEXT.md] | Do not add credentials or auth bypasses; record only public/admin route behavior already implemented. [VERIFIED: firmware/bitaxe/src/http_api.rs] |
| V3 Session Management | no browser session state changed. [VERIFIED: 17-CONTEXT.md] | Keep WebSocket captures bounded and avoid persistent clients after evidence collection. [VERIFIED: 17-CONTEXT.md] |
| V4 Access Control | yes, live API/WebSocket routes pass through private-network/origin gates in firmware. [VERIFIED: crates/bitaxe-api/src/route_shell.rs, firmware/bitaxe/src/http_api.rs] | Use existing access gate; do not weaken it for evidence. [VERIFIED: crates/bitaxe-api/src/route_shell.rs] |
| V5 Input Validation | yes, helper inputs and route responses are untrusted. [VERIFIED: 17-CONTEXT.md] | Parse/validate `DEVICE_URL` at boundary; allowlist WebSocket paths; reject userinfo/path/query/fragment. [VERIFIED: 17-CONTEXT.md] |
| V6 Cryptography | no new cryptography. [VERIFIED: 17-CONTEXT.md] | Do not add crypto; use existing HTTP/HTTPS origin scheme as operator-provided. [VERIFIED: 17-CONTEXT.md] |
| V8 Logging and Data Protection | yes, evidence contains live device outputs. [VERIFIED: 17-CONTEXT.md] | Redact `DEVICE_URL`, private endpoints, IPs, MACs, Wi-Fi/pool/API/NVS secrets, and cite only reviewed artifacts. [VERIFIED: 17-CONTEXT.md] |

### Known Threat Patterns for Phase 17

| Pattern | STRIDE | Standard Mitigation |
| --- | --- | --- |
| Accidental device targeting through discovery or inferred URL. [VERIFIED: 17-CONTEXT.md] | Spoofing / Tampering | Explicit origin-only `DEVICE_URL`; no scan, mDNS, ARP, router, or serial-log inference. [VERIFIED: 17-CONTEXT.md] |
| Evidence leaks private IP, MAC, Wi-Fi, pool, API token, or terminal secret. [VERIFIED: 17-CONTEXT.md] | Information Disclosure | Redacted snippets, artifact matrix, redaction scan, and absent-artifact marking. [VERIFIED: 17-CONTEXT.md] |
| Overclaiming OTA/WebSocket behavior from weak route evidence. [VERIFIED: 17-CONTEXT.md] | Repudiation / Elevation of Privilege | Separate route presence, no-upgrade coexistence, frame proof, and non-claims in ledger/checklist. [VERIFIED: 17-CONTEXT.md] |
| Long-running WebSocket capture or mining/soak behavior sneaks into Phase 17. [VERIFIED: 17-CONTEXT.md] | Denial of Service / Scope Creep | Duration and max-frame bounds; no pool credentials, mining smoke, or soak. [VERIFIED: 17-CONTEXT.md] |

## Sources

### Primary (HIGH confidence)

- `.planning/phases/17-live-http-api-and-static-evidence/17-CONTEXT.md` - locked decisions, probe set, WebSocket proof depth, evidence/redaction, checklist promotion, deferred scope. [VERIFIED: file read]
- `.planning/REQUIREMENTS.md` - API-09, REL-01, REL-07, EVD-05 and Phase 17-21 traceability note. [VERIFIED: file read]
- `.planning/STATE.md` - Phase 16 status, prior decisions, current blockers. [VERIFIED: file read]
- `AGENTS.md`, `AGENTS.bright-builds.md`, `standards/core/architecture.md`, `standards/core/code-shape.md`, `standards/core/verification.md`, `standards/core/testing.md`, `standards/languages/rust.md`, `standards/languages/typescript-javascript.md`, `standards/core/operability.md` - repo and standards constraints. [VERIFIED: file reads]
- `scripts/phase16-http-static-smoke.sh` and `scripts/phase16-http-static-smoke-test.sh` - reusable HTTP route/redaction helper and tests. [VERIFIED: file reads]
- `scripts/phase15-websocket-capture.mjs` and `scripts/phase15-controlled-mining-test.sh` - bounded Node WebSocket capture and tests. [VERIFIED: file reads]
- `firmware/bitaxe/src/http_api.rs`, `firmware/bitaxe/src/static_files.rs`, `firmware/bitaxe/src/websocket_api.rs`, `firmware/bitaxe/src/log_buffer.rs`, `firmware/bitaxe/src/ota_update.rs` - firmware route/static/WebSocket/log/OTA behavior. [VERIFIED: file reads]
- `crates/bitaxe-api/src/route_shell.rs`, `static_plan.rs`, `update_plan.rs`, `telemetry.rs`, `logs.rs`, `websocket_state.rs` - pure route/static/update/WebSocket contracts. [VERIFIED: file reads]
- `Justfile`, `scripts/BUILD.bazel`, `MODULE.bazel`, `Cargo.toml` - command surface, helper test wiring, Bazel/Rust versions. [VERIFIED: file reads]
- Local probes: `just detect-ultra205`, `node --version`, `curl --version`, `espflash --version`, `bazel --version`, `just --version`, `python3 --version`, `git rev-parse HEAD`, `git -C reference/esp-miner rev-parse HEAD`. [VERIFIED: command output]

### Secondary (MEDIUM confidence)

- Node.js official WebSocket client guide - built-in WebSocket client and no need for external client libraries on current Node. [CITED: https://nodejs.org/learn/getting-started/websocket]
- Espressif ESP-IDF HTTP server docs - HTTP server provides WebSocket support and requires a WebSocket client for interaction. [CITED: https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/protocols/esp_http_server.html]
- Espressif `ws_echo_server` example README - WebSocket URI handlers are registered with `is_websocket` enabled and require care around handshake/frame handling. [CITED: https://github.com/espressif/esp-idf/blob/master/examples/protocols/http_server/ws_echo_server/README.md]
- OWASP ASVS project page - ASVS as a basis for web application security controls and verification. [CITED: https://owasp.org/www-project-application-security-verification-standard/]

### Tertiary (LOW confidence)

- None. [VERIFIED: no LOW-confidence sources used]

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH - all recommended tools are existing repo patterns or locally version-probed; no new package is recommended. [VERIFIED: local command probes, scripts/BUILD.bazel, scripts/phase16-http-static-smoke.sh, scripts/phase15-websocket-capture.mjs]
- Architecture: HIGH - route/static/WebSocket behavior is visible in local pure crates and firmware adapters, and Phase 17 decisions lock evidence boundaries. [VERIFIED: crates/bitaxe-api/src/route_shell.rs, firmware/bitaxe/src/http_api.rs, 17-CONTEXT.md]
- Pitfalls: HIGH for evidence overclaim/redaction/stale identity; MEDIUM for raw `/api/ws` frame availability because live timing depends on whether new retained log lines occur during the bounded capture. [VERIFIED: 17-CONTEXT.md, crates/bitaxe-api/src/logs.rs]
- Environment: MEDIUM-HIGH - local CLI tools and USB detector gate are available, but explicit reachable `DEVICE_URL` was not provided during research. [VERIFIED: local command probes, 17-CONTEXT.md]

**Research date:** 2026-07-02 [VERIFIED: system/developer date]
**Valid until:** 2026-08-01 for repo-local patterns; re-check tool versions and `DEVICE_URL`/hardware availability at execution time. [VERIFIED: local command probes; ASSUMED: 30-day validity window for stable repo-local tooling]
