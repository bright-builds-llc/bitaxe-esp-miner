# Parity work result

- Parity row: `API-002`
- Final status: `verified`
- Implementation commit: `524b445ee45c986a1366cfe64d2cbcbe41178da8`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Stack-fix commit: `84b90c9e677b4def1d0ab7508e2b8e64dd08c617`
- Board: Ultra 205
- Hardware attempts under this plan: one

## Evidence and verification

The detector-gated `attempt-002` capture used the exact schema-v3 package built
from the clean pushed implementation commit. The sole detector admitted one
Ultra 205 before `just capture-system-info-evidence` performed its factory
flash and passive same-origin observations.

The committed
[`bitaxe-system-info-evidence-v1`](../../evidence/api002-system-info/system-info-projection.json)
projection has SHA-256
`6ec58fdaeb7cbad3cf103832cd3e59fe470fcb05f6f6a4d41e218ffd6378991a`.
The repository-owned Rust validator independently accepts it. Its closed facts
prove:

- the exact clean source, pinned reference, package-manifest identity, and
  board 205 were admitted;
- the fixed firmware booted without a stack-overflow, panic, or Guru Meditation
  marker and emitted repeated boot attestation and heartbeat evidence;
- one coherent boot session joins substantive HTTP revision 594 and later
  WebSocket revision 595 with both exact retained runtime-health tuples;
- all 87 unconditional names in the 94-field versioned contract are present
  with the required types in both snapshots;
- all seven block-conditional fields are absent while block notification is
  inactive, and every confirmed-setting field is present;
- the trusted same-session origin, mining-disabled state, hardware-control-
  disabled state, complete cleanup, private modes, and redaction all pass; and
- the selected device port has no remaining holder.

The exact effect-capable commands were the immutable plan's `just package`,
private `just detect-ultra205`, and conditional `just
capture-system-info-evidence` commands. Before hardware, the focused stack,
source-ownership, automation, API, and contract tests passed, followed by:

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
- `just build`
- `just package`
- package admission, continuation-aware selector, immutable-plan,
  task-uniqueness, sensitive-output, mode, holder, reference-cleanliness, and
  diff checks

## Conclusion

The exact pushed Rust firmware exposes the complete pinned system-info contract
through coherent live HTTP and WebSocket snapshots on the detector-admitted
Ultra 205, with matching retained identity evidence and the required passive
safety state. This satisfies `API-002` with typed workflow, API-comparison, and
hardware-smoke evidence.

## Non-claims and residual risks

Raw detector, USB, serial, HTTP, WebSocket, retained-log, configuration,
hostname, origin, network, credential, and process material remains only in
ignored private roots. The Wi-Fi credential file was an opaque local input,
and no pool credential was read. Conditional block fields are verified by pure
tests plus live inactive absence, not a live found-block event. The evidence
does not claim arbitrary future settings, repeated or power-loss longevity,
mining, accepted or rejected shares, ASIC work, hardware controls, network
reconnect longevity, OTA/recovery, other boards, or release readiness.
