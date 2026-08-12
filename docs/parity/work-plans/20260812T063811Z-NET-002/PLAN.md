# Parity work plan

- Run ID: `20260812T063811Z-NET-002`
- Parity row: `NET-002`
- Initial status: `implemented`
- Source commit: `41a88e98a997c8bb81e12821bd3c206b7c25dc24`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-net002-provisioning-network-attempt-004`
- Continues plan: `docs/parity/work-plans/20260812T061233Z-NET-002/PLAN.md`

## Selection and diagnosis

The canonical selector has no open plan and ranks `NET-002` first. Attempt-003
is explicitly closed after exact runtime admission passed and the typed macOS
client stopped at `configuration_candidate`. No later network effect began,
host and exact-package device recovery passed, and no projection exists.

The exact-device detector already runs `espflash board-info` in one owned USB
session. With the pinned ESP32-S3 four-universal-MAC SDK configuration, ESP-IDF
defines the SoftAP address as the detected base address plus one. The current
detector discards that private output, and the client treats macOS nearby-network
enumeration as the only candidate source. The device also emits no recurring
AP-ready fact, so late serial attachment cannot independently prove the AP,
DHCP, and captive DNS owners are ready. This plan closes both identity and
readiness gaps without exposing a MAC address or SSID.

## Implementation and effects

Extend the protected detector handoff with a private expected configuration
SSID derived inside the same admitted board-info session. Parse one canonical
base MAC, increment the final octet with checked arithmetic under the pinned
four-address contract, and emit the resulting candidate only into the
mode-0600 ignored detector artifact. Keep detector stdout backward-compatible
for every other consumer and reject missing, duplicate, malformed, or overflow
identity.

Add a recurring firmware marker with only closed categories:
`provisioning_network_ready schema_version=1 ap=ready dhcp=ready dns=ready
redacted=true`. Publish it only after AP netif readiness and captive DNS startup
have both succeeded, and replay it at the existing boot-evidence cadence. It
contains no SSID, MAC, address, route, origin, hostname, or credential value.

Change only this capture path to parse the private candidate and require two
recurring ready markers from the exact package before client association. The
macOS client may record whether enumeration observed that exact candidate, but
it joins the exact detector-derived candidate even when the inventory omits it;
an observed different Bitaxe candidate remains ambiguous and fails closed.
Preserve the six public failure boundaries, exact-package safety gate, DHCP,
DNS, captive redirect, system-info, cleanup, recovery, validator, and unchanged
public success schema.

Add focused firmware/core, detector, client, orchestration, redaction, malformed
identity, late-attach, enumeration-invisible, ambiguity, recovery-precedence,
and real-child regressions. Perform a simplification pass before the full gate.

Authorized and prohibited effects remain identical to attempt-003: one exact
normal package flash without credentials, safe default NVS, bounded receive-only
USB, association only to the detector-bound open configuration AP, DHCP, one
wildcard DNS query, captive and same-origin system-info reads, host Wi-Fi
cleanup, and one exact-package owner-Wi-Fi recovery flash. No provisioning
submission, router/RF mutation, external discovery, erase/raw writes, OTA,
power interruption, mining, ASIC/pool work, controls, self-test, direct UART,
or pins are permitted.

All detector/device identities, SSIDs, interfaces, ports, addresses, routes,
origins, credentials, DNS/HTTP bytes, commands, processes, and serial content
remain private. Public failure output remains one existing closed boundary
token plus safe recovery booleans.

## Verification and hardware attempt

Run focused tests and the ordered Cargo, Bright Builds, `just test`, parity,
progress, redaction, reference, generated-contract, selector, immutable-plan,
task, reference-cleanliness, sensitive-output, fresh-path, private-mode,
no-holder, and diff gates. Commit and push this immutable plan before
implementation and the exact implementation before hardware use.

After a clean pushed implementation:

1. `bazel build //firmware/bitaxe:firmware_image`
2. `test ! -e scratch/net002-provisioning/wrapper-004 && (umask 077; mkdir -m 700 -p scratch/net002-provisioning/wrapper-004 && just detect-ultra205 > scratch/net002-provisioning/wrapper-004/detector.stdout 2>&1)`
3. Only after detector success:
   `test ! -e scratch/net002-provisioning/attempt-004 && test ! -e docs/parity/evidence/net002-provisioning/provisioning-network-projection.json && (umask 077; just capture-provisioning-network-evidence --private-root scratch/net002-provisioning/attempt-004 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/net002-provisioning/wrapper-004/detector.stdout --projection docs/parity/evidence/net002-provisioning/provisioning-network-projection.json --capture-timeout-seconds 120 > scratch/net002-provisioning/wrapper-004/capture.stdout 2> scratch/net002-provisioning/wrapper-004/capture.stderr)`

Any capture start consumes attempt-004; never retry this ordinal. Promotion
requires exact detector binding, two recurring AP-ready samples, exact-package
passive safety, live association, DHCP, DNS, captive redirect, system-info,
cleanup, exact recovery, modes, redaction, and independent validation.
Otherwise record the earliest closed boundary, withhold public evidence, and
keep `NET-002` implemented.
