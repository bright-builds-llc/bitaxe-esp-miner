# Parity work plan

- Run ID: `20260818T160811Z-ASIC-09`
- Parity row: `ASIC-09`
- Initial status: `implemented`
- Source commit: `8b57eecb384ffade0948323a869cb23e8acc03b2`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-asic09-mode-separation`

## Selection

The clean `main` worktree equals `origin/main`, the pinned reference is clean,
and the selector reports no open plan. Candidate order is `SELF-001`,
`BAP-002`, `STAT-003`, then `ASIC-09`.

`SELF-001` remains unavailable because no production-safe full self-test route
exists for its hardware regression. `BAP-002` remains dependency- and safety-
blocked by unfinished `BAP-001` firmware/UART lifecycle plus unauthorized
external electrical UART work. `STAT-003` remains environment-blocked after
attempt-004's distinct `network_unavailable` result; its active task prohibits
an unchanged retry without an objective protected pool/network recovery signal.
`ASIC-09` is therefore first actionable.

The row's missing live production proof now exists in already accepted,
independently validated exact-package evidence. ASIC-002 proves mining-ready
initialization and retained production UART; ASIC-003 proves production-ready-
gated live work, qualified result, and accepted submit; ASIC-004 and ASIC-005
prove the corresponding production result and bounded UART chain. Current pure
tests prove diagnostic modes require exact compile-time acknowledgements,
otherwise fail closed, and the production executor contains no diagnostic work
variant.

Loaded guidance materially includes `AGENTS.md`, `AGENTS.bright-builds.md`,
managed architecture/code-shape/verification/testing/Rust standards, the
active tracker/checklist, and bounded lessons for protected evidence,
private-first classification, earliest failure, source identity, standing
authorization, and agent-runtime timing. Active lesson inputs total 31,758
bytes, so headings were inventoried and relevant complete blocks loaded;
less-relevant blocks were omitted under the deterministic budget. The August
lesson-audit baseline remains current and no new audit trigger is due.

## Scope and non-scope

Advance only `ASIC-09`. Produce a source-bound evidence summary that joins the
four accepted public ASIC projections, their independent Rust validators,
current diagnostic-mode admission tests, current production-command/executor
tests, current source digests, pinned reference behavior, and explicit
non-claims. No new runtime implementation or projector is needed because the
typed separation and accepted hardware behavior already exist.

Bind these accepted projections exactly:

- ASIC-002 initialization:
  `eee750561a7c1dcec1a5698b1e5827d3f1508d43655c3c4aa237097338dcf8d4`
- ASIC-003 work send:
  `447af65ae9e6cd5cc2199ef639ff8e0fa7f63d4c9708570bd66781c5a162e80c`
- ASIC-004 result parsing:
  `e99c054c4d660155d5c2b1ee38d3f17aed5ae7101e7e4a5fd1c6451d1b48b7c7`
- ASIC-005 serial transport:
  `bad828db694ee59c4ef3d77b2e58ef89e0195ef382526b97912d0a71e882ad69`

The evidence summary may contain only repository paths, commits, digests,
closed labels, booleans, counts, and command outcomes. It must contain no raw
ASIC frames, nonce/work/share values, pool or credential data, endpoints,
ports, USB/network identity, telemetry, logs/payloads, commands, PIDs, traces,
or protected identifiers.

This plan authorizes local tests, committed-public evidence reads,
documentation, build/package, Git commit, and push only. It authorizes no
credential or protected-attempt access, detector, device/USB/network runtime,
flash, monitor, mining, restart, recovery, hardware attempt, fault injection,
external UART/BAP, pins, or electrical work.

## Implementation

- [ ] Independently validate all four accepted ASIC projections and bind their
      complete live initialization/work/result/UART chain.
- [ ] Run current diagnostic admission, production-command, and production-
      executor separation tests plus source/reference and privacy review.
- [ ] Produce `summary.md`, `WORKLOG.md`, and `RESULT.md` with exact digests,
      conclusions, and non-claims.
- [ ] Commit the evidence as `SOURCE_COMMIT`, transition only ASIC-09, sync
      progress, archive this task, final-gate, and push.

## Verification and promotion

Focused verification is:

- `cargo test -p bitaxe-asic adapter_gate`
- `cargo test -p bitaxe-asic production`
- `cargo test -p bitaxe-firmware production_executor_module_never_references_diagnostic_work`
- the four existing Rust evidence validators over absolute projection paths
- `just verify-reference`
- `just package`

The mandatory ordered gates are `cargo fmt --all`, `cargo clippy --all-targets
--all-features -- -D warnings`, `cargo build --all-targets --all-features`,
`cargo test --all-features`, `bun scripts/bright-builds-check.ts all`, `just
test`, `just parity`, and `just parity-progress`, plus projection digests/modes,
redaction, file-size, selector, sensitive-value, source-diff, and final diff
checks.

Promotion requires accepted evidence for one Ultra 205 BM1366 production chain
with all nine initialization steps, exactly one chip, mining-ready completion,
retained production UART, a required production-ready gate, typed production
work, qualified parsed/correlated result, accepted response, safe stop, cleanup,
trusted identity/safety, current source compatibility, independent validation,
and redaction.

Current tests must also prove that absent, incomplete, or incorrect diagnostic
compile-time acknowledgements select fail-closed mode; exact diagnostic
acknowledgements select only their diagnostic modes; production commands contain
only production work/result variants; and the production executor cannot
reference diagnostic work.

On success create `RESULT.md`, commit evidence without checklist change and
save that full commit as `SOURCE_COMMIT`; transition only `ASIC-09` to
`verified` with `unit,golden,workflow,hardware-smoke,hardware-regression`, sync
progress, archive only this task, run final gates, and push. On failure create
`CLOSURE.md`, leave ASIC-09 `implemented`, and do not sync unchanged progress.

## Non-claims

This plan does not verify arbitrary diagnostic builds, frequency transitions,
voltage/fan/thermal behavior, nonzero version-mask or multi-midstate breadth,
arbitrary-load serial behavior, other ASICs/boards, arbitrary pools/profiles,
unbounded mining, OTA/recovery, or release readiness. It does not promote
ASIC-10, ASIC-11, ASIC-12, STR-08, or STR-09.
