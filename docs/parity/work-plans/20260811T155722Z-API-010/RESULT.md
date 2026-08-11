# Parity work result

- Parity row: `API-010`
- Final status: `verified`
- Implementation commit: `f2520d1e7c12f49dc376d528ddf78b6f9c5391c5`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Board: Ultra 205
- Hardware attempts under this plan: one

## Evidence and verification

The detector-gated attempt-012 capture used the exact schema-v3 package built
from the clean pushed implementation commit. The sole detector admitted one
Ultra 205 before `just verify-theme-durability` performed its typed transaction.

The committed
[`bitaxe-theme-durability-evidence-v1`](../../evidence/api010-theme-durability/theme-durability-projection.json)
projection has SHA-256
`fbf93cd115e1c99cd1c727b2e4536c49f571b69172fc94228d4942181d005288`.
Its closed facts prove:

- the exact source and pinned reference package was admitted on board 205;
- the original theme was read, a generated alternate theme was POSTed, and its
  immediate readback matched;
- an armed reader observed serial delivery before the sole normal software
  restart request, whose HTTP response was received;
- service loss and exact-build application recovery occurred on the same
  physical device with a changed boot session and boot ordinal `N+1`;
- post-restart serial delivery was correlated and the persisted theme matched;
- the original theme was restored and confirmed;
- mining and hardware control remained disabled; and
- cleanup, holder release, artifact modes, semantic redaction, and public
  sensitive-value scans passed.

The exact effect-capable commands were the immutable plan's `just package`,
private `just detect-ultra205`, and conditional `just
verify-theme-durability` commands. The focused automation, device-session, CLI,
and redaction regressions passed, followed by:

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bun scripts/bright-builds-check.ts all`
- `just test`
- `just parity`
- `just parity-progress`
- `just verify-redaction`
- `just verify-reference`
- continuation-aware selector, immutable-plan, sensitive-output, mode, holder,
  and diff checks

## Conclusion

The exact pushed Rust firmware preserves the theme across one attributed normal
software restart on the detector-admitted Ultra 205 and restores the original
theme afterward. This satisfies the `API-010` route, persistence, restart, and
restoration claim with typed workflow and hardware-smoke evidence.

## Non-claims and residual risks

Raw detector, USB, serial, HTTP, theme, hostname, origin, network, credential,
and process material remains only in ignored private roots. The Wi-Fi credential
file was an opaque local input, and no pool credential was read. The evidence
does not claim installed-browser rendering, browser storage behavior, arbitrary
theme schema extensions, repeated or power-loss durability, mining, ASIC work,
hardware controls, networking longevity, OTA/recovery, other boards, or release
readiness.
