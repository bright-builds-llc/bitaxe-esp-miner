# Parity work closure

- Parity row: `STAT-003`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `e898f57714b2b5576463d2453b5ba8c282f1857d720d93d71364e7283cf42ef3`
- Active task: `task-parity-stat003-scoreboard`

## Closure reason

The software-only correction completed at
`4594760b08e606959d952a1fc7803095967e5bf2`. Restart persistence now compares
the exact pre-restart scoreboard after projecting only difficulty through the
pinned one-decimal NVS codec. Count, ordering, job ID, extranonce2, ntime,
nonce, and version bits remain exact, and both same-boot repeat pairs must still
match byte-for-byte through their normalized digests.

The projection uses exact IEEE-754 rational rounding with ties-to-even rather
than JavaScript `toFixed`, which disagrees with Rust/C on exact midpoints such
as `1.25`. Source inventory binds the Rust and upstream write/reload formats.
Positive and negative pure and real-child regressions pass alongside all
mandatory repository gates.

During verification, `just test` exposed that Bazel recursively scanned
Git-ignored `scratch/` and `target/` trees. The committed `.bazelignore` mirrors
the repo-local generated/ignored roots, prevents protected evidence traversal,
and restores the exact full test command to normal completion.

No hardware or protected input was used. `STAT-003` remains `implemented`
because this deterministic correction does not itself establish live device
restart persistence or authorize re-evaluation of attempt-005.

## Next safe action

Create a separate immutable evidence plan that binds the corrected clean pushed
source. It may either re-evaluate eligible existing protected evidence under an
explicit privacy/publication contract or authorize a fresh bounded hardware
ordinal with the complete detector, safety, recovery, cleanup, and promotion
gates. This closure grants neither action.

## Non-claims

This closure does not verify live scoreboard persistence, promote `STAT-003`,
reinterpret or publish attempt-005, authorize attempt-006, or verify arbitrary
profiles/pools, other ASICs/boards, unbounded mining, OTA, recovery, or release
readiness.
