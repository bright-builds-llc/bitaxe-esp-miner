# Parity work plan

- Run ID: `20260802T184857Z-STR-002`
- Parity row: `STR-002`
- Initial status: `implemented`
- Source commit: `ce4ed10df6d4c4b2cde8583a26aa3ddc002c06f7`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-str-002-message-verification`

## Selection

`bazel run //tools/parity:report -- next-item --format json` reported no open
plan. It ranked the following candidates before `STR-002`; each remains
ineligible for this invocation:

- `CFG-001`: the remaining frequency/voltage-default claim is explicitly
  safety-critical and lacks a row-specific authorized hardware contract.
- `CFG-005`: complete API PATCH and firmware NVS-adapter behavior remains
  absent, so the existing pure update model does not close the broad row.
- `NET-001`: retry reasons, fallback timing, IPv6, and long-running reconnect
  parity require network/hardware evidence without a row-specific task gate.
- `ASIC-002` through `ASIC-005`: initialization, work send, result parsing,
  and serial transport retain explicit live-hardware evidence gaps.
- `ASIC-007`: live frequency-transition behavior is safety-critical and lacks
  an authorized hardware-regression contract.
- `STR-001`: the live socket adapter and hardware-backed lifecycle evidence
  remain incomplete.

`STR-002` is the first bounded software-verifiable candidate. Its reference
surface is deterministic JSON-RPC parsing and serialization, and the checklist
separates live socket, mining, ASIC, and hardware behavior into other rows.

## Scope and non-scope

Close the pinned `stratum_api.c` message-shape contract for subscribe,
authorize, version-rolling configure, difficulty suggestion and notification,
extranonce subscribe and assignment, mining notify, share submit, result/error
responses, ping/pong, reconnect, pool messages, and version queries/responses.
Make the committed reference-derived fixture executable as golden evidence and
enforce the reference's maximum of 32 Merkle branches at the parser boundary.

Do not modify the pinned reference tree, copy GPL source expression, open a
network connection, access credentials, touch hardware, alter Stratum session
or mining orchestration, or claim live pool, ASIC, share, transport, timing,
TLS, reconnect-lifecycle, or production-mining parity.

## Implementation

- [ ] Expand the reference-derived protocol fixture to identify `STR-002` and
      cover every message family in the row with synthetic values only.
- [ ] Add fixture-driven golden tests that exercise client serialization and
      server parsing and fail when fixture ownership or method coverage drifts.
- [ ] Enforce and test the pinned reference's 32-Merkle-branch boundary without
      weakening the existing fail-closed field validation.
- [ ] Record exact verification and non-claims in `WORKLOG.md`, then create
      `RESULT.md` only if every promotion criterion passes.

## Verification and promotion

Focused acceptance commands:

- `cargo test -p bitaxe-stratum messages --all-features`
- `bazel test //crates/bitaxe-stratum:tests`
- `bazel run //scripts:verify_reference_clean`
- `jq empty crates/bitaxe-stratum/fixtures/v1/protocol-cases.json`

The mandatory Rust checks, managed Bright Builds checks, full Bazel test
surface, parity validation, and parity-progress validation must also pass.
Promotion to `verified` requires `unit,golden` evidence that every named
message family in `STR-002` is represented by the pinned fixture and exercised
through the Rust parser or serializer, the Merkle-branch cap matches the
reference constant, and all live/effectful behavior remains an explicit
non-claim. No hardware, credential, detector, recovery, retry, or evidence
privacy gate is entered because this plan is software-only and effect-free.
