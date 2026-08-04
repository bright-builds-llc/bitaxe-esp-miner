# BAP-002 work result

- Parity row: `BAP-002`
- Final status: `implemented`
- Implementation commit: `c88ae0470e41716f011be2162d0a084bd1ac02c4`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware attempts: none

## Evidence and verification

The independent pure Rust implementation defines all ten pinned BAP command
tokens and eighteen request/subscription parameter tokens. It calculates the
sentence-body XOR checksum, emits canonical `$BAP,...*XX\r\n` messages, and
rejects complete messages at or above the 256-byte buffer boundary. The parser
preserves bounded response-only parameter tokens and returns only closed,
value-free failure categories.

The ingress contract preserves the reference compatibility rules: checksum-free
`SUB` and `UNSUB` messages are admitted, a mismatched `SUB` checksum is admitted
with an explicit compatibility disposition, and other missing or mismatched
checksums fail closed. Identical complete frames are suppressed strictly inside
the one-second window. Retained bytes, parameter tokens outside the known
vocabulary, values, and credential-bearing setting intents never appear in
`Debug` output.

Pure command planning covers connected/AP request restrictions, exact supported
request response tokens and values, subscription default/positive intervals,
the five-minute subscription timeout, unsubscribe acknowledgements, exact BAP
error values, and validated setting intents. It describes but does not execute
persistence, restart, frequency, voltage, fan, credential, or block-state
effects. Request snapshots and setting intents use custom redacted diagnostics.

The following gates passed on the implementation commit:

- twelve focused BAP vocabulary, golden, checksum, bound, malformed-input,
  duplicate, request, subscription, setting, AP-mode, and redaction tests;
- the focused `//crates/bitaxe-core:tests` Bazel target;
- focused strict `bitaxe-core` Clippy;
- the simplification and file-size review, with wire and semantics kept in
  separate sub-500-line modules;
- `cargo fmt --all`;
- `cargo clippy --all-targets --all-features -- -D warnings`;
- `cargo build --all-targets --all-features`;
- `cargo test --all-features`;
- `bun scripts/bright-builds-check.ts all` with zero findings;
- `just test` with all 30 Bazel test targets passing and the ESP32-S3 artifact
  produced;
- `just parity` with no validation errors and `just parity-progress`;
- `just verify-redaction`, `just verify-reference`, sensitive-value review, and
  `git diff --check`.

## Conclusion

BAP framing, checksum behavior, bounded ingress, command vocabulary, request
projection, subscription decisions, setting validation, and public error
behavior are implemented in the functional core with unit and synthetic golden
evidence. No new dependency was added, and no imperative hardware capability
was introduced.

## Non-claims and residual evidence gap

`BAP-002` remains `implemented`, not `verified`. There is no firmware BAP UART
adapter, accessory session, task lifecycle, live request/response exchange,
subscription delivery, persistence proof, restart proof, or interoperability
evidence. Those surfaces remain owned by `BAP-001` and require a separately
authorized electrical/accessory path before verified promotion. No accessory,
device hardware, credentials, observed network values, external request,
mining, ASIC traffic, frequency/voltage/fan/power effect, OTA, recovery, direct
UART, or pin manipulation was used or claimed.
