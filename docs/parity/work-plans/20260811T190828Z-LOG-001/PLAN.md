# Parity work plan

- Run ID: `20260811T190828Z-LOG-001`
- Parity row: `LOG-001`
- Initial status: `implemented`
- Source commit: `1714c70fdc5dd94315b17b743d2385ab1ddbeccf`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-log001-live-retained-stream`

## Selection

The branch is clean, synchronized with `origin/main`, and the deterministic
selector reports no open plan. `CFG-001` is closed at a repeated
network-correlation boundary with no unchanged safety-controlled soak retry.
`CFG-006` requires unavailable non-205 hardware. `NET-001` through `NET-003`
require controlled access-point failure/recovery, provisioning-client, scan,
or IPv6 environments that are not qualified by a repository contract.
`ASIC-002` through `ASIC-005`, `ASIC-007`, `STR-001`, `STR-006`, and `STR-007`
depend on safety-controlled mining evidence whose last targeted attempt
repeated its terminal continuity signature.

`API-009` cannot prove its complete command-effect set without active mining,
a physical identify observation, and a live block-notification state.
`PWR-001` through `PWR-003`, `PWR-005`, `PWR-006`, `THR-001` through
`THR-003`, and `SELF-001` require qualified sensors, actuation, or fault
stimulus. `IO-001` requires controlled transient bus faults, and `IO-002`
requires an independent calibrated voltage reference. `UI-001` and `UI-002`
require a trusted visual capture, `UI-003` requires a recorded physical input,
and `BAP-002` requires a compatible accessory. `UI-004` retains real mutation,
upload, and responsive operator-UAT gaps whose effects are broader than the
current passive environment.

`LOG-001` is therefore the first actionable row. Its pure retention, download,
and raw-stream rules already have unit and API-comparison evidence, while one
exact-package Ultra 205 can safely prove the remaining device delivery seam by
correlating a newly appended raw WebSocket connection marker with retained log
downloads from the same admitted boot.

## Scope and non-scope

Add a typed aggregate-only `bitaxe-log-buffer-evidence-v1` workflow. It will
flash one exact clean Ultra 205 package, derive one trusted same-origin target
from the admitted monitor session, verify the runtime package and passive safe
state, download the retained log with exact upstream-compatible headers,
connect once to raw `/api/ws`, capture its newly emitted plain-text connection
marker, and download the retained log again. The final retained body must have
the complete baseline as an exact prefix and exactly one additional connection
marker; the WebSocket frame must be plain text rather than a JSON envelope.

All log bodies, frames, origins, ports, USB and network identities, Wi-Fi
values, credentials, and process traces remain under the protected private
root. The public projection contains only closed schema/provenance fields,
cryptographic digests, bounded byte/marker counts, exact-header and correlation
booleans, safe-state facts, cleanup, and redaction status.

This work does not mutate settings, restart after boot, mine, initialize or
submit ASIC work, actuate voltage, frequency, fan, thermal, or power controls,
scan or discover networks, update firmware through OTA, erase flash, write raw
partitions, inject faults, terminate foreign processes, use direct UART, or
manipulate pins. The exact-package factory flash may install the existing
owner-supplied Wi-Fi credentials and `mineonboot=false`; no pool credentials
are read. Log persistence across reset, maximum-capacity wraparound under live
load, long-duration streaming, multiple simultaneous clients, and other boards
remain separate non-claims.

## Implementation

- [ ] Add the Rust-owned evidence schema, validator, command identity, and
      synchronized TypeScript contract for `capture-log-buffer-evidence`.
- [ ] Add the private-first exact-package capture, same-boot identity checks,
      bounded retained-text responses with selected headers, plain-text raw
      WebSocket capture, exact append correlation, cleanup, and no-clobber
      publication.
- [ ] Add behavior-focused unit, failure-category, privacy, mode, process-
      boundary, header, append-correlation, and plain-text-frame regressions.
- [ ] Run all mandatory software gates, push the implementation, freeze its
      exact package, and spend at most one detector plus conditional capture.
- [ ] Independently validate the closed projection and promote only `LOG-001`
      when every acceptance condition passes.

## Verification and promotion

Run focused contracts, HTTP/WebSocket, automation, firmware log-buffer, and
real-child-process tests followed by, in order:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`
9. `just verify-redaction`
10. `just verify-reference`
11. selector, immutable-plan, task-uniqueness, sensitive-output,
    reference-cleanliness, and diff checks

After a clean implementation commit is pushed, run exactly these bounded
commands:

1. `just package`
2. `test ! -e scratch/log001-retained-stream/wrapper-001 && (umask 077; mkdir -m 700 -p scratch/log001-retained-stream/wrapper-001 && just detect-ultra205 > scratch/log001-retained-stream/wrapper-001/detector.stdout 2>&1)`
3. Only after command 2 succeeds:
   `test ! -e scratch/log001-retained-stream/attempt-001 && test ! -e docs/parity/evidence/log001-retained-stream/log-buffer-projection.json && (umask 077; just capture-log-buffer-evidence --private-root scratch/log001-retained-stream/attempt-001 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/log001-retained-stream/wrapper-001/detector.stdout --projection docs/parity/evidence/log001-retained-stream/log-buffer-projection.json --capture-timeout-seconds 240 > scratch/log001-retained-stream/wrapper-001/capture.stdout 2> scratch/log001-retained-stream/wrapper-001/capture.stderr)`

The wrapper and attempt roots must be absent before use, mode `0700`, and
contain only mode-`0600` files. Detector failure stops before writes. The
capture permits one exact-package factory flash and its normal USB reset,
bounded receive-only USB and same-origin HTTP/WebSocket observation, and no
second flash or recovery effect. Preserve the earliest typed failure. Map
flash/session readiness failures to `hardware_blocked`, malformed or missing
evidence to `evidence_invalid`, child timeout to `timeout`, and launch/child
failure to `process_failed`. Exactly one capture is permitted; no unchanged
retry is authorized.

Promotion requires exact source/reference/package identity, one admitted board
205, one safe same-boot origin, exact `text/plain` and attachment headers on
both downloads, bounded bodies, one plain-text connection frame, an exact
baseline prefix, exactly one newly retained matching marker, disabled mining
and hardware control, complete socket/process/USB cleanup, correct private
modes, an independently validated redacted projection, and every mandatory
gate passing. Otherwise withhold `RESULT.md` and public evidence, create a
typed non-verified closure, keep `LOG-001` at `implemented`, and stop without
retry.
