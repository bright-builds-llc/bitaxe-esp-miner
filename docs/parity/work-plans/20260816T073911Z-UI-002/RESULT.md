# Parity work result

- Parity row: `UI-002`
- Final status: `verified`
- Implementation commit: `7c5ddac733242acb20e6cd09b977c3b55b9844bf`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The repository-only projector ran once from clean synchronized pushed
implementation commit `7c5ddac733242acb20e6cd09b977c3b55b9844bf`
against the exact committed API-009 display-UAT and command-effects
projections. It atomically published
`docs/parity/evidence/ui002-screen-flow/screen-flow-projection.json` with
SHA-256
`86a9887ebac787297ff76dacbaaf56715c88584647c425268b9e76d87aa5b5fe`,
mode `0644`, and no surviving candidate. The independent Rust validator
accepted the final document.

The closed projection binds board 205, captured package source
`522d5abda3af659a45691c2d4a7c03712573fb80`, current source, pinned reference,
one IDENTIFY request, machine-confirmed render and natural clear,
operator-confirmed visible render and clear, exact build and USB admission,
safe stop, cleanup, disabled mining and hardware control, and passed
redaction. It also admits the six priority pages, two bounded intro pages,
four carousel pages, 500 ms evaluation cadence, 3,000 ms intro dwell,
10,000 ms carousel dwell, eight notification-mask states including paused,
IDENTIFY override, new-block statistics pinning, bounded private frames,
side-effect-free projection, retained ownership, change-only rendering,
priority-to-power visibility, and display-failure isolation.

Four core screen-flow paths are byte-identical across the captured and current
commits. The retained runtime owner is bound through exact unique semantics at
both commits, and the pinned reference is bound through exact unique screen
flow semantics. Verification included focused Rust contract and TypeScript
projector tests, incomplete UAT and task/plan/source/reference/worktree/process
rejections, typed CLI and failure boundaries, generated contract equality,
explicit Bazel runfile ownership, the ordered Cargo checks, Bright Builds,
all 45 Bazel test targets, parity and progress, firmware packaging, reference
cleanliness, redaction, immutable digests, file mode, candidate absence,
sensitive-field checks, and diff checks.

## Conclusion

The existing exact-package API-009 attempt proves one physical IDENTIFY overlay
rendered and naturally cleared through the production screen owner with both
machine and operator confirmation. Current deterministic tests and the sealed
source/reference admission prove the full bounded priority, intro, carousel,
notification, timing, privacy, and retained-owner decisions. The complete
typed quorum therefore supports promoting only `UI-002` from `implemented` to
`verified`.

## Non-claims and residual risks

This result does not claim physical observation of every page, dwell,
notification, new-block state, private value, input path, animation, bitmap,
QR code, pixel geometry, brightness, another board, mining, soak, update,
recovery, or release readiness. The timing values are milliseconds by contract;
the result contains no voltage or millivolt observation, and core-voltage
commands remain a separate millivolt-typed surface. No detector, USB/serial,
network/HTTP, credential, physical display, operator checkpoint, mining,
settings, restart, OTA, recovery, hardware-control, UART, BAP, or electrical
action ran. Future changes to the admitted screen-flow paths require fresh
compatibility evidence rather than inheriting this result.
