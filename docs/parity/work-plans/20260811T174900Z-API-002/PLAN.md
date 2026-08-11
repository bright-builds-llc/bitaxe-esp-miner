# Parity work plan

- Run ID: `20260811T174900Z-API-002`
- Parity row: `API-002`
- Initial status: `implemented`
- Source commit: `84b90c9e677b4def1d0ab7508e2b8e64dd08c617`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api002-system-info-contract`
- Continues plan: `docs/parity/work-plans/20260811T164522Z-API-002/PLAN.md`

## Selection

The branch is clean, synchronized, and has no open parity plan. The selector
ranks `CFG-001` first, but its purpose-bound 485 MHz/1200 mV soak lineage is
closed at a repeated network-correlation boundary and has no unchanged retry.
`CFG-006` requires unavailable non-205 hardware. `NET-001` through `NET-003`
still lack qualified reconnect, provisioning-client, live-scan, and IPv6
environment contracts. `ASIC-002` through `ASIC-005`, `ASIC-007`, `STR-001`,
`STR-006`, and `STR-007` depend on safety-controlled mining evidence whose last
targeted attempt repeated its terminal continuity signature and cannot be
retried unchanged.

`API-002` is the first actionable row. Its prior exact-package attempt produced
49 aggregate `main` task stack-overflow detections after boot readiness began.
Commit `84b90c9e677b4def1d0ab7508e2b8e64dd08c617` removes full operator-snapshot
construction from startup readiness, bounds the inline API snapshot, and adds a
real-firmware disassembly gate that measures the replacement frame at 480 bytes
under a 1 KiB limit. The complete software gate and ESP32-S3 build passed. This
is a targeted, regression-backed change at the exact failed boundary, so one
fresh bounded hardware attempt is eligible under the retry policy.

## Scope and non-scope

Build and admit one clean exact package containing the startup-stack fix, then
run one detector-gated passive system-info capture. The capture may perform one
factory-package flash, derive one trusted same-session origin from private
serial evidence, issue passive same-origin HTTP/WebSocket/retained-log reads,
and publish only the existing aggregate `bitaxe-system-info-evidence-v1`
projection after every identity, schema, coherence, safety, cleanup, and
redaction condition passes.

No settings mutation, restart request, mining profile, pool credential, ASIC
work, voltage, frequency, fan, thermal or power control, OTA, erase, raw write,
network discovery, fault injection, foreign-process termination, direct UART,
pins, pads, headers, probes, jumpers, soldering, or injected signals is in
scope. The package flash's normal USB reset and re-enumeration are permitted.
The workflow may perform at most one exact-package recovery flash only after an
initial flash effect if terminal safe-state or USB cleanup cannot otherwise be
confirmed. Conditional block values remain a structural-test claim; the live
attempt must keep block notification and mining inactive.

Private raw response, configuration, hostname, origin, port, USB/network or
process identity, credential, serial, and trace material must remain beneath
ignored mode-0700 roots in mode-0600 files. Public output may contain only
closed categories, booleans, counts, field names and types, and cryptographic
source, reference, package, workflow, and schema identities.

## Implementation

- [ ] Commit and push this immutable retry plan and updated active task before
      building or interacting with hardware.
- [ ] Re-run the complete software and real-firmware gates against the exact
      clean plan commit, build the schema-v3 package, and admit its source,
      reference, ELF digest, factory layout, and artifacts.
- [ ] Run exactly one detector and, only on admission, exactly one passive
      `attempt-002` capture through the existing aggregate workflow.
- [ ] Validate the public projection independently and promote only `API-002`
      if every acceptance condition passes; otherwise preserve the typed first
      failure, close without promotion, and do not retry this plan.

## Verification and promotion

Before hardware, run focused stack-budget, source-ownership, API, automation,
and evidence-contract tests; a real `just build`; then, in order:

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
11. continuation-aware selector, immutable-plan, task-uniqueness,
    sensitive-output, reference-cleanliness, and diff checks

After the immutable plan is pushed and a clean exact package is admitted, the
only authorized hardware sequence is:

1. `test ! -e scratch/api002-system-info/wrapper-002 && (umask 077; mkdir -m 700 -p scratch/api002-system-info/wrapper-002 && just detect-ultra205 > scratch/api002-system-info/wrapper-002/detector.stdout 2>&1)`
2. Only after command 1 succeeds:
   `test ! -e scratch/api002-system-info/attempt-002 && test ! -e docs/parity/evidence/api002-system-info/system-info-projection.json && (umask 077; just capture-system-info-evidence --private-root scratch/api002-system-info/attempt-002 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/api002-system-info/wrapper-002/detector.stdout --projection docs/parity/evidence/api002-system-info/system-info-projection.json --capture-timeout-seconds 360 > scratch/api002-system-info/wrapper-002/capture.stdout 2> scratch/api002-system-info/wrapper-002/capture.stderr)`

The ignored wrapper and attempt roots must be absent before launch, mode 0700,
and contain only mode-0600 files. Preserve the earliest typed failure. Map
non-ready hardware to `hardware_blocked`, malformed evidence to
`evidence_invalid`, child timeout to `timeout`, and launch failure to
`process_failed`. Record recovery separately, release every owned resource, and
never retry this plan.

Promotion requires exact clean source and pinned reference identity, one
detector-admitted Ultra 205, exact runtime build identity, stable boot without
panic or stack overflow, one coherent substantive HTTP/WebSocket/retained-log
snapshot, all 87 unconditional field names and types, correct absence of the
seven inactive block fields, confirmed-setting projection, healthy aggregate
identity, mining and hardware control disabled, complete cleanup, private file
modes, independent evidence validation, and passed redaction. Only `API-002`
may transition to `verified`; otherwise create a truthful non-verifying closure
and stop.
