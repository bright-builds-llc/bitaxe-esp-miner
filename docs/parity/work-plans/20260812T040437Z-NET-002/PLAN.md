# Parity work plan

- Run ID: `20260812T040437Z-NET-002`
- Parity row: `NET-002`
- Initial status: `implemented`
- Source commit: `58822cc90a6718e455ee6ad675ab18679d9a5b68`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-net002-provisioning-network-attempt-001`

## Selection

The branch and pinned reference are clean and synchronized. The canonical
selector reports no open plan and lists `NET-002` first, so no row was skipped.
The prior result at
`docs/parity/work-plans/20260804T160000Z-NET-002/RESULT.md` proves the bounded
AP-only/AP+STA implementation and wildcard DNS core, but explicitly withholds
live SSID visibility, association, DHCP, DNS-over-radio, captive redirect, and
settings access. `NET-001` now independently proves ordinary station reconnect
behavior. This plan closes the remaining configuration-network surface without
claiming credential submission or production station handoff.

The macOS host currently exposes exactly one Wi-Fi interface, powered on and
unassociated, and the read-only baseline inventory contains no matching
`Bitaxe_XXXX` candidate. Those facts make a bounded local client transaction
actionable without displacing an existing Wi-Fi association or requiring a
host credential.

## Scope and non-scope

Add a typed, fail-closed `capture-provisioning-network-evidence` transaction and
closed `bitaxe-provisioning-network-evidence-v1` projection. The transaction
must validate the exact package and detector admission, flash it without Wi-Fi
credentials, observe the safe AP-only boot over USB, discover exactly one local
configuration-network candidate, associate the sole admitted macOS Wi-Fi
interface, obtain DHCP, prove a wildcard IN/A DNS response to the AP gateway,
prove the captive HTTP redirect and same-origin system-info/build quorum, clean
up the host association, and recover the device with the same package plus the
owner Wi-Fi file as an opaque local input.

The private transaction may contain Wi-Fi interface names and state, SSIDs,
MACs, USB/device identities, IP addresses, routes, origins, credentials,
commands, HTTP bodies, DNS bytes, and serial bytes. None may enter stdout,
stderr, committed evidence, diagnostics, or debug formatting. The public
projection may contain only source/reference/package digests, board and closed
workflow identity, AP/client/DHCP/DNS/HTTP/build/safety/cleanup/mode/redaction
booleans, the DNS TTL and bounded attempt counts, and safe recovery booleans.

Authorized effects are one exact normal package flash without a Wi-Fi input;
replacement NVS containing the exact Ultra 205 defaults and
`mineonboot=false`; bounded receive-only USB; local macOS Wi-Fi candidate
enumeration; association to the single open configuration AP; one DHCP lease;
one synthetic wildcard IN/A UDP query to the AP DNS service; one missing-path
HTTP request and same-origin `/api/system/info` read; host Wi-Fi off/on cleanup
back to powered-on and unassociated; and one ordinary recovery flash of the
same package with the owner Wi-Fi file. Recovery must end on the exact build
with mining and hardware control disabled.

This plan does not authorize router or RF configuration changes, joining any
non-Bitaxe network, host credential access or mutation, provisioning credential
submission, a software restart, credential handoff, external discovery,
internet requests for evidence, erase-flash, arbitrary raw writes, OTA,
recovery upload, power interruption, foreign-process termination, mining, ASIC
initialization or work, pool traffic, voltage, frequency, fan, thermal or power
control, self-test, direct UART, pins, pads, headers, GPIO, probes, jumpers,
soldering, or injected signals. `NET-003`, other boards, credential submission,
station handoff, repeated provisioning, and release readiness remain
non-claims.

The design follows the repo-local task, USB, privacy, and evidence guidance plus
the Bright Builds architecture, code-shape, verification, testing, Rust, and
TypeScript standards. The host client is admitted only on macOS with exactly
one powered-on, initially unassociated Wi-Fi interface and zero matching
baseline candidates; any mismatch stops before the first flash.

## Implementation

- [ ] Add the Rust-owned closed evidence schema, generated TypeScript contract,
      independent validator, CLI/Justfile wiring, and redaction policy.
- [ ] Add the imperative macOS provisioning-client transaction with strict
      initial-state admission, unique candidate selection, DHCP/DNS/HTTP
      validation, earliest-failure preservation, host cleanup, and exact-package
      recovery.
- [ ] Add focused pure/process tests for candidate parsing, admission,
      association ordering, DHCP/DNS wire validation, captive redirect and API
      quorum, every failure category, recovery precedence, sensitive-output
      absence, and a real child process that produces production-shaped output.
- [ ] Run the complete software gate, commit and push the exact implementation,
      then spend exactly one detector and at most one conditional attempt-001.
- [ ] Independently validate any emitted projection and promote only `NET-002`
      when every exact-package, AP/client/DHCP/DNS/HTTP/safety/recovery/cleanup,
      mode, and redaction fact passes.

## Verification and promotion

Run focused automation and contract tests, then in order:

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
11. selector, immutable-plan, generated-contract, task-uniqueness,
    reference-cleanliness, sensitive-output, private-mode, fresh-path, and diff
    checks

After the immutable plan and implementation commits are clean and pushed, run
exactly:

1. `bazel build //firmware/bitaxe:firmware_image`
2. `test ! -e scratch/net002-provisioning/wrapper-001 && (umask 077; mkdir -m 700 -p scratch/net002-provisioning/wrapper-001 && just detect-ultra205 > scratch/net002-provisioning/wrapper-001/detector.stdout 2>&1)`
3. Only after command 2 succeeds:
   `test ! -e scratch/net002-provisioning/attempt-001 && test ! -e docs/parity/evidence/net002-provisioning/provisioning-network-projection.json && (umask 077; just capture-provisioning-network-evidence --private-root scratch/net002-provisioning/attempt-001 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/net002-provisioning/wrapper-001/detector.stdout --projection docs/parity/evidence/net002-provisioning/provisioning-network-projection.json --capture-timeout-seconds 120 > scratch/net002-provisioning/wrapper-001/capture.stdout 2> scratch/net002-provisioning/wrapper-001/capture.stderr)`

Wrapper and attempt roots must be absent before creation, ignored, mode 0700,
and contain only mode-0600 files. Detector failure stops before writes. Any
conditional capture start consumes attempt-001. Preserve the earliest typed
failure through host cleanup and at most one ordinary exact-package recovery
flash; release every owned resource and never retry this ordinal. Accepted
non-success categories are `package_invalid`, `process_failed`, `timeout`,
`hardware_blocked`, `evidence_invalid`, `service_recovery_failed`, and
`recovery_failed`.

Promotion requires exact source/reference/package identity; exactly one
detector-admitted board 205; the strict macOS host baseline; safe AP-only boot;
one unique configuration SSID; successful association and DHCP; one wildcard
IN/A response whose answer equals the admitted AP gateway with the configured
300-second TTL; the exact captive 302/root/body contract; same-origin system
info with AP enabled, credentials-missing status, exact build identity, and
disabled mining/control; complete host restoration; exact-package device
recovery with owner Wi-Fi and disabled effects; complete USB/process cleanup;
private modes; redaction; independent projection validation; and all gates
passing. Otherwise create a truthful closure, withhold public evidence, and
keep `NET-002` at `implemented`.
