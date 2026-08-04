# Parity work plan

- Run ID: `20260804T122408Z-V12-OPERATOR-SNAPSHOT-205`
- Parity row: `V12-OPERATOR-SNAPSHOT-205`
- Initial status: `implemented`
- Source commit: `dd6437e7d5d639507443eaf0d291265578669d08`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-v12-operator-snapshot-typed-capture`

## Selection

`bazel run //tools/parity:report -- next-item --format json` reported no open
plan. The clean `main` branch matched `origin/main`. Candidate order was
evaluated against the current checklist, active terminal records, standing USB
authorization, and the requirement that one invocation advance exactly one
row.

- `CFG-001` retains a safety-critical 485 MHz/1200 mV actuation claim. The
  prior soak evidence is private, non-promotable, and closed at a repeated
  boundary.
- `CFG-005` and `NET-001` retain broad runtime PATCH and reconnect/fallback/
  IPv6 behavior that the hostname-only evidence does not prove.
- `ASIC-002` through `ASIC-005`, `ASIC-007`, `STR-001`, `STR-006`, and
  `STR-007` retain full initialization, loaded transport, live result, socket,
  coordinator, watchdog, or promotable soak gaps. Consumed terminal mining
  attempts cannot be repurposed as row evidence.
- `API-002`, `API-003`, and `API-009` retain full live field, broad PATCH, and
  claim-specific command-effect gaps.
- `PWR-001` through `PWR-003`, `PWR-005`, `PWR-006`, `THR-001` through
  `THR-003`, and `SELF-001` retain safety-control, sensor, reference-math,
  fault, or hardware-mode gaps without a promotable row-specific contract.
- `LOG-001` still lacks upstream-compatible no-init header validation and
  soft-reboot retention in addition to complete live lifecycle evidence.
- `REL-001` through `REL-003` retain selected-partition, rollback, recovery,
  OTAWWW, or release-readiness gaps.
- `SAFE-10`, `SAFE-11`, `CFG-07`, `ASIC-09` through `ASIC-12`, `STR-08`,
  `STR-09`, `SAFE-12`, and `SAFE-13` require promotable live mining and safety
  evidence that the terminal soak records explicitly withhold.

`V12-OPERATOR-SNAPSHOT-205` is the first actionable row. Phase 36 named the
exact correction, `snapshot_substance_insufficient`, and admitted no snapshot
join. Current firmware already publishes boot-local snapshot identities in
HTTP, WebSocket, and retained logs, while the now-verified hostname work
provides a typed same-device single-restart transaction. A private-first
two-epoch capture can therefore close the narrow row without mining or hardware
control.

The plan follows repository task/archive, deterministic device-session,
hardware-attempt, evidence-privacy, and standing-authorization guidance plus
the loaded Bright Builds architecture, code-shape, verification, testing,
Rust, and TypeScript standards. The active lesson budget was handled using the
current audit baseline; the disclosed unrelated omissions remain outside this
row.

## Scope and non-scope

Implement one typed capture workflow that freezes an exact package, admits one
Ultra 205, captures an HTTP system-info snapshot, a later same-boot live
WebSocket snapshot, and their exact retained-log marker, performs one normal
software restart through `device-session reboot-live`, and repeats the same
join in boot epoch `N+1`. Require exact source/reference/application identity,
same physical USB device, distinct boot sessions, ordinal advancement by one,
monotonic boot-local snapshot revisions, matching substantive safe operator
fields, complete cleanup, and closed redaction.

Private artifacts remain mode `0600` beneath one mode-`0700` absent-before-use
attempt root. The committed projection may contain only schema names,
cryptographic identities, bounded counts, terminal categories, and booleans.
It must not contain hostnames, origins, URLs, IP or MAC values, Wi-Fi or pool
values, ports, USB identities or paths, raw serial/HTTP/WebSocket/log content,
or process identifiers.

Do not modify the pinned reference tree; resume archived Phase 35/36 scripts;
use network discovery; change settings; enable mining; actuate voltage, fan,
power, ASIC, display, or input hardware; perform OTA, erase, arbitrary writes,
fault injection, direct UART, or pin work; or promote runtime health, package
identity, settings, networking, mining, safety, or any other checklist row.

## Implementation

- [ ] Add a typed operator-snapshot evidence contract and automation command
      with strict invocation, package, detector, private-root, and projection
      admission.
- [ ] Capture and validate substantive HTTP/WebSocket/retained-log joins in
      boot epochs `N` and `N+1`, reusing the authoritative live reboot
      transaction for the single restart and same-device proof.
- [ ] Preserve earliest typed failure through bounded exact-package recovery,
      keep public recovery state boolean-only, and publish final evidence only
      after cleanup and privacy validation.
- [ ] Add behavior-focused unit and real-child-process regressions for both
      epoch joins, revision/session mismatches, non-ready reboot categories,
      malformed or missing projections, recovery precedence, and sensitive
      output denial.
- [ ] Record exact commands and evidence in `WORKLOG.md`; create `RESULT.md`
      only if the hardware capture satisfies every promotion condition.

## Verification and promotion

Focused checks will cover the automation command, typed contract, existing
operator-snapshot validator, device-session integration, real process boundary,
and redaction verifier. Before implementation commit, run:

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bun scripts/bright-builds-check.ts all`
- `just test`
- `just parity`
- `just parity-progress`
- `just verify-redaction`
- `just verify-reference`
- `git diff --check`

After a clean pushed implementation, run exactly the task-recorded package,
detector, and capture commands once. Promotion from `implemented` to `verified`
requires two substantive boot-local joins, the same physical Ultra 205, one
software restart, exact build recovery, changed boot session, ordinal `N+1`,
monotonic snapshot revisions, disabled mining and hardware control, complete
cleanup, a valid closed projection, and passed redaction. Any missing fact
withholds evidence and leaves the row below verified without a retry.
