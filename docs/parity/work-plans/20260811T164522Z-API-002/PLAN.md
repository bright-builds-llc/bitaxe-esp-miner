# Parity work plan

- Run ID: `20260811T164522Z-API-002`
- Parity row: `API-002`
- Initial status: `implemented`
- Source commit: `705e44b2c0b151408fff681195f3bb1dcd9a4854`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api002-system-info-contract`

## Selection

The clean synchronized branch has no open parity plan. The deterministic
selector ranks `CFG-001` first, but its purpose-bound 485 MHz/1200 mV soak
lineage is closed at a repeated network-correlation boundary and has no
unchanged retry. `CFG-006` requires unavailable non-205 hardware. `NET-001`
through `NET-003` still lack qualified reconnect, provisioning-client, live
scan, and IPv6 environment contracts. `ASIC-002` through `ASIC-005`,
`ASIC-007`, `STR-001`, `STR-006`, and `STR-007` depend on safety-controlled
mining evidence whose last targeted attempt repeated its terminal continuity
signature and cannot be retried unchanged.

`API-002` is the first actionable row. The live route and a useful typed slice
exist, but a fresh schema audit found that the pinned upstream `SystemInfo`
contract declares 94 required fields while the current safe Rust fixture has
83 fields and omits 37 upstream names. The omissions are dominated by
confirmed settings plus CPU/overheat state and conditionally emitted block
details. This is an implementation gap that can be closed and tested before a
single passive detector-gated hardware capture.

## Scope and non-scope

Complete the Ultra 205 `/api/system/info` wire contract against the pinned
`system_api_json.c` writer and OpenAPI schema. Add typed confirmed-setting,
platform, and conditional block-detail inputs; preserve the upstream mixed
boolean/number encodings and conditional absence behavior; retain the existing
additive build, observation-truth, runtime-health, mining-state, and operator-
snapshot fields; and prevent secret-bearing values from Debug, logs, errors,
committed fixtures, and public evidence.

Add a typed `capture-system-info-evidence` workflow that performs one exact-
package flash-monitor, derives exactly one trusted origin from that private
capture, issues passive same-origin system-info and corroborating live/retained
observations, validates exact package and coherent snapshot identity, and emits
only an aggregate field/type projection. The public artifact may name schema
fields and closed type categories, counts, booleans, and cryptographic source,
reference, package, workflow, and schema identities. It must never expose raw
field values, credentials, pool configuration, hostnames, origins, ports, USB
or network identities, HTTP bodies, WebSocket frames, serial, or process traces.

No settings mutation, mining profile, pool credential, ASIC work, voltage,
frequency, fan, thermal, power control, restart, OTA, erase, raw write, network
discovery, fault injection, foreign-process termination, direct UART, pins,
pads, headers, GPIO, probes, jumpers, soldering, or injected signals is in
scope. Conditional block-found values may be proven structurally in pure tests;
the hardware attempt must keep the notification state inactive and claims no
live block event, mining statistics history, repeated capture longevity,
browser rendering, other board, or release behavior.

## Implementation

- [ ] Derive one versioned exhaustive system-info field contract from the
      pinned OpenAPI and writer behavior, including exact type and conditional
      presence rules, and make fixture/API comparison tests fail on drift.
- [ ] Deepen the pure snapshot and wire DTOs so every upstream field is owned
      by a typed input, with custom redaction for secret-bearing configuration
      and no placeholder assertion of unavailable runtime truth.
- [ ] Populate the contract from the atomically confirmed NVS snapshot and
      firmware-owned platform/runtime state without logging or retaining raw
      sensitive values outside the response transaction.
- [ ] Add the typed capture command, real-child/process tests, private artifact
      mode checks, recovery precedence, aggregate-only public projection, and
      semantic redaction coverage.
- [ ] Build and admit one clean exact schema-v3 package from the pushed
      implementation commit, then run one detector and one conditional passive
      attempt-001 capture.

## Verification and promotion

Before hardware, run focused config/API/automation/parity tests, the real
ESP32-S3 firmware build, and then:

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
11. selector, immutable-plan, task, sensitive-output, and diff checks

After a clean implementation commit is pushed, build `just package` and admit
the exact manifest. The only authorized hardware sequence is:

1. `test ! -e scratch/api002-system-info/wrapper-001 && (umask 077; mkdir -m 700 -p scratch/api002-system-info/wrapper-001 && just detect-ultra205 > scratch/api002-system-info/wrapper-001/detector.stdout 2>&1)`
2. Only after command 1 succeeds:
   `test ! -e scratch/api002-system-info/attempt-001 && test ! -e docs/parity/evidence/api002-system-info/system-info-projection.json && (umask 077; just capture-system-info-evidence --private-root scratch/api002-system-info/attempt-001 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/api002-system-info/wrapper-001/detector.stdout --projection docs/parity/evidence/api002-system-info/system-info-projection.json --capture-timeout-seconds 360 > scratch/api002-system-info/wrapper-001/capture.stdout 2> scratch/api002-system-info/wrapper-001/capture.stderr)`

The ignored wrapper and attempt roots must be absent, mode 0700, and contain
only mode-0600 artifacts. The workflow owns cleanup and may perform at most one
exact-package recovery flash only after its initial flash effect if terminal
safe-state or USB cleanup cannot otherwise be confirmed. Preserve the earliest
typed failure; map non-ready hardware to `hardware_blocked`, malformed evidence
to `evidence_invalid`, child timeout to `timeout`, and launch failure to
`process_failed`. Do not retry.

Promotion requires `bitaxe-system-info-evidence-v1` to bind exact clean source
and reference identities, board 205, one detector-admitted device, exact build
runtime identity, one coherent HTTP/WebSocket/retained-log boot, the complete
versioned field contract with all unconditional names and types present,
correct all-or-none inactive block-field behavior, confirmed persisted-setting
projection, healthy aggregate identity, disabled mining and hardware control,
complete cleanup, private modes, and passed redaction. Only `API-002` may move
to `verified`; otherwise create a truthful non-verified closure or checkpoint
without weakening the contract or spending another attempt.
