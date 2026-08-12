# CFG-001 Ultra 205 configured-defaults result

- Parity row: `CFG-001`
- Final status: `verified`
- Implementation and package commit:
  `78bbc156f3a8c7d0b1cacd21396933d80fdc9612`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Board: Ultra 205

## Evidence and verification

The exact clean pushed implementation passed the mandatory ordered Cargo,
Bright Builds, Bazel, parity, progress, redaction, and pinned-reference gates.
The normal firmware package was then rebuilt from that commit. One fresh
`just detect-ultra205` invocation admitted exactly one board 205, and the only
conditional hardware command was:

```text
just capture-ultra205-defaults-evidence --private-root scratch/cfg001-ultra205-defaults/attempt-001 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/cfg001-ultra205-defaults/wrapper-001/detector.stdout --projection docs/parity/evidence/cfg001-ultra205-defaults/ultra205-defaults-projection.json --capture-timeout-seconds 600
```

The closed committed
[defaults projection](../../evidence/cfg001-ultra205-defaults/ultra205-defaults-projection.json)
has SHA-256
`415ac8830d2360b0ceff5fe1cf68f69fcf95d9a44da749b4c641b7c82e38f8b2`
and records:

- exact source, pinned-reference, package-manifest, workflow-request, seed-
  fixture, and nested system-info evidence identities;
- one detector-admitted board 205 and one observed boot;
- all 27 configured fields matching the pinned Ultra 205 seed inside the
  firmware's strict loaded-NVS attestation;
- all 23 API-visible configured fields matching the same seed in both the
  same-boot HTTP and equal-or-later WebSocket snapshots;
- exact retained attestation continuity;
- the deliberate `mineonboot=false` override, disabled mining, and disabled
  hardware control;
- complete cleanup, valid private directory/file modes, and passed redaction.

The independent Rust validator accepted the public projection after capture.
A separate sensitive-pattern check found no origin, hostname, port, USB,
network, Wi-Fi, pool, credential, device identity, or raw trace value. Raw
detector, board-info, serial, HTTP, WebSocket, and configured values remain
only in the ignored mode-`0700` private roots with mode-`0600` files. The owner
Wi-Fi credential file was passed as an opaque local input and was not read,
printed, summarized, copied, or committed. No recovery flash ran because the
single attempt succeeded and cleanup completed.

## Conclusion

The exact current Rust package loads the complete pinned Ultra 205 configured-
default profile and reports every observable configured value consistently on
a real Ultra 205 while mining and hardware control remain disabled. This
satisfies `CFG-001` with `unit,golden,workflow,hardware-smoke` evidence.

## Non-claims and residual risks

This result proves configured values, not actuation. It does not verify mining,
ASIC initialization or work, pool connectivity, frequency or voltage effects,
fan or thermal control, self-test execution, power behavior, OTA, recovery,
long-running network behavior, non-205 boards, direct UART, or pin
manipulation. Those checklist rows and their hardware gates remain unchanged.
