# Parity work result

- Parity row: `NET-003`
- Final status: `verified`
- Implementation commit: `619752535d90a4aa8570b7c96f8339712a329ba8`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The closed projection at
`docs/parity/evidence/net003-scan/network-scan-projection.json` has SHA-256
`37107b8ad290696a09246c024f373a8d49f3ccb27d15be3cf3f4ff82215fafa1`.
It binds exact source commit `619752535d90a4aa8570b7c96f8339712a329ba8`,
the pinned reference, package-manifest digest
`41fbd948ac23622d8851e6834b8e00230a3893240f5cad73f6d18734800af4ca`,
and workflow-request digest
`a9640d7a9d35e5b1962530d74450a065c24a0aa8b033defd8feba1b3f26fe6fa`.

The immutable-plan sequence ran `bazel build
//firmware/bitaxe:firmware_image`, one protected `just detect-ultra205`, and
one conditional `just capture-network-scan-evidence` with the exact manifest,
opaque owner Wi-Fi input, detector output, fresh attempt-001 root, public
projection path, and 240-second capture bound. The selected USB path and all
raw serial, HTTP, radio, device, station, and credential values remain only in
ignored protected artifacts.

The typed transaction and an independent direct invocation of
`validate_network_scan_evidence` prove:

- one detector-admitted Ultra 205 and one exact-package passive safe boot;
- one trusted origin and exact source/reference/app identity;
- connected client-only system-info before and after exactly one scan;
- the same boot session, monotonic uptime, and stable station address;
- 20 records within the fixed one-to-20 response bound, each with exact wire
  shape, signed-byte signal value, and supported numeric auth mode;
- a valid stable v6 station address classified only as `unique_local`;
- a 3,152-ms scan transaction within the 10,000-ms bound;
- disabled mining and hardware control, complete cleanup, no recovery flash,
  mode-`0700` private directories, mode-`0600` private files, and no lingering
  flash/monitor process; and
- semantic redaction over all ten admitted public projections.

Before hardware, ordered Cargo formatting, strict Clippy, all-target build,
all-feature tests, Bright Builds, all 37 Bazel tests, the real ESP32-S3 image,
parity/progress, redaction, reference cleanliness, generated contracts,
selector, immutable-plan, task, fresh-path, and diff checks passed. The verbose
parity report twice encountered transient host `os error 35`; the unchanged
protected-file run passed with `validation_errors: none`. The first independent
validator invocation through `bazel run` used a runfiles-relative path and
failed before validation; the built validator invoked directly from the
workspace then passed against the same unchanged projection.

## Conclusion

The exact clean Rust package returned a real bounded Wi-Fi scan response on
the detector-admitted Ultra 205 while preserving connected client-only service,
boot identity, uptime ordering, and a stable valid station v6 projection. The
closed public evidence contains only aggregates and safe provenance. This
satisfies the remaining live scan, connection-preservation, and station v6
evidence required to promote `NET-003` to `verified`.

## Non-claims and residual risks

This attempt observed a unique-local address; it does not prove a global v6
assignment or router prefix delegation. It does not prove hidden-network
semantics, scan failure injection, overlapping or repeated scans, AP-only scan
restoration on live hardware, roaming performance, long-duration connectivity,
credential mutation, host network discovery, mining, ASIC work, voltage,
frequency, fan, thermal or power control, self-test, restart, OTA, erase,
recovery upload, other boards, direct UART, pins, or release readiness.
