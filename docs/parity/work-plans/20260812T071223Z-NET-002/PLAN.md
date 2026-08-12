# Parity work plan

- Run ID: `20260812T071223Z-NET-002`
- Parity row: `NET-002`
- Initial status: `implemented`
- Source commit: `62f633674ca9e1ef66d356231f3b1feaa9e3db17`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-net002-provisioning-network-attempt-005`
- Continues plan: `docs/parity/work-plans/20260812T063811Z-NET-002/PLAN.md`

## Selection and diagnosis

The canonical selector has no open plan and again ranks `NET-002` first.
Attempt-004 is closed after exact device-derived candidate admission,
exact-package passive safety, and eleven recurring AP/DHCP/DNS readiness
samples passed. It failed only at the typed `association` boundary; host and
device recovery passed and no public projection exists.

The installed Apple macOS 26.5 SDK identifies the root host seam. The existing
`networksetup -setairportnetwork` command depends on the ordinary Wi-Fi
inventory that omitted the exact candidate. CoreWLAN instead exposes a directed
`scanForNetworks(withSSID:includeHidden:)` request and typed
`associate(to:password:)` transaction, both available on the qualified host.
A no-effect Swift typecheck against the installed SDK passed. This provides an
exact-candidate association path without broad discovery or a factory reset.

## Implementation and effects

Add one repository-owned Swift CoreWLAN helper and keep it behind the existing
TypeScript provisioning client. Production input is a mode-0600 JSON intent
beneath the mode-0700 private attempt root containing only the admitted Wi-Fi
interface and detector-bound candidate. The child command line contains only
the checked-in helper path and private intent/result paths, never the candidate.

The helper must validate the closed candidate shape, resolve the exact admitted
interface, perform one directed scan with hidden inclusion, reject zero or
ambiguous exact matches, call the blocking open-network association API once,
and require the interface to reach running state. It writes one mode-0600
private result with a closed internal status plus private NSError domain/code
when present. It emits no stdout. Parent execution is bounded to 45 seconds;
launch, timeout, malformed result, directed-scan, selection, association, and
confirmation outcomes remain privately distinguishable while every public
non-ready result maps to the existing `association` boundary.

Keep `system_profiler` only for the pre-effect ambiguity guard: a different
Bitaxe candidate still fails closed, while zero or the exact candidate is
eligible. Keep the existing DHCP, DNS, captive redirect, system-info, host
cleanup, exact-package recovery, evidence validator, and public success schema.
Add helper fixture mode for a real-child no-network integration test plus
focused tests for intent/result modes, no candidate in argv/stdout, every
private association subtype, timeout, malformed/missing result, primary failure
precedence, cleanup, recovery, and sensitive-output absence. Perform a
simplification pass before the full gate.

Authorized effects are one exact normal package flash without credentials,
safe default NVS, bounded receive-only USB, one directed CoreWLAN scan for only
the detector-bound open configuration AP, one association request, DHCP, one
wildcard DNS query, captive and same-origin system-info reads, host Wi-Fi
cleanup, and one exact-package owner-Wi-Fi recovery flash. Prohibited effects
remain provisioning submission, router/RF mutation, external or broad network
discovery, erase/raw writes, OTA, power interruption, mining, ASIC/pool work,
controls, self-test, direct UART, and pins.

All detector/device identities, candidates, interfaces, ports, addresses,
routes, origins, credentials, DNS/HTTP bytes, NSError descriptions, commands,
processes, and serial content remain private. Public failure output remains one
closed boundary token plus safe recovery booleans.

## Verification and hardware attempt

Run focused Swift typecheck/fixture, automation real-child and boundary tests,
then the ordered Cargo, Bright Builds, `just test`, parity, progress, redaction,
reference, generated-contract, selector, immutable-plan, task,
reference-cleanliness, sensitive-output, fresh-path, private-mode, no-holder,
and diff gates. Commit and push this immutable plan before implementation and
the exact implementation before hardware use.

After a clean pushed implementation:

1. `bazel build //firmware/bitaxe:firmware_image`
2. `test ! -e scratch/net002-provisioning/wrapper-005 && (umask 077; mkdir -m 700 -p scratch/net002-provisioning/wrapper-005 && just detect-ultra205 > scratch/net002-provisioning/wrapper-005/detector.stdout 2>&1)`
3. Only after detector success:
   `test ! -e scratch/net002-provisioning/attempt-005 && test ! -e docs/parity/evidence/net002-provisioning/provisioning-network-projection.json && (umask 077; just capture-provisioning-network-evidence --private-root scratch/net002-provisioning/attempt-005 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/net002-provisioning/wrapper-005/detector.stdout --projection docs/parity/evidence/net002-provisioning/provisioning-network-projection.json --capture-timeout-seconds 120 > scratch/net002-provisioning/wrapper-005/capture.stdout 2> scratch/net002-provisioning/wrapper-005/capture.stderr)`

Any capture start consumes attempt-005; never retry this ordinal. Promotion
requires exact detector binding, two recurring AP-ready samples, exact-package
passive safety, directed exact-candidate CoreWLAN association, DHCP, DNS,
captive redirect, system-info, cleanup, exact recovery, modes, redaction, and
independent validation. Otherwise record the earliest closed boundary, withhold
public evidence, and keep `NET-002` implemented.
