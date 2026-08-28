# Prove Ultra 205 TCP payload delivery

- Run ID: `20260828T185251Z-STR-005`
- Parity row: `STR-005`
- Initial status: `implemented`
- Source commit: `478fed4d25a38d1d87cf70edf9a5c40b0f183614`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-str005-tcp-payload-205`
- Parent plan: `docs/parity/work-plans/20260828T175218Z-STR-005-DECOMPOSITION/PLAN.md`

## Selection

The clean synchronized selector returned no open plan and ordered `ASIC-009`,
`ASIC-010`, `STR-005`, `BAP-001`, then `BAP-002`.

- `ASIC-009` is not actionable: its pure BM1368 protocol core is complete, but
  the required firmware adapter and supported BM1368 hardware are unavailable.
- `ASIC-010` is not actionable for the same concrete reason for BM1397.
- `STR-005` is the first actionable row. Its decomposed active TCP-payload task
  authorizes this child plan and repository implementation before one bounded
  task-local hardware diagnostic.

The plan follows the repository's hardware-attempt, evidence/privacy,
deterministic ESP-device-session, exact-restoration, and task-archive guidance.
The Bright Builds architecture, code-shape, testing, Rust, and verification
standards require a typed pure decision core, thin effect shell, focused
Arrange/Act/Assert regressions, and the complete ordered pre-commit gates.

## Scope and non-scope

Implement one repository-owned `stratum-v2-tcp-payload` workflow that proves an
exact detector-admitted Ultra 205 connects to an admitted same-subnet local
fixture and delivers one fixed public 64-byte payload. The payload is the byte
sequence `0x00` through `0x3f`; evidence records its fixed SHA-256 identity, not
arbitrary runtime bytes.

The workflow must bind exact clean pushed source, pinned reference, canonical
package, immutable plan, task-local ordinal `diagnostic-001`, detector identity,
fixture listener, exact peer, firmware connect and complete-write boundaries,
fixture acceptance and exact receipt, byte count, payload digest, restoration,
inactive zero-work runtime, and USB/process cleanup. It must preserve the first
typed failure through restoration, cleanup, sealing, and projection.

Allowed effects are limited to ESP32-S3 detector/board-info admission, one exact
current-package flash with private Wi-Fi and generated local-fixture NVS seed,
one same-subnet TCP connection and fixed payload write, receive-only monitoring,
and the predeclared recovery-006 firmware/settings restoration. The diagnostic
must keep `mineonboot=false`, mining inactive, ASIC work disabled, and the ASIC,
fan, voltage, thermal-control, and power-control owners untouched.

Noise acts, encrypted or Stratum V2 protocol messages, channel/job/share
handling, ASIC work, nonce generation, mining, external pools, DNS/mDNS/ARP or
router discovery, other boards, direct UART/pins/pads/headers, raw NVS or
coredump access, credentials in output, fault injection, OTA, erase, arbitrary
writes, stress, and parity promotion are outside scope.

## Attempt contract

No device, network, package-installation, or credential effect is eligible
until the plan/task checkpoint and implementation/evidence-contract commit are
separately verified, committed, and pushed. On the resulting clean exact HEAD:

1. Run the full ordered software gates and `just package`.
2. Run `just detect-ultra205` and continue only when exactly one Ultra 205 is
   admitted by successful ESP32-S3 board-info.
3. Run exactly once:

   `just stratum-v2-tcp-payload start --board 205 --port <detector-port> --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --restore-bundle scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json --private-parent scratch/str005-tcp-payload/diagnostic-001 --projection docs/parity/evidence/str005-tcp-payload/tcp-payload-projection-001.json --plan docs/parity/work-plans/20260828T185251Z-STR-005/PLAN.md --diagnostic-ordinal 1 --capture-timeout-seconds 360 --redact-evidence`

The outer supervisor creates the absent mode-`0700` private parent, creates
distinct mode-`0600` child stdout/stderr siblings, proves its intended capture
child absent immediately before launch, and gives the child exclusive ownership
of capture-root creation. All protected artifacts remain secret-sanitized,
mode-`0600`, ignored, immutable after sealing, and unavailable to Git. Wi-Fi,
generated fixture address/port, USB identity/path, peer address, device origin,
settings, logs, PIDs, and commands containing operational values remain
`ProtectedOperational`; credential contents remain `NeverPersistRaw`.

The public projection may contain only the closed schema, source/reference and
opaque package/plan/evaluator digests, board and ordinal, fixed payload digest,
typed stage booleans, bounded byte counts/durations, closed failure signature,
restoration/cleanup booleans, and `redaction_status: passed`. An independent
validator whose transitive source inventory is identity-bound must accept it.

Recovery always runs after any admitted flash or diagnostic effect. It restores
the exact recovery-006 package, partition identity, settings, appearance, and
inactive state; proves `mineonboot=false`, zero hashrate/work/share activity,
fresh board admission, no unexpected serial holder, and zero owned processes;
and preserves restoration failures as secondary to the earliest diagnostic
failure. If exact restoration or cleanup cannot be proved, stop with
`stop_hardware_blocker` and make no success claim.

This ordinal has no unchanged retry. `complete` requires every accepted stage,
exact 64-byte fixture receipt and digest, exact restoration, protected modes,
independent validation, and redaction. Otherwise select exactly one closed
hardware-attempt outcome. A fresh ordinal requires a distinct authoritative
signature plus a regression-backed targeted fix or objectively proven
authorized non-invasive remediation; one recurrence after its fix selects
`stop_repeated_boundary`.

## Implementation

- [ ] Add a typed TCP-payload diagnostic admission and NVS seed that cannot
      enter Noise, protocol, campaign, ASIC, or hardware-actuation paths.
- [ ] Add a fixture mode that admits one exact peer and reads exactly the fixed
      64-byte payload with bounded partial-read and extra-byte classification.
- [ ] Add the protected outer supervisor, closed projection, independent
      validator, evaluator inventory, CLI/Just/Bazel wiring, and exact
      recovery-006 restoration.
- [ ] Add focused pure, real-child, real-loopback TCP, source-ownership,
      privacy, failure-precedence, restoration, and command-contract tests.
- [ ] Append every implementation and hardware checkpoint to `WORKLOG.md`.

## Verification and promotion

Before the plan commit and again before any later source/finalization commit,
run the mandatory ordered Rust checks, `bun scripts/bright-builds-check.ts all`,
`just test`, `just parity`, and `just parity-progress`, plus focused TCP
diagnostic tests, canonical package, `just verify-redaction`, reference
cleanliness, sensitive-value review, whitespace, and final diff review.

Accepted diagnostic evidence completes only this decomposed prerequisite. It
does not change the STR-005 checklist status, target, or evidence cell and must
not synchronize parity progress. Close this plan truthfully without claiming
verification, complete and archive this child task, then move only
`task-str005-noise-auth-205` to active and require its own immutable plan.
STR-005 remains `implemented | unit,golden,workflow` until the final cumulative
share campaign and promotion task succeed.

