# NET-002 Ultra 205 provisioning-network result

- Parity row: `NET-002`
- Final status: `verified`
- Implementation and package commit:
  `ec2204425fd44ec2ae67faee3300a829ee3b5046`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Board: Ultra 205

## Evidence and verification

The exact clean pushed implementation passed the mandatory ordered Cargo,
Bright Builds, Bazel, parity, progress, redaction, generated-contract, and
pinned-reference gates. The normal firmware package was rebuilt from that
commit. One fresh detector invocation admitted exactly one board 205 and one
private exact-device configuration candidate. The only conditional hardware
command was the attempt-006 capture recorded verbatim in immutable `PLAN.md`.

The closed committed
[provisioning-network projection](../../evidence/net002-provisioning/provisioning-network-projection.json)
has SHA-256
`c5add47149fbafe7ccdd3047c7c0d39f384f0d622c479e2ea72088e6a3d6b3c4`
and records:

- exact source, pinned-reference, package-manifest, and workflow identities;
- one detector-admitted Ultra 205 and one observed exact-package boot;
- a single eligible macOS Wi-Fi interface and clean unassociated baseline;
- one detector-bound candidate, live CoreWLAN association, and DHCP lease;
- one wildcard DNS query whose answer matched the configuration gateway with
  the pinned 300-second TTL;
- the expected captive 302 root redirect and exact captive response body;
- same-origin system-info AP/postcondition and exact build identity matches;
- disabled runtime mining and hardware control;
- complete host cleanup and exact-package owner-Wi-Fi recovery;
- valid private modes and passed semantic redaction.

The independent Rust validator accepted the projection after capture. A
separate semantic redaction check passed. Candidate, interface, port, USB,
network, route, origin, hostname, credential, DNS/HTTP bytes, CoreWLAN errors,
commands, processes, and raw serial content remain only in ignored mode-0700
private roots with mode-0600 files. No serial holder remains.

## Conclusion

The exact current Rust package exposes the expected configuration SoftAP on a
real Ultra 205, admits a live client, supplies DHCP, answers wildcard DNS, and
serves the captive redirect and configuration API while runtime mining and
hardware control remain disabled. Cleanup and exact-package recovery complete.
This satisfies `NET-002` with `unit,workflow,hardware-smoke` evidence.

## Non-claims and residual risks

This result does not verify credential submission, station handoff after
provisioning, repeated or long-running provisioning sessions, router/RF fault
behavior, other boards, mining, ASIC or pool work, voltage/fan/thermal/power
controls, self-test, OTA, recovery parity, release readiness, direct UART, or
pin manipulation. Those rows and hardware gates remain separate.
