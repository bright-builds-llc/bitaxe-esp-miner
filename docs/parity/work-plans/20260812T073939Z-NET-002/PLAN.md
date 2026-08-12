# Parity work plan

- Run ID: `20260812T073939Z-NET-002`
- Parity row: `NET-002`
- Initial status: `implemented`
- Source commit: `3dfdafd24437acd0f465fb9ae4fd6ea970082afa`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-net002-provisioning-network-attempt-006`
- Continues plan: `docs/parity/work-plans/20260812T071223Z-NET-002/PLAN.md`

## Selection and diagnosis

The canonical selector has no open plan and ranks `NET-002` first. Attempt-005
is closed after every live network behavior passed: exact-device CoreWLAN
association, DHCP, wildcard DNS, captive redirect, system-info, cleanup, and
recovery. It failed only because the evidence checker required the persisted
`startMiningOnBoot` preference to be false, although the API and checked-in wire
fixtures establish true as valid and the exact runtime attestation separately
proved mining, work submission, and hardware control disabled.

## Implementation and effects

Remove only `startMiningOnBoot === false` from the configuration-network API
postcondition. Keep `wifiStatus === credentials_missing`, `apEnabled === 1`,
exact build identity, and the already-required exact-package runtime safety
attestation. Add paired regressions proving both boolean preference values are
eligible only when runtime safety is independently disabled; missing or
non-boolean preference data remains irrelevant to this row because NET-002 does
not claim settings defaults or mining configuration.

All CoreWLAN, DHCP, DNS, captive HTTP, recovery, privacy, validator, and public
schema behavior remains unchanged. Perform a simplification and sensitive-
output review before the full gate.

Authorized and prohibited effects remain identical to attempt-005: one exact
normal AP-only flash, receive-only USB, one exact directed CoreWLAN scan and
association, local DHCP, one DNS query, captive and same-origin system-info
reads, host cleanup, and one exact-package owner-Wi-Fi recovery flash. No
provisioning submission, router mutation, broad discovery, erase/raw write,
OTA, mining, ASIC/pool work, controls, self-test, direct UART, or pins.

## Verification and hardware attempt

Run focused true/false preference and runtime-safety regressions, then ordered
Cargo, Bright Builds, `just test`, parity, progress, redaction, reference,
generated-contract, selector, immutable-plan, task, reference-cleanliness,
sensitive-output, fresh-path, private-mode, no-holder, and diff gates. Commit
and push this immutable plan before implementation and the exact implementation
before hardware use.

After a clean pushed implementation:

1. `bazel build //firmware/bitaxe:firmware_image`
2. `test ! -e scratch/net002-provisioning/wrapper-006 && (umask 077; mkdir -m 700 -p scratch/net002-provisioning/wrapper-006 && just detect-ultra205 > scratch/net002-provisioning/wrapper-006/detector.stdout 2>&1)`
3. Only after detector success:
   `test ! -e scratch/net002-provisioning/attempt-006 && test ! -e docs/parity/evidence/net002-provisioning/provisioning-network-projection.json && (umask 077; just capture-provisioning-network-evidence --private-root scratch/net002-provisioning/attempt-006 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/net002-provisioning/wrapper-006/detector.stdout --projection docs/parity/evidence/net002-provisioning/provisioning-network-projection.json --capture-timeout-seconds 120 > scratch/net002-provisioning/wrapper-006/capture.stdout 2> scratch/net002-provisioning/wrapper-006/capture.stderr)`

Any capture start consumes attempt-006; never retry it. Promotion requires the
complete attempt-005 live quorum plus independent validation and redaction.
Otherwise record the earliest closed category, withhold evidence, and keep
`NET-002` implemented.
