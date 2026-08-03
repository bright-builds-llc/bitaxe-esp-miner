# REL-09 typed operator-workflow result

- Parity row: `REL-09`
- Final status: `verified`
- Evidence source commit: `66cf184943d7f3a5aedfc99e692a9f500707de9e`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Board: Ultra 205

## Evidence and verification

The new [release operator-evidence root](../../evidence/rel09-typed-operator-workflow/conclusion.md)
contains the exact current inventory and profile/disposition fields expected by
the typed consumer. Its observed slots bind the committed SYS-004 run's exact
package, one detector-admitted board 205, successful ESP32-S3 board-info gate,
canonical detector-output parsing, safe disabled boot, same-origin API
observation, later same-boot WebSocket observation, cleanup, and semantic
redaction. Private runtime artifacts remain outside the repository.

The typed command
`capture-operator-evidence --profile release --require-redaction-passed`
accepted the root and returned a successful `bitaxe-automation-result-v1`
envelope. The focused automation and parity Bazel suites also passed.

The share-outcome and safe-stop slots are explicitly deferred. The root records
that no mining ran, no share outcome was observed, and safe no-op boot is not
production mining safe-stop evidence.

## Conclusion

The current canonical typed operator surface admitted a single detector-gated
Ultra 205 and completed its exact-package, boot, HTTP, WebSocket, privacy, and
cleanup boundaries. This closes the typed migration's fresh-schema evidence gap
and supports `workflow` verification for `REL-09`.

## Non-claims and residual risks

This result reprojects immutable closed facts and performs no new device or
network effect. It does not verify credentials during mining, settings
durability, ASIC or Stratum behavior, shares, production safe stop, voltage,
fan, thermal or power control, partitions, rollback, OTA/recovery, non-205
boards, direct UART, pin manipulation, or release readiness.
