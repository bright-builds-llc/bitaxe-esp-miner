# Native USB configuration-AP recovery

- Run ID: `20260830T184150Z-NATIVE-USB-CONFIG-AP-RECOVERY`
- Source base: `d8a7049c`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-native-usb-config-ap-recovery-205`
- Blocked predecessors: `task-native-usb-recovery-transition-205`, `task-native-usb-display-recovery-205`

## Objective

Complete recovery-006 authentication without flashing or rewriting NVS. Read
and semantically validate the installed NVS partition over admitted USB, join
the Mac only to the USB-derived `Bitaxe_ABCD` configuration AP, authenticate
the recovery runtime, restore settings/theme once, restart once, restore the
Mac's original route, and prove final recovery through the station IP shown on
the display.

Mining, ASIC, fan/voltage effects, transition diagnostics, OTA, erase, manual
board buttons, direct UART/pins/pads/probes/headers, broad Wi-Fi scans, ARP,
mDNS, router inspection, subnet discovery, other devices, other boards,
durability, and parity promotion remain excluded.

## Interfaces and roots

Add `just native-usb-config-ap-recovery preflight|start|resume|finalize`.

- `preflight` creates no root or effect. It validates clean/pushed source,
  exact package/plan/task, recovery bundle/readiness/receipts, contained
  managed `esptool.py` and `nvs_tool.py`, host Wi-Fi eligibility, absent
  outputs, and zero owned processes.
- `start` creates `scratch/native-usb-config-ap-recovery/attempt-001` mode
  `0700` and owns NVS readback, directed AP association, strict local API
  restoration, one restart, host-route restoration, and the final display
  checkpoint.
- `resume` consumes authenticated private state after Codex or network
  connectivity returns. It never repeats NVS readback, association, settings
  PATCH, theme POST, or restart.
- `finalize` performs no USB, network, settings, or host-network effect and
  publishes
  `docs/parity/evidence/native-usb-config-ap-recovery/recovery-projection-001.json`.

All effectful counts are bounded to one. Preserve the earliest failure and
stop when the same authoritative post-fix signature repeats.

## Read-only NVS admission

Behind `UsbOwnership`, admit the same physical Ultra 205 and ROM downloader,
then run only a contained managed `esptool.py read_flash` for address `0x9000`
and size `0x6000`. Retain the dump privately as mode `0600`, return to the
recovery application, and prove cleanup.

Run contained managed `nvs_tool.py --integrity-check --format json` over the
dump. Independently generate and decode the expected ordinary seed using the
current protected Wi-Fi input. Compare typed semantic entries for namespace
`main`, `wifissid`, `wifipass`, `mineonboot=0`, and every ordinary Ultra 205
default. Raw NVS and values remain private; public evidence contains only
digests, counts, and semantic-match booleans.

## Configuration AP and restoration

Derive the exact `Bitaxe_XXXX` AP candidate from private ROM base-MAC evidence.
Snapshot the Mac's Wi-Fi power, interface, default-route interface, and
gateway. Permit one directed CoreWLAN scan and association only to that exact
open candidate; require one DHCP client address and RFC1918 gateway. Make no
external-network call while associated.

Authenticate the local API using strict direct HTTP:

- API `macAddr` equals the USB base MAC plus the documented AP offset;
- recovery source, reference, ELF, label/timestamp, partition, and
  `startMiningOnBoot=false` exactly match recovery-006;
- `wifi_status` is one of `credentials_missing`, `credentials_invalid`, or
  `connection_failed`.

Send at most one allowlisted system PATCH and one theme POST. Reconcile
uncertain responses by reads without repetition. After exact persistence,
request one software restart. Restore host Wi-Fi once and require the original
default-route interface/gateway to return. Then collect one no-timeout display
station-IP checkpoint, bind the station API MAC to the USB base MAC, and
require exact recovery identity/settings/theme, `paused` or `safe_blocked`,
zero hash rate and shares, final USB admission, cleanup, and zero owned
processes.

`resume` may perform only unfinished read/cleanup/checkpoint work authorized by
the sealed state. Unknown, inconsistent, already-consumed, or pre-effect state
fails closed.

## Evidence and guardrails

The public schema is
`bitaxe-native-usb-config-ap-recovery-projection-v1`. It contains only safe
source/evaluator/input digests, NVS integrity/schema/value-match booleans,
closed Wi-Fi reason, USB/AP/STA MAC-binding booleans, bounded association and
request counts, restoration/safe-state booleans, host-route restoration,
cleanup, and redaction status.

Raw NVS, SSIDs, passwords, pool values, MACs, IPs, gateways, interface names,
HTTP bodies, USB identities, and host-network details stay private. Local
development UI may show IP addresses. Add a short AGENTS pointer authorizing
only this detector-bound directed AP recovery path; it does not authorize
general network discovery.

Closed failures include `nvs_read_failed`, `nvs_integrity_failed`,
`nvs_schema_mismatch`, `candidate_absent`, `candidate_ambiguous`,
`association_failed`, `ap_identity_mismatch`, `restoration_uncertain`,
`restart_uncertain`, `host_network_restore_failed`,
`station_recovery_not_observed`, and cleanup failures.

## Verification and hardware contract

Use vertical red-to-green tests for read-only command ownership and exact
range, contained tools, NVS JSON/integrity/schema/value comparison, raw-value
exclusion, AP candidate derivation, CoreWLAN association/DHCP, AP MAC offset,
host route snapshot/restoration, strict HTTP identity/restoration/restart,
resume non-repetition, final station binding, projection allowlisting, modes,
runfiles, process groups, and cleanup.

Before each commit or hardware action run ordered Cargo formatting, strict
Clippy, all-target/all-feature build, all-feature tests, Bright Builds, all
Bazel tests, normal and rollback firmware links, canonical package,
native-USB ownership, parity/progress, redaction, reference cleanliness,
whitespace, sensitive-value scan, and final diff review.

After separate verified plan and implementation commits are pushed, run one
preflight and one start. Use `resume` only after verified state or host
connectivity return. On acceptance run one no-effect finalizer, create
`RESULT.md`, archive only `task-native-usb-config-ap-recovery-205`, and leave
recovery-006 installed. Transition diagnostics remain a later child.

## Assumptions

- The visible `Bitaxe_ABCD` is the USB-derived recovery-006 configuration AP.
- Temporary Mac Wi-Fi interruption is authorized; the local owner continues
  and host-route restoration is mandatory.
- The historical recovery firmware uses the ESP32-S3 base MAC for STA and the
  documented incremented MAC for AP.
- Repo-local native-USB/hardware/evidence guidance and Bright Builds
  architecture, code-shape, verification, testing, Rust, and TypeScript
  standards govern implementation.
