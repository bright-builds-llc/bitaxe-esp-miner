# Parity work result

- Parity row: `API-009`
- Final status: `verified`
- Programmatic package source: `522d5abda3af659a45691c2d4a7c03712573fb80`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Programmatic hardware attempt: `046`
- Physical display UAT attempt: `005`

## Evidence and verification

The sealed `bitaxe-api-command-effects-evidence-v1` projection at
`docs/parity/evidence/api009-command-effects/command-effects-projection-attempt-046.json`
binds one detector-admitted Ultra 205, the exact package and workflow, and one
request each for pause, block-notification dismissal, IDENTIFY, resume, and
software restart. Claim-specific postconditions prove pause and resume
generations with runtime witnesses, dismissal state change, successful
IDENTIFY render and later non-IDENTIFY render receipts, and a reader-armed
same-device restart with changed boot session, ordinal N+1, exact build
identity, service loss, and reacquisition. Safe stop, disabled mining and
hardware control, cleanup, and redaction pass; recovery was not required.

The separately sealed `bitaxe-display-uat-evidence-v1` projection at
`docs/parity/evidence/api009-command-effects/display-uat-projection-attempt-005.json`
binds the accepted programmatic projection to one fresh physical UAT. It proves
one IDENTIFY request, successful machine render and clear receipts, exact build
and USB admission, and the user's independent confirmation that the physical
IDENTIFY frame visibly rendered and then cleared. The projection contains only
aggregate booleans, the request count, board category, a binding digest, and
redaction status.

The following promotion gates passed:

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bun scripts/bright-builds-check.ts all`
- `just build`
- `just test`
- `just parity`
- `just parity-progress`
- `just verify-redaction`
- `just verify-reference`
- diff and public-evidence review

## Conclusion

API-009 has a closed composed proof for the five command effects on one Ultra
205 exact-package session. The programmatic campaign proves the typed command
transactions and their independent runtime witnesses; the replayable physical
UAT separately confirms the two display pixel observations that software
cannot establish.

## Non-claims and residual risks

This result does not claim native USB duplex control, external UART/BAP,
arbitrary command CRUD, other boards, display pixel geometry or brightness,
repeated restart endurance, power-loss behavior, network-failure recovery,
mining performance, hardware controls, OTA behavior, factory reset, release
readiness, or any command beyond the five scoped API effects. Human readiness
and observation remain unbounded; automated effects and cleanup retain finite
safety limits.
