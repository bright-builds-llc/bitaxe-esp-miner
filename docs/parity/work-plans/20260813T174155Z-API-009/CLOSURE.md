# Parity work closure

- Parity row: `API-009`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `89f3013e6da22e7210a430f4e4ca4bf840463c564c7aa737d078ada4ab7363a4`
- Active task: `task-parity-api009-command-effect-evidence-audit`

## Closure reason

The attempt-012 orchestration blocker is fixed in software. One checked typed
budget replaces the fixed 810-second campaign parent and 920-second fixture
deadlines. It covers the version probe; three USB commands with one allowed
retry each; retry, post-flash, monitor-admission, and final-cleanup recovery;
the 600-second command-effects observation; the 180-second child terminal
grace; and process termination. The derived parent is strictly larger than the
3,250-second maximum child envelope, and the fixture remains larger than both.

Cross-language source assertions bind these components to the Rust and fixture
implementations. A scaled real-child regression proves cleanup can be written
before the parent guard. Closed, missing, and malformed campaign recovery files
prove outer `timeout` remains the primary category while only validated facts
are projected; incomplete artifacts safely yield false facts.

Focused targets, every mandatory software/privacy/reference gate, and the real
ESP firmware build pass. No hardware-capable command ran under this plan.
API-009 remains `implemented` because no new five-command hardware quorum was
produced.

## Next safe action

Commit and push this software fix. Then use a fresh clean selector. A later
hardware attempt requires its own immutable bounded contract and must not reuse
attempt-012 paths or protected artifacts.

## Non-claims

This closure does not verify or promote API-009, claim device cleanup or
command effects beyond prior closures, authorize attempt-013, access protected
attempt data, or expose device, USB, network, credential, process, sensor, or
raw-trace material.
