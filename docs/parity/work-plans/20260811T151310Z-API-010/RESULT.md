# Ultra 205 boot recovery result

- Parity row: `API-010`
- Final status: `implemented`
- Implementation and package commit:
  `fc12e24fdb5b9fda35964b9e774f5727b456aa16`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Plan SHA-256:
  `ed79db9ec36f9c7426db50926bc7dc5c55183267bae569425007376c591317c3`
- Board: Ultra 205
- Hardware attempts under this plan: one

## Evidence and verification

Attempt 011 ran one protected detector and one conditional canonical
observation campaign. The detector admitted exactly one Ultra 205 and completed
board-info. The campaign admitted the exact clean schema-v3 package built from
the pushed implementation commit, completed the factory-package flash, applied
the safe NVS seed, and then entered bounded receive-only runtime capture.

The private sealed `mining-campaign-result-v5` record has SHA-256
`c153170aae6172909332a9cd4e974c6da86eca6b896088477fcdb45ecbc202a4`.
Its closed fields record:

- accepted `observation_complete` status after 360 seconds;
- trusted package and runtime identity with clean serial classification;
- 1,049 accepted runtime markers and five fresh observation checkpoints;
- `mineonboot=false`, no mining profile, and no pool configuration read;
- fresh safety state, no campaign failure, and no parity promotion; and
- ready USB cleanup with no remaining holder on the admitted port.

Both private roots were mode 0700 and every contained artifact was mode 0600.
The result digest matched its seal, and the redacted result contained no USB
port, origin, IP address, or MAC address.

The complete ordered Cargo gate, Bright Builds checks, all Bazel tests,
package, parity/progress, redaction, reference, selector, immutable-plan, and
diff checks passed before the hardware attempt. Focused regressions prove the
canonical observation command parses without a mining profile or pool
credentials and that the earlier assignment-style form remains rejected.

## Conclusion

The current pushed firmware remained alive and emitted fresh, trusted runtime
markers throughout the bounded observation window. This resolves the observed
panic-reset boot loop and proves the device can be flashed successfully through
the normal detector-gated USB path. A factory reset was not required.

`API-010` remains `implemented`: this recovery attempt did not mutate a theme,
restart the device, or prove theme durability.

## Privacy and non-claims

Raw detector, USB, flash, serial, network, credential, and process data remain
only in ignored private roots. Wi-Fi credentials were passed as an opaque local
input and no pool credential file was read. No mining, pool traffic, ASIC work,
voltage, frequency, fan, thermal, power-control, OTA, erase-flash, raw-write,
direct-UART, or pin effect was authorized or claimed.

This result does not verify theme persistence, broader API parity, network
longevity, mining, hardware controls, update or recovery behavior, other
boards, or release readiness.
