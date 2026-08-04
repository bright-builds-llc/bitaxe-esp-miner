# UI-002 work result

- Parity row: `UI-002`
- Final status: `implemented`
- Implementation commit: `9b2f37945b34a0e9fece56c8aa90703afda3ac63`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware attempts: none

## Evidence and verification

The pure screen owner implements the pinned priority order for self-test,
firmware update, ASIC status, overheat, welcome, and connection states; the
identify and new-block overlays; the one-shot Bitaxe and open-source intro; and
the URL, statistics, mining, and Wi-Fi carousel. It applies the pinned 500 ms
evaluation cadence, 3,000 ms intro and 10,000 ms carousel dwell intervals plus
the next evaluation tick, bounded delayed-wakeup coalescing, and one-evaluation
accepted, rejected, and work notifications.

Every frame is exactly four bounded ASCII rows for the Ultra 205 panel.
Control characters are sanitized, unavailable facts remain explicit, and
private pool, network, address, and device text is redacted from `Debug` and
has no log or evidence route. The firmware projection reads current state
without operator publication, retained-log mutation, statistics drain, mining
state mutation, or credential exposure. One retained screen owner runs beside
the display owner on the absolute 500 ms schedule, redraws only changed frames,
and feeds priority visibility into the existing display power policy.

The following gates passed on the implementation tree immediately before the
implementation commit:

- twelve focused pure screen-flow tests;
- display adapter and source-ownership Bazel tests;
- the real ESP32-S3 firmware Bazel build;
- `cargo fmt --all`;
- `cargo clippy --all-targets --all-features -- -D warnings`;
- `cargo build --all-targets --all-features`;
- `cargo test --all-features`;
- `bun scripts/bright-builds-check.ts all` with zero findings;
- `just test` with all 34 Bazel test targets passing;
- `just parity` with no validation errors and `just parity-progress` reporting
  the pre-transition baseline of 39 of 94 active rows verified (41.5%);
- `just verify-redaction`, `just verify-reference`, immutable-plan, obsolete-
  symbol, sensitive-surface, and diff checks.

## Conclusion

The bounded priority, overlay, intro, carousel, notification, private runtime
projection, retained ownership, absolute scheduling, change-only rendering,
and display-failure isolation contract is implemented with unit and workflow
evidence. This supports transitioning `UI-002` from `in-progress` to
`implemented` without a physical display claim.

## Non-claims and residual evidence gap

`UI-002` remains below `verified`. No live panel content, exact LVGL bitmap or
animation behavior, physical button/input path, mining, ASIC traffic, pool
interaction, voltage, fan, thermal, or power behavior was exercised. Network
difficulty, block height, and scriptsig remain unavailable until their correct
runtime owners publish those facts. UI-003 retains physical input ownership.
No origin, hostname, SSID, address, port, USB identity, credential, pool field,
worker, device identifier, or raw trace is included in this result.
