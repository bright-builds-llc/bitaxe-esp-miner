# Parity work closure

- Parity row: `API-009`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `549bb6e564fdf87f5ea35362b488b48b1503fe48933a995cb972a78b1237c0f5`
- Active task: `task-parity-api009-command-effect-evidence-audit`

## Closure reason

The bounded public-evidence audit found no complete genuine quorum for the five
combined command effects. Committed Ultra 205 serial evidence proves only that
the identify and block-dismiss routes register; it contains no trusted request,
physical identify render/clear observation, active block-found notification, or
dismissal postcondition. The user's newly visible display content is not
identify-command evidence because it was not causally bound to that request or
its 30-second lifecycle.

Current production source exposes a stronger implementation precondition:
`BlockFoundNotificationState` initializes to zero/false and the dismiss route
can write only the next false state. No production nonce/result owner raises
`block_found` and `show_new_block`; true-state construction exists only in pure
tests. Upstream raises the state only when a nonce difficulty meets or exceeds
network difficulty. There is therefore no bounded, non-synthetic repository
contract that can create the required active state on the device. A diagnostic
injector would manufacture evidence, and waiting for a real network-difficulty
block is unbounded.

Pause/resume also lacks command-correlated hardware evidence while actively
mining, and physical identify still requires trusted observable-device evidence.
Generic software-restart evidence does not close the other effects. Because
the row is conjunctive, partial restart, identify, or mining commands cannot
support promotion and would add hardware effects without resolving the blocker.

## Terminal unblock conditions

Resume API-009 only after all of the following exist:

1. production code raises block count and the visible notification from a
   qualified current-generation nonce/result using upstream-compatible network
   difficulty semantics;
2. a bounded accepted evidence contract can enter an active notification state
   without a diagnostic or synthetic setter, then prove the public dismiss
   response and false postcondition while preserving the block count;
3. a trusted capture can causally bind the identify request to the physical
   30-second render and clear lifecycle; and
4. detector-gated active-mining evidence proves pause and resume intent effects,
   safe state transitions, restoration, cleanup, and redaction.

## Non-claims

This closure does not verify or promote API-009, claim any command hardware
effect, or authorize an ad hoc retry. It does not inspect protected evidence,
credentials, or network state and does not interact with USB or hardware. Pure
response/effect tests and registered routes remain implemented evidence only.
