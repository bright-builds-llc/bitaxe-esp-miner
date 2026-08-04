# V12-OPERATOR-SNAPSHOT-205 work result

- Parity row: `V12-OPERATOR-SNAPSHOT-205`
- Final status: `verified`
- Implementation and package commit:
  `409864d0eb053c335225a99999bac61d7a6b3d1b`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Board: Ultra 205
- Hardware attempts: one

## Evidence and verification

The detector-gated `attempt-001` ran the committed
`just capture-operator-snapshot-evidence` interface against the exact clean
package built from the pushed implementation commit. The private detector
admitted exactly one Ultra 205 before the workflow flashed that package and
confirmed the passive disabled safe state.

The committed
[`bitaxe-operator-snapshot-evidence-v1`](../../evidence/v12-operator-snapshot-205/snapshot-projection.json)
projection has SHA-256
`f2ea9b8ef5e566d32a42605bbd819bb19575406dd3ae2a63853b7149a5597bd5`.
Its closed facts prove:

- an HTTP system-info snapshot and a later same-boot WebSocket snapshot in
  each of two boot epochs;
- exact retained-log marker membership for both revisions in each epoch;
- substantive Ultra 205/BM1366, build, version, mining-state, and operator
  fields present and identical across each HTTP/WebSocket pair;
- monotonic revisions within each epoch and distinct cryptographic boot
  session identities across epochs;
- the same physical USB device, an armed reader, one restart request, service
  loss, and recovery of the exact source, reference, and application ELF;
- one software restart with boot ordinal `N+1`, preserved hostname
  postcondition, and complete holder-free cleanup; and
- mining disabled, hardware control disabled, and redaction passed.

The Rust-owned validator independently accepted the committed projection. The
private attempt root was mode `0700`, every private file was mode `0600`, the
semantic redaction verifier passed, and the projection contains no origin,
hostname, network or USB identifier, port, credential, raw document, trace, or
process identifier.

Before the hardware attempt, the following gates passed on the exact pushed
source:

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bun scripts/bright-builds-check.ts all`
- `just test`
- `just parity`
- `just parity-progress`
- `just verify-redaction`
- `just verify-reference`
- `git diff --check`

Focused tests cover every non-ready restart category, malformed and missing
projections, epoch session/revision/substance mismatch, retained-join failure,
recovery precedence, prepublication validation, sensitive-value denial, and a
real child-process transaction.

## Conclusion

The exact pushed Rust package produced coherent, substantive operator
snapshots through HTTP, WebSocket, and retained logs on both sides of one
attributed same-device software restart. This closes the narrow
`V12-OPERATOR-SNAPSHOT-205` claim with typed workflow and hardware-smoke
evidence.

## Privacy, recovery, and non-claims

USB identities and paths, serial, HTTP, WebSocket, and retained-log documents,
device origins, hostnames, network identifiers, and Wi-Fi credentials remain
only in ignored private roots. The credential file was passed as an opaque
local input and was never read, printed, summarized, or copied. The primary
workflow succeeded, so no recovery flash ran and the single attempt was not
repeated.

This result does not claim broader settings or networking parity, runtime
health, mining, ASIC work, voltage, fan, thermal or power control, OTA or
recovery, other boards, release readiness, direct UART, or pin manipulation.
