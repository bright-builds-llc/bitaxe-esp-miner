# API-010 work result

- Parity row: `API-010`
- Final status: `implemented`
- Implementation commit: `b65d19c27a92fa486597d9af3457860600e600af`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware attempts: none

## Evidence and verification

The Rust-owned theme contract now models the pinned upstream dark color scheme
and complete accent-color defaults. The typed GET projection reads the
confirmed settings snapshot and omits malformed stored accent colors. The POST
planner enforces the upstream request-size bound, rejects malformed JSON,
ignores unknown or wrong-typed fields, preserves partial-update behavior, and
produces the exact success response.

The firmware registers private-network-gated `/api/theme` GET and POST routes.
Accepted writes run under the existing serialized settings transaction, commit
once, independently reload from NVS, reconcile the requested values, and publish
the complete confirmed snapshot. Route ownership, AxeOS service usage, and
captured-response fixtures cover both methods.

The following gates passed on the implementation commit:

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bun scripts/bright-builds-check.ts all`
- `just test` (all 28 Bazel test targets passed)
- `just parity`
- `just parity-progress`
- `just verify-redaction`
- `just verify-reference`
- `git diff --check`

Focused verification passed all 51 configuration tests and 226 API tests,
strict focused Clippy, the parity tests, the real ESP-IDF firmware build, and
API comparison with 103 schema, 51 captured-response, and 38 static-route
checks. The pinned reference remained clean.

## Conclusion

Theme GET/POST behavior and confirmed NVS persistence are implemented with
typed unit, golden, and API-comparison evidence. The change introduces no new
hardware effect, credential use, or network client behavior.

## Non-claims and residual evidence gap

`API-010` remains `implemented`, not `verified`. No detector-gated live route or
reboot-durability evidence was collected, and installed AxeOS browser behavior
remains owned by `UI-004`. No hardware, credentials, mining, voltage, fan,
thermal, power, OTA, recovery, direct UART, or pin action was used or claimed.
