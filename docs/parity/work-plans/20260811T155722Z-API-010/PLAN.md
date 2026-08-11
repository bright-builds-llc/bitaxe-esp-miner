# Parity work plan

- Run ID: `20260811T155722Z-API-010`
- Parity row: `API-010`
- Initial status: `implemented`
- Source commit: `e9b775166e7f93c2933fef8694204aaaaabde02f`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api010-live-theme-durability-attempt-012`
- Continues plan: `docs/parity/work-plans/20260811T150224Z-API-010/PLAN.md`
- Progress basis: `docs/parity/work-plans/20260811T151310Z-API-010/RESULT.md`

## Selection

The clean synchronized branch has no open parity plan. The deterministic
selector lists `CFG-001` first, but live use of its voltage and frequency
defaults remains behind unfulfilled safety-control evidence. `CFG-006` requires
non-205 hardware that is unavailable. `NET-001` through `NET-003` require
qualified Wi-Fi fault/provisioning/scan/IPv6 environment contracts that do not
yet exist. `ASIC-002` through `ASIC-005`, `ASIC-007`, `STR-001`, `STR-006`, and
`STR-007` depend on safety-controlled mining work whose current bounded soak is
closed at a repeated network-correlation boundary. `API-002` still lacks full
field-level live production-statistics evidence; `API-003` lacks a complete
safe broad-PATCH hardware capture; and `API-009` retains mining, restart, and
display-effect subclaims beyond the present safe contract.

`API-010` is the first actionable row. Its typed route, persistence, device
session, restoration, and redaction workflows are implemented. Attempt 009
stopped before flashing at bootloader synchronization, while the linked
attempt-011 result now proves a clean exact-package flash and 360 seconds of
stable trusted runtime on the same available Ultra 205. That is an objective
boundary change and admits one fresh theme-durability attempt without changing
the existing workflow.

## Scope and non-scope

Run one detector-gated `verify-theme-durability` transaction against a clean
package built from the pushed plan commit. The transaction may perform one
exact-package flash with the ignored local Wi-Fi credential input, read the
original non-secret theme, POST one generated alternate theme, confirm
immediate readback, request one normal software restart, prove same-device
exact-build recovery at boot ordinal `N+1`, confirm persisted theme equality,
restore the original theme, and confirm restoration and cleanup. The built-in
exact-package recovery flash remains available only if normal restoration
cannot be confirmed.

No mining profile or pool credential may be read. Mining, ASIC work,
voltage/frequency/fan/thermal/power control, Wi-Fi or hostname mutation, network
discovery, OTA, erase-flash, arbitrary raw writes, fault injection, foreign
process termination, direct UART, pins, pads, headers, GPIO, probes, jumpers,
soldering, and injected signals are prohibited. Installed AxeOS browser
behavior remains a non-claim because this transaction exercises the HTTP theme
surface, persistence, and normal restart only.

## Implementation

- [ ] Re-run the focused theme-durability, device-session, CLI, and semantic
      redaction regressions against current HEAD; change production code only
      if a current regression exposes a real blocker.
- [ ] Build and admit one clean exact schema-v3 Ultra 205 package after the
      immutable plan commit is pushed.
- [ ] Run exactly one protected detector and, only after it succeeds, one
      protected attempt-012 theme-durability capture.
- [ ] Publish the redacted v1 projection and promote only `API-010` if every
      acceptance fact passes; otherwise withhold evidence, preserve the first
      typed failure, close without retry, and keep `implemented`.

## Verification and promotion

Before hardware, run focused automation and device-session tests, the canonical
firmware package, then the ordered repository gate:

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
11. selector, immutable-plan, sensitive-output, and diff checks

The only authorized hardware sequence is:

1. `just package`
2. `test ! -e scratch/api010-theme-durability/wrapper-012 && (umask 077; mkdir -m 700 -p scratch/api010-theme-durability/wrapper-012 && just detect-ultra205 > scratch/api010-theme-durability/wrapper-012/detector.stdout 2>&1)`
3. Only after command 2 succeeds:
   `test ! -e scratch/api010-theme-durability/attempt-012 && test ! -e docs/parity/evidence/api010-theme-durability/theme-durability-projection.json && (umask 077; just verify-theme-durability --private-root scratch/api010-theme-durability/attempt-012 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/api010-theme-durability/wrapper-012/detector.stdout --projection docs/parity/evidence/api010-theme-durability/theme-durability-projection.json --capture-timeout-seconds 360 > scratch/api010-theme-durability/wrapper-012/verify.stdout 2> scratch/api010-theme-durability/wrapper-012/verify.stderr)`

The wrapper and attempt roots must be absent before use, ignored, mode 0700,
and contain only mode-0600 private artifacts. Credentials, themes, hostnames,
origins, ports, USB/network/process identifiers, commands, HTTP bodies, serial,
and child traces remain private. The only eligible committed hardware artifact
is the redacted
`docs/parity/evidence/api010-theme-durability/theme-durability-projection.json`.

This plan authorizes one detector and one conditional capture only. Preserve a
completed flash independently from later proof, preserve the earliest typed
failure through recovery, and release every owned resource. Do not retry.

Promotion requires `bitaxe-theme-durability-evidence-v1` to bind the exact
clean source and reference, one board 205, one physical device, one software
restart, exact build recovery, changed boot session, ordinal `N+1`, immediate
and post-restart theme equality, exact original-theme restoration, disabled
mining and hardware control, complete cleanup, and passed redaction. Only
`API-010` may transition to `verified`.
