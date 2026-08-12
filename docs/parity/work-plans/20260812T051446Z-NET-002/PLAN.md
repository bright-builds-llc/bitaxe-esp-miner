# Parity work plan

- Run ID: `20260812T051446Z-NET-002`
- Parity row: `NET-002`
- Initial status: `implemented`
- Source commit: `7c8c1c01388aaf441c080634f0c25b4c43c40518`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-net002-provisioning-network-attempt-002`
- Continues plan: `docs/parity/work-plans/20260812T040437Z-NET-002/PLAN.md`

## Selection

The canonical selector reports no open plan and ranks `NET-002` first. The
explicit attempt-001 closure at
`docs/parity/work-plans/20260812T040437Z-NET-002/CLOSURE.md` records a recovered
`evidence_invalid` terminal outcome and consumes its ordinal. No row is skipped
and no attempt-001 private observation is reused as verification evidence.

Private aggregate diagnosis is sufficient to correct the orchestration
contract: the exact AP-only build emitted recurring trusted
`runtime_boot_attestation`, runtime-heartbeat, runtime-health, and operator
snapshot records, but the late-attached flash monitor missed the one-shot
`wifi_status=credentials_missing ap_enabled=true captive_dns=started` line.
The existing exact-safe-build predicate already accepts the recurring trusted
runtime attestation. Requiring the separate one-shot AP line before client
association is therefore an invalid readiness gate; the unique SSID plus the
same-origin API, DNS, and captive HTTP quorum are the authoritative live AP
proof.

## Scope and non-scope

Remove only the one-shot provisioning startup-line prerequisite from the typed
`capture-provisioning-network-evidence` transaction. Continue requiring exact
package identities and the recurring trusted passive-safe runtime attestation
before the host client acts. Then independently prove the unique local
configuration-network candidate, association, DHCP, wildcard IN/A response,
captive redirect, and same-origin system-info values
`wifiStatus=credentials_missing`, `apEnabled=1`, and
`startMiningOnBoot=false` with exact build identity. Preserve host cleanup,
same-package owner-Wi-Fi recovery, independent projection validation, private
modes, redaction, and earliest-failure precedence.

Add production-shaped regression coverage where the initial flash-monitor
output contains exact identities and recurring trusted safe runtime records but
no one-shot boot/AP line; the complete client/API quorum must proceed and pass.
Retain a fail-closed regression for missing trusted passive-safe runtime
attestation so this correction cannot weaken mining or hardware-control safety.

The private transaction may contain interface names, SSIDs, USB/device
identities, IP addresses, routes, origins, credentials, commands, HTTP bodies,
DNS bytes, and serial bytes. None may enter stdout, stderr, committed evidence,
diagnostics, or debug formatting. The public closed projection remains
unchanged and aggregate-only.

Authorized effects are one exact normal package flash without Wi-Fi
credentials; replacement default NVS with `mineonboot=false`; bounded
receive-only USB; local macOS candidate enumeration; association to the unique
open configuration AP; one DHCP lease; one synthetic wildcard IN/A UDP query;
one captive missing-path HTTP request; one same-origin system-info read; host
Wi-Fi off/on cleanup to powered-on and unassociated; and one ordinary recovery
flash of the same package with the opaque owner Wi-Fi input. Recovery must end
on the exact build with mining and hardware control disabled.

This plan does not authorize router/RF changes, non-Bitaxe association, host
credential access or mutation, provisioning credential submission, software
restart, station handoff, external discovery, internet evidence requests,
erase-flash, arbitrary raw writes, OTA, recovery upload, power interruption,
foreign-process termination, mining, ASIC initialization or work, pool
traffic, voltage, frequency, fan, thermal/power control, self-test, direct
UART, pins, pads, headers, GPIO, probes, jumpers, soldering, or injected
signals. Other boards, repeated provisioning, credential submission, station
handoff, and release readiness remain non-claims.

## Implementation

- [ ] Replace the one-shot AP startup marker prerequisite with the existing
      exact-build plus trusted passive-safe runtime admission.
- [ ] Add success and failure regressions proving late attachment proceeds only
      with recurring trusted safety and the full client/API quorum.
- [ ] Run focused and complete software gates, commit and push the exact
      implementation, then spend exactly one detector and at most one
      conditional attempt-002.
- [ ] Independently validate any emitted projection and promote only `NET-002`
      when every AP/client/DHCP/DNS/HTTP/safety/recovery/cleanup criterion
      passes.

## Verification and promotion

Run focused automation tests, then in order:

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
11. generated-contract, selector, immutable-plan, task-uniqueness,
    reference-cleanliness, sensitive-output, private-mode, fresh-path,
    no-holder, and diff checks

After the immutable plan and implementation commits are clean and pushed, run
exactly:

1. `bazel build //firmware/bitaxe:firmware_image`
2. `test ! -e scratch/net002-provisioning/wrapper-002 && (umask 077; mkdir -m 700 -p scratch/net002-provisioning/wrapper-002 && just detect-ultra205 > scratch/net002-provisioning/wrapper-002/detector.stdout 2>&1)`
3. Only after command 2 succeeds:
   `test ! -e scratch/net002-provisioning/attempt-002 && test ! -e docs/parity/evidence/net002-provisioning/provisioning-network-projection.json && (umask 077; just capture-provisioning-network-evidence --private-root scratch/net002-provisioning/attempt-002 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/net002-provisioning/wrapper-002/detector.stdout --projection docs/parity/evidence/net002-provisioning/provisioning-network-projection.json --capture-timeout-seconds 120 > scratch/net002-provisioning/wrapper-002/capture.stdout 2> scratch/net002-provisioning/wrapper-002/capture.stderr)`

Wrapper and attempt roots must be absent before creation, ignored, mode 0700,
and contain only mode-0600 files. Detector failure stops before writes. Any
conditional capture start consumes attempt-002. Preserve the earliest typed
failure through host cleanup and at most one ordinary exact-package recovery;
release every owned resource and never retry this ordinal.

Promotion requires exact source/reference/package identity; exactly one
detector-admitted board 205; trusted recurring passive-safe runtime admission;
the strict macOS host baseline; one unique configuration SSID; successful
association and DHCP; one wildcard IN/A answer equal to the AP gateway with TTL
300; exact captive 302/root/body behavior; same-origin system info with AP
enabled, credentials missing, exact build identity, and mining-on-boot false;
complete host restoration; exact-package device recovery with owner Wi-Fi and
disabled effects; cleanup; private modes; redaction; independent validation;
and all gates passing. Otherwise create a truthful closure, withhold evidence,
and keep `NET-002` at `implemented`.
