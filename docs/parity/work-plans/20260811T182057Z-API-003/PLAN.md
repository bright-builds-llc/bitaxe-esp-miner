# Parity work plan

- Run ID: `20260811T182057Z-API-003`
- Parity row: `API-003`
- Initial status: `implemented`
- Source commit: `dcf58b3be41d660ac2d7920c668e1f73790a3072`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api003-live-multifield-patch`

## Selection

The branch is clean and synchronized, and the deterministic selector reports
no open plan. `CFG-001` remains behind safety-controlled default-frequency and
voltage soak evidence whose unchanged retry is closed at a repeated network
correlation boundary. `CFG-006` requires unavailable non-205 hardware.
`NET-001` through `NET-003` lack qualified reconnect, provisioning-client,
live-scan, and IPv6 environment contracts. `ASIC-002` through `ASIC-005`,
`ASIC-007`, `STR-001`, `STR-006`, and `STR-007` depend on safety-controlled
mining evidence whose last targeted attempt repeated its terminal continuity
signature and cannot be retried unchanged.

`API-003` is the first actionable row. Exhaustive reference-derived unit and
golden evidence now covers every accepted field, validation rule, atomic
rejection, NVS write, legacy mirror, reload, reconciliation, and route body
limit. Separate live evidence proves one-field hostname and theme mutation,
persistence, and restoration, but the row still lacks a current exact-package
multi-field PATCH smoke that proves the production route applies one benign
atomic request and restores both original values.

## Scope and non-scope

Add a typed aggregate-only `bitaxe-settings-patch-evidence-v1` workflow for one
Ultra 205. It will flash one exact clean package, derive the trusted same-origin
target from the admitted session, read baseline system information, submit one
PATCH containing generated non-secret hostname and theme values, confirm both
values in one immediate readback, restore both originals in one PATCH, confirm
restoration, clean up, and publish only closed categories, cryptographic
identities, hashes, counts, and safe booleans.

Do not expose or publish origins, hostnames, themes, ports, USB or network
identities, credentials, raw HTTP bodies, serial output, settings, or traces.
Do not read pool credentials, restart the device, mine, control ASIC, voltage,
fan, thermal, or power behavior, scan or discover the network, update, erase,
write raw flash data, inject faults, terminate foreign processes, use direct
UART, or manipulate pins. The generated hostname and theme must satisfy the
existing validators and differ from their baselines. This row does not claim
durability, broad live mutation of safety-sensitive or credential fields,
network reconnect behavior, mining, other boards, or release readiness.

## Implementation

- [ ] Add a typed private capture intent and closed public projection with
      exact package, workflow, mutation, readback, restoration, cleanup,
      disabled-mining, disabled-hardware-control, and redaction facts.
- [ ] Reuse the admitted flash/monitor and trusted-origin transaction seams;
      perform exactly one combined benign mutation PATCH and one combined
      restoration PATCH, preserving the earliest typed failure through
      recovery and cleanup.
- [ ] Add behavior-focused unit and real-child-process integration coverage for
      intent admission, atomic request shape, successful restoration, every
      terminal category, recovery precedence, no-clobber/private modes, and
      public-output privacy.
- [ ] Run the full software gate, commit and push the implementation, build and
      validate one exact schema-v3 package, then spend the sole detector and
      conditional hardware capture.
- [ ] Independently validate the closed projection and promote only `API-003`
      when every acceptance condition passes.

## Verification and promotion

Before hardware use, run focused settings/API/automation tests followed by:

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

After a clean implementation commit is pushed, build and validate its exact
package and run exactly these bounded commands:

1. `test ! -e scratch/api003-settings-patch/wrapper-001 && (umask 077; mkdir -m 700 -p scratch/api003-settings-patch/wrapper-001 && just detect-ultra205 > scratch/api003-settings-patch/wrapper-001/detector.stdout 2>&1)`
2. Only after command 1 succeeds:
   `test ! -e scratch/api003-settings-patch/attempt-001 && test ! -e docs/parity/evidence/api003-settings-patch/settings-patch-projection.json && (umask 077; just capture-settings-patch-evidence --private-root scratch/api003-settings-patch/attempt-001 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/api003-settings-patch/wrapper-001/detector.stdout --projection docs/parity/evidence/api003-settings-patch/settings-patch-projection.json --capture-timeout-seconds 240 > scratch/api003-settings-patch/wrapper-001/capture.stdout 2> scratch/api003-settings-patch/wrapper-001/capture.stderr)`

The ignored wrapper and attempt roots must be absent before use, mode 0700,
and contain only mode-0600 files. Detector failure stops before any write. The
capture permits one exact-package factory flash and its normal USB reset, one
generated two-field PATCH, immediate same-origin readback, one exact two-field
restoration PATCH, confirmed restoration, cleanup, and at most one exact-
package recovery flash after an initial flash effect. Preserve the earliest
typed failure; accepted failure categories are `hardware_blocked`,
`evidence_invalid`, `timeout`, and `process_failed`. No retry is permitted.

Promotion requires exact source/reference/package identity, one admitted board
205, trusted same-origin HTTP, one atomic mutation request, both generated
values observed together, one atomic restoration request, both original values
confirmed together, mining and hardware control disabled throughout, complete
cleanup, correct private modes, an independently validated redacted projection,
and every mandatory gate passing. Otherwise withhold `RESULT.md` and public
evidence, record a typed non-verified closure, keep `API-003` at `implemented`,
and stop this invocation without another hardware attempt.
