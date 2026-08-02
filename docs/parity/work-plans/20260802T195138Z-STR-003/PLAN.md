# Parity work plan

- Run ID: `20260802T195138Z-STR-003`
- Parity row: `STR-003`
- Initial status: `implemented`
- Source commit: `470b5e133c9fc5d686f93e3c8f2d0da6a922cd99`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-str-003-mining-job-verification`

## Selection

`bazel run //tools/parity:report -- next-item --format json` reported no open
plan. It ranked the following candidates before `STR-003`; each is ineligible
for this invocation:

- `CFG-001`: the remaining frequency/voltage-default claim is explicitly
  safety-critical and lacks a row-specific authorized hardware contract.
- `CFG-005`: complete API PATCH and firmware NVS-adapter behavior remains
  absent, so the existing pure update model cannot close the broad runtime row.
- `NET-001`: retry reasons, fallback timing, IPv6, and long-running reconnect
  parity require network/hardware evidence without a row-specific task gate.
- `ASIC-002`: full BM1366 initialization and its active safety prerequisites
  retain explicit hardware-regression gaps.
- `ASIC-003`: production work dispatch remains below verified without
  row-specific live hardware evidence.
- `ASIC-004`: valid live nonce/result parsing and share outcomes remain below
  verified without row-specific live hardware evidence.
- `ASIC-005`: accepted serial transport under mining load remains below
  verified without row-specific live hardware evidence.
- `ASIC-007`: live frequency transition is safety-critical and lacks an
  authorized hardware-regression contract.
- `STR-001`: the live socket adapter and hardware-backed lifecycle evidence
  remain incomplete.

`STR-003` is the first bounded software-verifiable candidate. Its Ultra 205
surface is deterministic construction of typed BM1366 work from parsed
Stratum notify, extranonce, difficulty, and version-mask state; live socket,
ASIC dispatch, result, share, and hardware behavior are owned by other rows.

## Scope and non-scope

Make the committed reference-derived mining-job fixture executable and prove
the complete Ultra 205 construction boundary: extranonce2 generation,
coinbase concatenation and double SHA-256, ordered Merkle folding, header-field
endianness, base version handling, pool difficulty, clean-jobs state, and
negotiated version-mask retention through `MiningWork`.

Do not modify the pinned reference tree, copy GPL source expression into MIT
code, open a network connection, access credentials, touch hardware, flash
firmware, alter live production orchestration, or claim ASIC dispatch, raw UART
framing, nonce validity, share acceptance/rejection, timing, or production
mining parity. Multi-midstate behavior for non-BM1366 ASIC transports remains
outside this Ultra 205 row claim.

The plan follows the repository task/archive and evidence-integrity guidance,
the local software-only authorization boundary, and the Bright Builds
architecture, code-shape, verification, testing, and Rust standards loaded for
this run.

## Implementation

- [ ] Replace descriptive fixture placeholders with concrete, independently
      derived vectors tied to the pinned reference tests and `STR-003`.
- [ ] Add fixture-driven golden tests for hashing, Merkle folding, extranonce2,
      and every BM1366 work-field byte plus retained mining metadata.
- [ ] Add or strengthen behavior-focused rejection coverage for malformed
      construction inputs discovered by the fixture audit.
- [ ] Record exact commands, evidence, non-claims, and residual risks in
      `WORKLOG.md` and a terminal `RESULT.md` if promotion criteria pass.

## Verification and promotion

Focused acceptance commands:

- `cargo test -p bitaxe-stratum mining_job --all-features`
- `bazel test //crates/bitaxe-stratum:tests`
- `bazel run //scripts:verify_reference_clean`
- `jq empty crates/bitaxe-stratum/fixtures/v1/mining-job-cases.json`

The mandatory Rust checks, managed Bright Builds checks, full `just test`,
redaction verification, parity validation, and parity-progress validation must
also pass. Promotion to `verified` requires executable `unit,golden` evidence
for the complete typed Ultra 205 construction boundary above, exact fixture
provenance at the pinned reference commit, no unchecked placeholder
expectations, and explicit preservation of every live/effectful non-claim. No
hardware, detector, credential, recovery, retry, or destructive-action gate is
entered by this software-only plan.
