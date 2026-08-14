# Parity work plan

- Run ID: `20260814T042002Z-API-009`
- Parity row: `API-009`
- Initial status: `implemented`
- Source commit: `f8a0855713d65930ab9575804e93300d84b07678`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api009-command-effect-evidence-audit`
- Continues plan: `docs/parity/work-plans/20260814T035526Z-API-009/PLAN.md`

## Selection

The clean synchronized selector has no open plan and ranks API-009 first, so
no candidate is skipped. Attempt-015 admitted the exact package and one device,
completed both flash writes, trusted runtime identity and protocol admission,
and confirmed safe stop and USB cleanup. It then failed closed because the
first receive-ingress candidate was one malformed 1,490-byte line; the next
144 markers were accepted at 2,516–2,578 bytes.

The source matches that boundary exactly. After opening or reacquiring the
receive-only reader, `observe_receive_only_inner` forwards the first available
bytes directly to the ephemeral callback. The campaign callback feeds that
same chunk to both the strict campaign analyzer and network tracker, so neither
consumer has a proved initial line boundary. The strict analyzer therefore
treats a pre-open fragment ending at the first observed newline as a complete
marker. This software-only plan establishes the missing boundary without
relaxing marker validation.

## Scope and non-scope

Add one private resettable line-admission type under the existing USB session.
For ephemeral chunk observation only, discard bytes through the first newline
after every receive-only reader open or reopen, then forward each remaining
chunk unchanged. The callback remains the single fan-out boundary, ensuring
campaign and network consumers see the identical admitted suffix. Empty
suffixes must not trigger the callback. Persistent ordinary monitor capture
must remain byte-for-byte unchanged.

Add red-to-green tests for a malformed leading marker fragment followed by a
valid marker, a boundary split across chunks, a full first line conservatively
discarded before later lines pass unchanged, and reset on reader reacquisition.
Keep all post-admission malformed UTF-8, JSON, schema, and evidence-contract
failures terminal. Do not add retries, sleeps, marker exceptions, heuristic
length checks, content repair, a second analyzer, or evidence relaxation.

Do not access a credential, protected attempt artifact beyond the public
category/count/length facts already recorded, detector, USB/device/network
interface, HTTP endpoint, display, mining hardware, or public evidence path.
No flash, monitor, reset, restart, erase, OTA, power cycle, direct UART,
pin/pad/GPIO action, attempt-016, or hardware effect is authorized.

## Implementation and verification

- [ ] Commit and push this immutable software-only plan/task checkpoint before
      editing production or test source.
- [ ] Turn the real receive-session seam red for an untrusted initial fragment
      and prove the same admitted suffix is the only chunk visible to its
      callback consumers.
- [ ] Add the resettable boundary admission in a focused USB submodule and
      reset it on every reader open/reopen.
- [ ] Prove ordinary retained monitor capture is unchanged and post-admission
      malformed markers still fail closed.
- [ ] Run focused device-session, flash campaign/framing/network,
      command-effects, automation, and real-process targets plus every
      mandatory, privacy, reference, firmware, selector, digest,
      sensitive-output, and diff gate.
- [ ] Close with API-009 still `implemented`; require a later clean selector
      and separate immutable contract before any hardware ordinal.

Before plan commit and software closure commit, run in order: Cargo format,
strict Clippy, all-target build, all-feature tests, Bright Builds, `just test`,
`just parity`, and `just parity-progress`. Also run focused Bazel targets,
`just verify-redaction`, `just verify-reference`, `just build`, immutable plan
digest, unique task binding, selector ownership, source-sensitive scans,
`git diff --check`, and complete diff review.

Success means the receive-only session explicitly owns line admission: every
new reader discards exactly its untrusted leading fragment through one newline,
all subsequent bytes remain unchanged and shared by both live analyzers, and
strict evidence validation begins only after that boundary. This plan makes no
hardware or API-009 parity-verification claim.
