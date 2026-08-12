# Parity work plan

- Run ID: `20260812T061233Z-NET-002`
- Parity row: `NET-002`
- Initial status: `implemented`
- Source commit: `aba1d583ead8ec4e9fb366b57db35ff950886a8a`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-net002-provisioning-network-attempt-003`
- Continues plan: `docs/parity/work-plans/20260812T051446Z-NET-002/PLAN.md`

## Selection and diagnosis

The canonical selector has no open plan and ranks `NET-002` first. Attempt-002
is explicitly closed after recurring passive-safe runtime admission passed and
the live client transaction failed before system-info publication. Host and
device recovery passed, but the current client maps candidate discovery,
association, DHCP, DNS, captive redirect, and API failures to one generic
`hardware_blocked` envelope. This plan adds closed boundary truth; it does not
reuse attempt-002 as client evidence.

## Implementation and effects

Add a private `ProvisioningClientError` carrying exactly one closed boundary:
`configuration_candidate`, `association`, `dhcp`, `wildcard_dns`,
`captive_redirect`, or `system_info`. Wrap each effect boundary at its narrow
owner, preserve the original detail only inside the process, and project only
the boundary token with the existing generic terminal category. Preserve host
admission semantics, exact-package runtime admission, earliest-failure
precedence, cleanup, exact recovery, and the unchanged public success schema.

Add focused tests for all six boundaries, public sensitive-output absence,
primary precedence through cleanup/recovery, and real-child behavior. The
success path and every existing client parser/ordering test remain mandatory.

Authorized effects are unchanged: one exact normal package flash without
credentials, safe default NVS, bounded receive-only USB, candidate enumeration,
association to one unique open Bitaxe configuration AP, DHCP, one wildcard DNS
query, captive and same-origin system-info reads, host Wi-Fi off/on cleanup,
and one exact-package owner-Wi-Fi recovery flash. Prohibited effects remain
router/RF mutation, non-Bitaxe association, provisioning submission, external
discovery, erase/raw writes, OTA, power interruption, mining, ASIC/pool work,
controls, self-test, direct UART, and pins.

All interface, SSID, USB/device, IP, route, origin, credential, command,
process, DNS, HTTP, and serial values remain private. Public failure output may
add only one of the six closed boundary tokens plus safe recovery booleans.

## Verification and hardware attempt

Run focused tests and the ordered Cargo, Bright Builds, `just test`, parity,
progress, redaction, reference, generated-contract, selector, immutable-plan,
task, reference-cleanliness, sensitive-output, fresh-path, private-mode,
no-holder, and diff gates. Commit and push the plan before implementation and
the exact implementation before hardware use.

After a clean pushed implementation:

1. `bazel build //firmware/bitaxe:firmware_image`
2. `test ! -e scratch/net002-provisioning/wrapper-003 && (umask 077; mkdir -m 700 -p scratch/net002-provisioning/wrapper-003 && just detect-ultra205 > scratch/net002-provisioning/wrapper-003/detector.stdout 2>&1)`
3. Only after detector success:
   `test ! -e scratch/net002-provisioning/attempt-003 && test ! -e docs/parity/evidence/net002-provisioning/provisioning-network-projection.json && (umask 077; just capture-provisioning-network-evidence --private-root scratch/net002-provisioning/attempt-003 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/net002-provisioning/wrapper-003/detector.stdout --projection docs/parity/evidence/net002-provisioning/provisioning-network-projection.json --capture-timeout-seconds 120 > scratch/net002-provisioning/wrapper-003/capture.stdout 2> scratch/net002-provisioning/wrapper-003/capture.stderr)`

Any capture start consumes attempt-003; never retry this ordinal. Promotion
still requires the complete exact-package, recurring-safe-runtime,
AP/client/DHCP/DNS/HTTP/API, recovery, cleanup, mode, redaction, and independent
validator quorum. Otherwise record the exact closed boundary, withhold public
evidence, and keep `NET-002` implemented.
