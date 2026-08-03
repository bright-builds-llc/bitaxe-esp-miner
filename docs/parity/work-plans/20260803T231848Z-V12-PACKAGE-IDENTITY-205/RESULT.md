# V12-PACKAGE-IDENTITY-205 typed-evidence result

- Parity row: `V12-PACKAGE-IDENTITY-205`
- Final status: `verified`
- Evidence source commit: `66cf184943d7f3a5aedfc99e692a9f500707de9e`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Board: Ultra 205

## Evidence and verification

The committed [typed version projection](../../evidence/sys004-version-reporting/version-projection.json)
uses schema `bitaxe-version-evidence-v1` and directly closes the typed-migration
gap recorded on this row. The Rust evidence validator accepted it, and its
closed fields bind:

- board 205 and exact clean package source commit
  `66cf184943d7f3a5aedfc99e692a9f500707de9e`;
- pinned reference commit
  `c1915b0a63bfabebdb95a515cedfee05146c1d50`;
- one SHA-256 package-manifest identity;
- safe boot with mining and hardware control disabled;
- one same-origin API observation whose build label, static-asset version,
  extended provenance, and ESP-IDF version match the manifest; and
- a later WebSocket projection from the same boot, with equal-or-later positive
  revision and version provenance matching the API projection.

The [SYS-004 result](../20260803T001834Z-SYS-004/RESULT.md) records that the
projection came from a fresh package, one detector-admitted Ultra 205, a
complete exact-package flash/boot/observation workflow, private mode-`0700`
attempt storage, mode-`0600` raw artifacts, and passing redaction. The evidence
source commit is an ancestor of this repository state, and the reference
submodule remains at the projection's pinned commit.

Focused verification passed:

- typed version-evidence validation using the Rust contract validator;
- 11 `bitaxe-api` runtime boot-attestation tests;
- 8 `xtask` package-manifest tests; and
- the Bazel `bitaxe-api`, `bitaxe-automation-contracts`, and `xtask` suites.

The regressions reject stale or mismatched package/reference identity,
malformed manifests, wrong application digests, mixed sessions or boot
ordinals, non-monotonic observations, incomplete readiness, invalid typed
evidence, and missing or duplicate package artifacts.

## Conclusion

The exact admitted Ultra 205 package and its observed HTTP/WebSocket runtime
surfaces report source and pinned-reference provenance consistent with the
package manifest. This satisfies the current row's explicit requirement for an
exact-package `bitaxe-version-evidence-v1` hardware smoke and supports
`workflow,hardware-smoke` verification for `V12-PACKAGE-IDENTITY-205`.

## Non-claims and residual risks

This result reuses only closed facts from an immutable committed projection; it
does not expand the completed SYS-004 attempt's effects. It does not claim
configuration persistence, operator-snapshot or runtime-health substance,
network longevity, mining, ASIC behavior, voltage/fan/power or thermal
controls, partitions, rollback, OTA recovery, other boards, direct UART, pin
manipulation, or release readiness. Raw detector, USB, serial, origin, network,
API, WebSocket, and credential material remains private and uncommitted.
