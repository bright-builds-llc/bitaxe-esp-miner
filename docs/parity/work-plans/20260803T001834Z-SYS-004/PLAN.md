# Parity work plan

- Run ID: `20260803T001834Z-SYS-004`
- Parity row: `SYS-004`
- Initial status: `implemented`
- Source commit: `6c377f4d7439d4cedb1162899f93647577c1a478`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-sys004-version-reporting`

## Selection

The clean synchronized `main` worktree and corrected
`bazel run //tools/parity:report -- next-item --format json` reported no open
plan and selected `SYS-004` first at `implemented`; no earlier candidate was
skipped. The prior implementation plan proved the canonical API projection but
left two exact verification gates: a live response from the exact current
package and an explicit static-asset version-semantics decision.

Source inspection found that both gates expose one concrete defect. Upstream
reads `axeOSVersion` from the flashed SPIFFS `/version.txt` and compares it with
the application descriptor version. Rust instead reads the checked-in
`assets/release.json` name `bitaxe-rust-fallback-ui`; the packaged SPIFFS image
contains no `version.txt`. A fresh device run would therefore prove a semantic
mismatch rather than close the row.

This plan follows the repository task/hardware/privacy guidance,
`AGENTS.bright-builds.md`, the architecture, code-shape, verification, testing,
and Rust standards, and the active lessons requiring exact-boundary evidence,
qualified transport reuse, and new information before a hardware retry.

## Scope and non-scope

Scope is limited to `SYS-004` version reporting:

- stage a package-owned `version.txt` containing the canonical build label in
  the generated SPIFFS image without modifying the source asset directory;
- read that mounted `/www/version.txt` through the firmware platform adapter
  so `axeOSVersion` identifies the installed static payload;
- add a pure, typed classifier that binds the private exact-package live
  `/api/system/info` version/provenance fields to the admitted manifest and
  emits only a commit-safe projection; and
- make one detector-gated Ultra 205 attempt using the qualified Phase 36
  broker to flash the exact package and capture passive serial, HTTP, and
  WebSocket observations.

No mining, pool credential use, voltage/frequency/fan control, OTA, OTAWWW,
rollback, fault injection, erase-flash, arbitrary raw write, discovery,
non-205 hardware, direct UART, pins, pads, headers, GPIO, probes, jumpers,
soldering, or injected signals is authorized. No other checklist row may
change. The broader Phase 36 operator-snapshot and runtime-health claims remain
non-claims even if this version-only projection passes.

## Implementation

- [x] Generate a build-label `version.txt` in an ephemeral package staging
      tree and read the mounted file through the firmware platform adapter.
- [x] Add focused package and platform/version regressions, including missing,
      malformed, stale, mismatched, and private-input cases.
- [x] Add a version-only classifier that joins exact manifest identity to the
      private Phase 36 API/WebSocket/serial capture and emits a closed,
      redaction-safe projection.
- [x] Commit and push the complete software fix before packaging or hardware
      use, then run at most one authorized hardware attempt.
- [x] Create `RESULT.md` and promote only `SYS-004` if every exact-package live
      version binding, evidence-integrity, and repository gate passes.

## Verification and promotion

Focused software verification will cover the package script, package manifest,
platform version parsing, API wire mapping, version projection, Phase 36
capture join, and their Bazel targets. Mandatory verification is the ordered
Rust sequence, `bun scripts/bright-builds-check.ts all`, `just test`, `just
parity`, `just parity-progress`, `just verify-redaction`, `just
verify-reference`, and `git diff --check`.

The only permitted hardware workflow is:

1. `just package`
2. `just detect-ultra205`
3. `just phase36-substantive-evidence mode=preflight board=205 private-parent=scratch/sys004-version-reporting/attempt-001 attempt-handle-file=scratch/sys004-version-reporting/attempt-001/handle.json candidate-output=scratch/sys004-version-reporting/attempt-001/candidate.json capture-timeout-seconds=360 package-manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`
4. `just phase36-substantive-evidence mode=hardware board=205 private-parent=scratch/sys004-version-reporting/attempt-001 attempt-handle-file=scratch/sys004-version-reporting/attempt-001/handle.json candidate-output=scratch/sys004-version-reporting/attempt-001/candidate.json capture-timeout-seconds=360 wifi-credentials=wifi-credentials.json`
5. `bazel run //tools/parity:report -- project-sys004-version-evidence --private-parent scratch/sys004-version-reporting/attempt-001 --attempt-handle-file scratch/sys004-version-reporting/attempt-001/handle.json --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --output docs/parity/evidence/sys004-version-reporting/version-projection.json`

The Phase 36 broker owns its one internal detector admission, exact-package
factory flash, passive receive-only serial observation, typed HTTP/WebSocket
reads, same-package recovery when required, and cleanup. Its 420-second
per-effect wall-clock bound and 360-second capture minimum remain unchanged.
The ignored private parent must be mode `0700`; private artifacts must be mode
`0600`; raw device, network, USB, credential, and response material must never
enter the committed projection.

Exactly one hardware attempt is authorized after a clean pushed software
source commit and fresh package. There is no unchanged retry. A later ordinal
requires a separate targeted regression-backed fix or objectively changed
non-invasive boundary. Accepted outcomes are `complete`,
`stop_repeated_boundary`, `stop_hardware_blocker`, `stop_authority_boundary`,
or `stop_impossible_contract`.

Promotion to `verified` requires one detector-admitted board 205, exact source,
reference, manifest and ELF identity, safe boot, a complete live HTTP response
and an identical-version WebSocket projection from the same boot with an
equal-or-later positive revision, `version` and
`axeOSVersion` equal to the manifest build label, every extended provenance
field equal to the manifest, `idfVersion` equal to the manifest ESP-IDF
version, static-package/source tests proving `version.txt` ownership, valid
private seals and commit-safe projection, cleanup, and every repository gate.
Any missing or contradictory fact leaves `SYS-004` at `implemented`.
