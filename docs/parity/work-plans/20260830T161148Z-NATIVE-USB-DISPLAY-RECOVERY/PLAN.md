# Native USB display-bound recovery authentication

- Run ID: `20260830T161148Z-NATIVE-USB-DISPLAY-RECOVERY`
- Source base: `6e3e88e0`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-native-usb-display-recovery-205`
- Blocked predecessor: `task-native-usb-recovery-transition-205`

## Objective

Authenticate and complete recovery-006 without another flash, NVS write,
manual button sequence, network discovery, or transition diagnostic. The
operator supplies the private IPv4 address currently shown on the Bitaxe
display. The host binds that address to the USB-admitted ESP32-S3 base MAC,
requires the exact recovery-006 runtime identity, restores settings once, and
proves the final inactive zero-work/share state.

This child stops with recovery-006 installed. It does not install the native
USB diagnostic package, run the transition verifier, resume the blocked
predecessor plan, promote parity, or authorize mining, ASIC, fan, voltage,
fault injection, erase, OTA, direct UART, pins, pads, probes, headers, jumpers,
soldering, test points, other devices, or other boards.

## Interfaces and evidence

Add `just native-usb-display-recovery preflight|capture|start|finalize`.

- `preflight` is effect-free and creates no task root. It validates clean,
  pushed source; this plan/task; the recovery-006 bundle/readiness/validator
  lineage; the completed snapshot and Wi-Fi-seed receipts under
  `scratch/native-usb-transition/recovery-002`; managed tooling; exact current
  package; absent outputs; and zero owned USB children.
- `capture` creates `scratch/native-usb-display-recovery/attempt-001` as mode
  `0700`, shows a plain macOS dialog with no human-response timeout, accepts an
  IPv4 address, and writes one mode-`0600` response. Local development UI and
  console output may show the address. Committed/shareable evidence remains
  redacted.
- `start` consumes the capture once, performs private USB admission and strict
  HTTP authentication, restores settings/theme, proves final state, and
  writes a protected result. It creates no flash, erase, NVS, reset-button, or
  discovery effect.
- `finalize` performs no USB, network, or settings effect. It independently
  validates the protected result and publishes the allowlisted projection at
  `docs/parity/evidence/native-usb-display-recovery/recovery-projection-001.json`.

The capture accepts only an origin-only RFC1918 IPv4 address on HTTP port 80.
It rejects hostnames, IPv6, public, loopback, link-local, multicast, broadcast,
unspecified, paths, queries, fragments, credentials, and explicit ports. The
dialog validates syntax before writing. One replacement capture is allowed
only when the first syntactically valid candidate is unreachable before any
settings request and the operator confirms a different currently displayed
address. Cancellation is effect-free. MAC or runtime-identity mismatch is
terminal and cannot consume a replacement capture.

The public schema is
`bitaxe-native-usb-display-recovery-projection-v1`. It contains only safe
source, reference, plan, evaluator, bundle, receipt, package, and input
digests; booleans for display input, private-address validation, USB/API MAC
binding, recovery identity, settings/theme exactness, mine-on-boot disablement,
inactive state, zero work/shares, stable physical identity, cleanup, and
redaction; bounded request/sample counts; and a closed terminal category. Raw
IPs, URLs, MACs, USB identities, ports, device nodes, settings, credentials,
HTTP bodies, timestamps, and process data remain local/private.

## USB and HTTP authentication

Add one exact task-bound private inspection path behind `UsbOwnership`:

1. Acquire one admitted Ultra 205 USB lease and require the
   `serial_jtag_runtime`/ROM-capable recovery profile.
2. Run read-only ESP32-S3 `board-info` and retain only SHA-256 digests of the
   normalized base MAC and stable physical identity in the task root.
3. Reacquire the same physical connector and release the lease with zero
   holders and children.
4. Normalize and hash API `macAddr`; require exact equality with the USB base
   MAC digest before any HTTP mutation.

The display-recovery HTTP Module uses the repository's direct strict HTTP/1.1
transport. It performs no proxy lookup, redirect, hostname resolution,
discovery, mDNS, ARP, router query, or subnet scan. Before settings mutation it
requires a complete parseable `/api/system/info` response matching every
recovery-006 identity field: source, reference, ELF, build timestamp/label,
running partition, and `startMiningOnBoot=false`.

After authentication, send at most one allowlisted system-settings PATCH and
one theme POST using the existing protected backup, Wi-Fi, and pool inputs.
Never enable mining or fallback. Record request-send completion separately
from response receipt. A missing/partial response never authorizes a repeated
request: reconcile with bounded read-only system/theme GETs and accept only an
already-exact state. Otherwise return `restoration_uncertain`.

Final acceptance requires exact settings and theme, the Wi-Fi and pool values
from the protected inputs, fallback disabled/empty, `startMiningOnBoot=false`,
`miningActivity` equal to `paused` or `safe_blocked`, zero hash rate and
accepted/rejected shares, stable USB identity, fresh final detector admission,
complete cleanup, and zero owned processes.

Closed failures preserve earliest precedence and include `capture_cancelled`,
`capture_invalid`, `origin_unreachable`, `usb_mac_mismatch`,
`recovery_identity_mismatch`, `settings_request_failed`,
`theme_request_failed`, `restoration_uncertain`, `runtime_state_mismatch`,
`physical_identity_drift`, and cleanup failures.

## Guardrails and durable learning

Update the always-loaded native-USB guidance with one short pointer: display
origins are a task-gated recovery fallback that requires the detailed native
USB document and USB/API MAC binding. Fresh monitor-derived origins remain the
ordinary path. This exception does not authorize general DEVICE_URL inference
or network discovery.

The required lesson audit is recorded before implementation. Append one lesson
after the audit: local RFC1918 addresses need no special interactive secrecy
during development, while committed/shareable evidence remains redacted.

Bind every reachable capture parser, prompt helper, USB reducer, HTTP model,
restoration validator, projector, finalizer, launcher, and source inventory
into evaluator identity.

## Verification and hardware contract

Use vertical red-to-green slices at the public Interfaces:

- RFC1918 parsing and rejection matrix;
- macOS dialog accept, cancel, malformed fixture, runfiles, and output modes;
- exact snapshot/NVS receipt and source-lineage admission;
- normalized USB/API MAC digest match and mismatch;
- strict fresh-process loopback for complete reads, redirects, timeouts, wrong
  MAC/build, no mutation before authentication, exact PATCH/POST, uncertain
  writes with read-only reconciliation, and final-state mismatch;
- private file/root modes, consume-once capture, one eligible correction,
  evaluator drift, explicit public allowlist, and raw-field exclusion;
- USB lease, physical identity, process-group, holder, and cleanup boundaries.

Before each commit or hardware action run ordered Cargo formatting, strict
Clippy, all-target/all-feature build, all-feature tests, Bright Builds,
focused tests, all Bazel tests, normal and rollback firmware links, canonical
package, native-USB ownership, parity/progress, redaction, reference
cleanliness, whitespace, sensitive-value scan, and final diff review.

After separate verified plan and implementation commits are pushed and an
exact clean package is built:

1. Run one effect-free preflight.
2. Run one display capture with no response timeout.
3. Run one authenticated restoration transaction.
4. Run one no-effect finalizer.

If accepted, create `RESULT.md`, archive only
`task-native-usb-display-recovery-205`, and leave recovery-006 installed. The
blocked predecessor remains historical/terminal. A later child may make the
single-transition diagnostic eligible using this recovery projection.

## Assumptions

- The currently displayed address is an RFC1918 IPv4 address reachable from
  the Mac.
- The runtime station `macAddr` is the ESP32-S3 base MAC returned by ROM
  `board-info`; any observed difference fails closed.
- Local development may display the IP normally. Raw addresses remain absent
  from committed public evidence.
- Planning followed repo-local native-USB, hardware, privacy, evidence, task,
  and lesson guidance plus Bright Builds architecture, code-shape,
  verification, testing, Rust, and TypeScript standards.
