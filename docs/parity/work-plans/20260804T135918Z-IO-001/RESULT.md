# IO-001 work result

- Parity row: `IO-001`
- Final status: `implemented`
- Implementation commit: `b15073c9d51698cd86eb6ee750271ba5dd1e8887`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware attempts: none

## Evidence and verification

The shared Ultra 205 I2C owner now uses the pinned reference transfer policy:
a 500 ms transaction timeout, three attempts, and a 10 ms FreeRTOS delay after
every failed attempt, including the terminal failure. The pure retry function
preserves the final driver error and returns immediately on success.

All six transfer shapes use the same helper: display read, write, write-read,
and transaction operations; INA260/EMC2101 register reads; and EMC2101/DS4432U
register writes. Existing compile-time address capabilities, the single runtime
owner, I2C0, GPIO 47/48, 400 kHz speed, and contextual error messages remain in
place. Internal SDA/SCL pull-ups are now explicit in the bus configuration.

Focused tests prove exact constants, first-attempt success without delay,
eventual success with one delay per preceding failure, terminal failure after
three attempts, final-error preservation, terminal-delay parity, and absence of
a direct transfer bypass. The real ESP-IDF firmware target builds successfully.

The following gates passed on the implementation commit:

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bun scripts/bright-builds-check.ts all`
- `just test` (all 29 Bazel test targets passed)
- `just parity`
- `just parity-progress`
- `just verify-redaction`
- `just verify-reference`
- `git diff --check`

## Conclusion

I2C initialization and the shared transfer policy are implemented with typed
unit and workflow evidence while preserving the existing safety boundaries.

## Non-claims and residual evidence gap

`IO-001` remains `implemented`, not `verified`. Existing hardware-smoke evidence
predates this retry change and does not prove live transient-fault recovery,
terminal timeout behavior, concurrent shared-bus load, or DS4432U actuation.
No hardware, credentials, network, mining, voltage, fan, thermal, power, OTA,
recovery, direct UART, or pin action was used or claimed.
