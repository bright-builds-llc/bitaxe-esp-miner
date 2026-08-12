# Parity work result

- Parity row: `NET-001`
- Final status: `verified`
- Implementation commit: `e56afbe44ab465eb3fc6b55acce761eb1d29a939`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The complete ordered Cargo, Bright Builds, 37-target Bazel, parity, progress,
redaction, reference, generated-contract, selector, immutable-plan, and diff
gates passed. The focused network reconnect suite additionally passed uncached
under Bun and through the canonical automation target, including a real POSIX
child that returns production-shaped monitor stdout without inventing a
monitor artifact.

The exact clean package built from pushed commit
`e56afbe44ab465eb3fc6b55acce761eb1d29a939`. One protected
`just detect-ultra205` invocation admitted exactly one board 205. The sole
conditional attempt-002 then ran the exact linked
`just capture-network-reconnect-evidence` command with fresh private roots and
the owner Wi-Fi file as an opaque local input. It completed successfully and
published
`docs/parity/evidence/net001-reconnect/network-reconnect-projection.json`.

The closed `bitaxe-network-reconnect-evidence-v1` projection binds the exact
source, pinned reference, and package manifest. It proves one post-boot station
disconnect, immediate configuration-AP fallback, first retry ordinal one,
configured 5,000-ms delay, observed 5,022-ms delay, same-boot DHCP recovery,
retry reset, client-only restoration, 15,000 ms of stable service, and matching
same-origin HTTP and exact-build postconditions. It also records admitted boot
evidence, disabled mining and hardware control, complete cleanup, no recovery
flash, valid private modes, and passed redaction.

The public projection passed both closed-field assertions and the independent
Rust `validate_network_reconnect_evidence` binary. Wrapper and attempt roots
were mode 0700, all contained files were mode 0600, and no process held the
attempt root after completion.

## Conclusion

The accepted exact-package hardware evidence closes the remaining ordinary
station reconnect lifecycle gap. Together with the existing persisted station
credential and hostname implementation and prior boot-connect evidence, this
supports `NET-001` at `verified` for Ultra 205.

## Non-claims and residual risks

This result does not verify repeated or long-running reconnect behavior,
provisioning-client suppression, live IPv6, router or RF failure modes,
credential mutation, `NET-002`, `NET-003`, other boards, mining, ASIC work,
voltage, frequency, fan, thermal or power controls, OTA, recovery behavior, or
release readiness. Raw detector, USB, flash, serial, network, origin,
credential, process, and HTTP material remains only in ignored private roots.
