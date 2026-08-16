# Parity work result

- Parity row: `UI-001`
- Final status: `verified`
- Implementation commit: `5cc823b699c33f361e97913f1aec60109e42a6a1`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The repository-only projector ran from clean synchronized pushed implementation
commit `5cc823b699c33f361e97913f1aec60109e42a6a1` against the exact committed
API-009 display UAT and command-effects projections. It atomically published
`docs/parity/evidence/ui001-display-behavior/display-behavior-projection.json`
with SHA-256
`8b6832e74024e3c36018dd60be86e35a39190530cf6502debcdf7f5c3b2246a3`,
mode `0644`, and no surviving candidate. The independent Rust validator and
repository redaction verifier accepted the final document.

The closed projection binds board 205, captured package source
`522d5abda3af659a45691c2d4a7c03712573fb80`, current projector source,
pinned reference source, one IDENTIFY request, machine-confirmed render and
natural clear, operator-confirmed visible render and clear, exact build and USB
admission, safe stop, cleanup, disabled mining and hardware control, and passed
redaction. It also proves the four display-owned implementation files are
byte-identical across the captured and current commits, admits the retained
runtime owner through unique semantic fragments at both commits, and admits
the pinned upstream inversion, rotation, timeout, identify-priority, and power
semantics.

Verification included focused Rust contract and TypeScript projector tests,
missing/duplicate/drifted source rejection, incomplete UAT rejection, typed
failure and invocation boundaries, generated contract equality, independent
validation, explicit Bazel runfile ownership, the ordered Cargo checks, Bright
Builds, all 45 Bazel test targets, parity and progress, firmware packaging,
reference cleanliness, redaction, immutable plan/task binding, file mode,
candidate absence, and diff checks.

## Conclusion

The existing exact-package API-009 attempt already proved one physical
IDENTIFY frame appeared and naturally cleared, with independent machine and
operator confirmation. The new source-bound projection joins that physical
observation to the unchanged current display implementation and pinned
reference behavior. The complete typed quorum therefore supports promoting
only `UI-001` from `implemented` to `verified`.

## Non-claims and residual risks

This result does not claim measured pixel geometry or brightness, physical
testing of every rotation or inversion, measured timeout duration or current
draw, physical button behavior, UI-002 content parity, another board, mining,
soak, update, recovery, or release readiness. It contains no voltage or
millivolt observation: core-voltage commands remain separately typed in
millivolts, while this projection is display-only. No detector, USB/serial,
network/HTTP, credential, display, operator-checkpoint, mining, settings,
restart, OTA, recovery, hardware-control, UART, BAP, or electrical action ran.
Future changes to the admitted display paths require fresh compatibility
evidence rather than inheriting this result.
