# Parity work plan

- Run ID: `20260812T034058Z-NET-001`
- Parity row: `NET-001`
- Initial status: `implemented`
- Source commit: `a972f191117f1a3a392f5785805fb590476c6f33`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-net001-reconnect-lifecycle-attempt-002`

## Selection

The branch and pinned reference are clean and synchronized. The canonical
selector reports no open plan and lists `NET-001` first, so no row was skipped.
This plan directly continues
`docs/parity/work-plans/20260812T025425Z-NET-001/PLAN.md`.
The attempt-001 closure records successful live disconnect, delayed retry,
DHCP recovery, client-only return, and stability, but no accepted public
evidence because the host validator rejected the valid 28-ms separation
between sequential connected and recovered publications before final HTTP
quorum. Commit `a972f191117f1a3a392f5785805fb590476c6f33` replaces exact
timestamp equality with a bounded nonnegative delay and adds a regression using
that observed separation. A focused direct-Bun run also exposed that the real-
child fixture inherits the launcher runtime and can emit empty stdout even
though the canonical Bazel/Node test passes. The owner Wi-Fi input exists, the
attempt-002 paths are fresh, and the connected Ultra 205 makes this row
actionable.

## Scope and non-scope

Reconfirm the corrected timing validator, replace the launcher-dependent
real-child fixture with a production-shaped POSIX child, and reuse the
implemented production reconnect lifecycle without further firmware behavior
changes. Build an exact package from the clean pushed execution commit, run one
new protected detector, and conditionally run one attempt-002 capture. The
capture must independently prove the complete lifecycle and reach the same-
origin HTTP system-info and retained-log postcondition before emitting the
closed redacted `bitaxe-network-reconnect-evidence-v1` projection.

The private transaction may contain Wi-Fi values, hostnames, device/network/
USB/process identities, origins, HTTP bodies, commands, and serial bytes. None
may enter public output, committed evidence, diagnostics, or debug formatting.
The public projection may contain only provenance digests, closed reason and
ordinal fields, retry/stability durations, equality/connected/fallback/safety/
cleanup/mode booleans, and the schema-defined no-sensitive-data facts.

Authorized effects are one exact normal package flash; replacement NVS with
the exact Ultra 205 defaults, owner Wi-Fi, `mineonboot=false`, and one private
`netreconprobe` marker; repo-owned USB reset/re-enumeration; bounded receive-
only USB and same-origin HTTP; one clear-before-effect station disconnect; the
normal configuration-AP fallback and station reconnect; and at most one
ordinary exact-package recovery flash without the marker. The successful final
state retains the exact package, owner Wi-Fi, exact defaults, and disabled
mining.

This plan does not authorize router or RF changes, discovery, credential
mutation after boot, erase-flash, arbitrary raw writes, OTA, recovery upload,
power interruption, foreign-process termination, mining, ASIC initialization
or work, pool traffic, voltage, frequency, fan, thermal or power control,
self-test, direct UART, pins, pads, headers, GPIO, probes, jumpers, soldering,
or injected signals. `NET-002`, `NET-003`, repeated or long-running reconnects,
provisioning-client suppression, IPv6, other boards, and release readiness
remain non-claims.

The design continues to follow the repo-local task, USB, privacy, and evidence
guidance plus the Bright Builds architecture, code-shape, verification,
testing, and TypeScript standards. The corrected boundary admits only a
nonnegative connected-to-recovered lag of at most 1,000 ms; all other timing,
identity, safety, cleanup, and evidence requirements remain unchanged.

## Implementation

- [ ] Reconfirm the attempt-001 closure, corrected timing predicate, observed-
      separation regression, and closed projection contract; make the real-
      child stdout regression independent of its JavaScript test launcher.
- [ ] Run the complete software gate, commit and push this immutable plan, and
      then commit and push the focused regression fix before building the exact
      normal package from the clean execution commit.
- [ ] Spend exactly one new protected detector and at most one conditional
      attempt-002 using fresh private and public paths.
- [ ] Independently validate any emitted projection and promote only `NET-001`
      when every lifecycle, exact-package, HTTP, retained-log, safety, cleanup,
      mode, and redaction fact passes.

## Verification and promotion

Run the focused automation timing and real-child regressions, then in order:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`
9. `just verify-redaction`
10. `just verify-reference`
11. selector, immutable-plan, generated-contract, task-uniqueness,
    reference-cleanliness, sensitive-output, private-mode, fresh-path, and diff
    checks

After the immutable plan and focused execution commits are clean and pushed,
run exactly:

1. `bazel build //firmware/bitaxe:firmware_image`
2. `test ! -e scratch/net001-reconnect/wrapper-002 && (umask 077; mkdir -m 700 -p scratch/net001-reconnect/wrapper-002 && just detect-ultra205 > scratch/net001-reconnect/wrapper-002/detector.stdout 2>&1)`
3. Only after command 2 succeeds:
   `test ! -e scratch/net001-reconnect/attempt-002 && test ! -e docs/parity/evidence/net001-reconnect/network-reconnect-projection.json && (umask 077; just capture-network-reconnect-evidence --private-root scratch/net001-reconnect/attempt-002 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/net001-reconnect/wrapper-002/detector.stdout --projection docs/parity/evidence/net001-reconnect/network-reconnect-projection.json --capture-timeout-seconds 90 > scratch/net001-reconnect/wrapper-002/capture.stdout 2> scratch/net001-reconnect/wrapper-002/capture.stderr)`

Wrapper and attempt roots must be absent before creation, ignored, mode 0700,
and contain only mode-0600 files. Detector failure stops before writes. Any
conditional capture start consumes attempt-002. Preserve the earliest typed
failure through cleanup and at most one ordinary exact-package recovery flash;
release every owned resource and never retry this ordinal. Accepted non-success
categories are `package_invalid`, `process_failed`, `timeout`,
`hardware_blocked`, `evidence_invalid`, `reconnect_not_observed`,
`reconnect_timing_invalid`, `service_recovery_failed`, and `recovery_failed`.

Promotion requires exact source/reference/package identity, exactly one
detector-admitted board 205, clear-before-effect probe consumption, one
post-boot station disconnect, immediate configuration-AP fallback, a closed
reason category, exactly one first retry no earlier than 5,000 ms, DHCP
reacquisition in the same boot, retry ordinal reset, client-only recovery,
15,000 ms of stable connected service, final same-origin HTTP system-info and
retained-log quorum, `mineonboot=false`, disabled mining and hardware control,
complete USB/process cleanup, private modes, redaction, independent projection
validation, and all gates passing. Otherwise create a truthful closure,
withhold public evidence, and keep `NET-001` at `implemented`.
