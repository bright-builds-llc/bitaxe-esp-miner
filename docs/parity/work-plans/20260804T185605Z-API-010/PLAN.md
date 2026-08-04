# Parity work plan

- Run ID: `20260804T185605Z-API-010`
- Parity row: `API-010`
- Initial status: `implemented`
- Source commit: `fbee3ff32f903466ae4a476061b1bc543c6b1368`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api010-live-theme-durability`

## Selection

The deterministic selector reported no open plan on a clean `main` branch
matching `origin/main`. Earlier candidates were audited in order:

- `CFG-001` is closed at `stop_repeated_boundary` for its safety-critical
  frequency and voltage claim; no targeted correction exists.
- `CFG-005` deliberately excludes broad credential, mining, voltage,
  frequency, and fan writes from its production PATCH route, so the verified
  hostname subclaim cannot close the broad runtime-settings row.
- `CFG-006` requires live seeded-default and runtime proof from non-205 board
  profiles that are not available on the sole Ultra 205 hardware target.
- `NET-001` through `NET-003` require reconnect/fallback/IPv6, provisioning,
  or connection-preserving live scan evidence and their own fault/recovery
  boundaries.
- `ASIC-002` through `ASIC-005`, `ASIC-007`, `STR-001`, `STR-006`, and
  `STR-007` retain loaded hardware or mining evidence gaps; the default-profile
  soak boundary is terminal and cannot be repurposed.
- `API-002` retains full field-level production-statistics history, while
  `API-003` retains the same broad settings gap as `CFG-005`; `API-009`
  requires mining and hardware command-effect evidence.

`API-010` is the first safe actionable row. Its typed route and confirmed NVS
persistence already pass unit, golden, API comparison, and firmware-build
tests. The remaining row-owned gap is a bounded live GET/POST, one-restart
durability, and restoration transaction. Installed AxeOS browser workflows
remain owned by `UI-004` and are not required or claimed here.

## Scope and non-scope

Implement one typed private-first capture that freezes an exact package,
admits exactly one Ultra 205, confirms mining and hardware control are disabled,
reads the original `/api/theme`, writes a generated non-secret alternate theme,
confirms immediate readback, performs one normal software restart through the
authoritative `device-session reboot-live` transaction, proves the same exact
device/build and boot ordinal `N+1`, confirms the alternate theme persisted,
restores the original theme, and confirms exact restoration before publishing.

Private artifacts are mode `0600` beneath one absent-before-use mode-`0700`
attempt root. The committed projection may contain only schema names,
cryptographic identities, bounded counts/categories, the closed device-session
projection, and safe booleans. It must not contain origins, URLs, theme values,
hostnames, ports, USB identities or paths, IP/MAC/Wi-Fi/pool values,
credentials, raw HTTP/serial/log content, or process identifiers.

Do not modify the pinned reference; claim installed-browser behavior; change
credentials, mining, voltage, frequency, fan, thermal, power, ASIC, input, or
OTA state; erase or write raw partitions; use network discovery; or use direct
UART or pins. The active task records the exact command, recovery, retry,
privacy, and stop contract for the sole hardware attempt.

## Implementation

- [ ] Add a strict `verify-theme-durability` automation surface and closed v1
      evidence contract using a functional core and thin process/HTTP shell.
- [ ] Reuse the canonical device-session reboot transaction and extract shared
      closed-projection validation where that removes duplication without
      weakening the existing hostname workflow.
- [ ] Preserve the earliest typed failure through normal restoration and
      bounded exact-package recovery; publish evidence only after restoration,
      cleanup, and privacy validation.
- [ ] Add focused behavior and real-child-process tests for success, all
      non-ready terminal categories, malformed/missing projections, child
      launch/timeout, recovery precedence, and sensitive-value denial.
- [ ] Record checkpoints in `WORKLOG.md`; create `RESULT.md` only after the
      single detector-gated capture passes every promotion criterion.

## Verification and promotion

Run focused automation and contract tests, then `cargo fmt --all`, strict
Clippy, all-target/all-feature Cargo build and tests, Bright Builds checks,
`just test`, `just parity`, `just parity-progress`, semantic redaction,
reference cleanliness, immutable-plan validation, sensitive-log scanning, and
diff checks. After a clean pushed implementation, run exactly the active-task
package, detector, and `attempt-001` capture commands.

Promote only `API-010` when the typed evidence proves exact package identity,
one admitted physical device, one restart request, boot-session change,
ordinal `N+1`, immediate and post-restart theme equality, exact restoration,
mining disabled, hardware control disabled, cleanup complete, and redaction
passed. Any missing fact withholds final evidence and stops without retry.
