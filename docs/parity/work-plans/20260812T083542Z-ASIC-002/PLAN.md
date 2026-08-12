# Parity work plan

- Run ID: `20260812T083542Z-ASIC-002`
- Parity row: `ASIC-002`
- Initial status: `implemented`
- Source commit: `90fdfea035302e55707d5cd5e689f0e75ad1b6b2`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-asic002-sealed-initialization-promotion`

## Selection

The canonical selector returned no open plan and listed `ASIC-002` first,
followed by `ASIC-003`, `ASIC-004`, and the remaining unfinished rows.
`ASIC-002` is actionable without skipping any earlier candidate. Its current
row is implemented and already has narrow diagnostic hardware-smoke evidence,
while the later sealed clean-HEAD accepted-share campaign proves the missing
full initialization boundary. No other row is in scope for this invocation.

Preflight found `main` clean, checked out, tracking `origin/main`, and exactly
synchronized after fetch. The read-only reference is clean at the commit
above. Repository guidance, Bright Builds architecture/code-shape/
verification/testing standards, Rust and TypeScript standards, and the active
lesson baseline require a typed functional-core projection, private boundary
parsing, conservative evidence, and no unnecessary hardware effect.

## Scope and non-scope

This run will convert the already sealed protected evidence from
`scratch/ultra205-accepted-pool-share/attempt-007` into one redacted public
`bitaxe-asic-initialization-evidence-v1` projection. The source attempt was
produced by exact clean commit
`3e0966a140edbff1a14d2a48ca63d140649762c0`; its archived task records one
detector-admitted Ultra 205, exact-package admission, all nine preparation
boundaries complete, trusted identity, live BM1366 work, accepted response,
fresh safety, confirmed safe stop, and USB cleanup.

The projection must independently validate the sealed campaign result,
private diagnostics and observations digests, protected modes, exact accepted
terminal state, 18 valid preparation events, zero invalid preparation events,
final `retain_production_uart/completed`, and current-source compatibility. The
compatibility check must prove that the pure initialization plan, mining-ready
command plan, actuation coordinator/adapter, reset adapter, UART adapter, and
ASIC status adapter are byte-identical between the attempt source and current
source commits. Current tests must additionally prove the fixed nine-step
order and fail-closed rollback behavior.

No new detector, flash, reset, serial session, network request, credential
read, mining lease, fan/voltage/power/ASIC actuation, or other device effect is
permitted. The protected source files are read-only inputs; the command must
not modify them. Raw observations, runtime identifiers, USB/network values,
pool/Wi-Fi values, targets, difficulty, messages, nonces, credentials, secrets,
or secret-derived hashes must not enter the public projection, logs, task
record, result, checklist, or Git history.

This row does not claim frequency-transition parity, voltage/fan/power parity,
thermal response, work-send or result-parsing parity, pool/Stratum parity,
default-profile safety, long-duration stability, other ASICs or boards,
updates, recovery, profitability, or release readiness.

## Implementation

- [ ] Add a Rust-owned closed evidence contract and validator for the exact
      ASIC-initialization projection.
- [ ] Add a thin host projection command that verifies campaign seals,
      protected modes, accepted campaign/preparation facts, source-task
      lineage, and byte-identical current initialization paths before writing
      only redaction-safe fields.
- [ ] Add behavior-focused regressions for every rejected campaign,
      preparation, seal, mode, compatibility, malformed-input, and sensitive-
      output boundary, including a real child-process/file seam.
- [ ] Produce the checklist's required public evidence from the sealed
      `attempt-007` inputs without interacting with hardware.

## Verification and promotion

Focused verification will run the new Rust contract tests, host projection
tests, real-child integration, current mining-actuation/adapter tests, campaign
serial tests, and direct validation of
`docs/parity/evidence/asic002-initialization/asic-initialization-projection.json`.
The mandatory ordered repository gate is:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Also require the real ESP32-S3 firmware image, generated automation contracts,
`just verify-redaction`, `just verify-reference`, exact reference cleanliness,
task uniqueness, immutable-plan digest, public-sensitive-value scan, campaign
seal/digest/mode validation, initialization-path compatibility, and
`git diff --check`.

Promote only `ASIC-002` from `implemented` to `verified` with
`unit,workflow,hardware-smoke,hardware-regression` if the closed projection
proves all nine ordered preparation steps necessarily completed on one exact-
package Ultra 205 campaign, production UART was retained, live initialized
BM1366 work occurred, exact identity and safety were trusted, and safe stop,
lease cleanup, USB cleanup, artifact modes, seals, current-source
compatibility, independent validation, and redaction all pass. Any missing,
malformed, inconsistent, unsealed, mode-unsafe, source-drifted, or sensitive
input withholds evidence and leaves the row `implemented`; there is no
hardware retry or effect path in this plan.
