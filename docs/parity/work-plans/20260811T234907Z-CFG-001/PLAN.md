# Parity work plan

- Run ID: `20260811T234907Z-CFG-001`
- Parity row: `CFG-001`
- Initial status: `implemented`
- Source commit: `cd87164dc17db9977dbe7c3dd7707723f3582fa7`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-cfg001-ultra205-defaults-attempt-001`

## Selection

The branch and pinned reference are clean and synchronized, and the selector
reports no open plan. `CFG-001` is the first unfinished candidate and is
actionable on the connected Ultra 205 without enabling mining or hardware
control. The existing unit and golden evidence proves the pure default model,
while inspection found the missing live seam: the ordinary owner-Wi-Fi NVS
seed contains only Wi-Fi values, so the exact `config-205.cvs` profile is not
present as one safety-paused runtime configuration.

No row was skipped. This invocation is limited to `CFG-001`.

## Scope and non-scope

Seed the ordinary owner-Wi-Fi image with the exact Ultra 205 configured
defaults plus the deliberate safety override `mineonboot=false`. Compare the
loaded live NVS semantics to the pinned default model inside firmware and
retain only a closed, redacted match marker. Add a typed host workflow that
reuses the exact-package system-info capture, requires the retained marker,
checks the API-visible subset against the same defaults, and publishes only a
closed `bitaxe-ultra205-defaults-evidence-v1` projection.

The private transaction may compare pool endpoints, ports, users, passwords,
certificates, hostnames, and device/network identifiers, but none may enter
public output, diagnostics, committed evidence, or debug formatting. Public
evidence records only provenance digests, field counts, equality booleans,
safe-state facts, cleanup, modes, and redaction.

After a clean pushed implementation, run one fresh detector and at most one
conditional attempt-001. Allowed effects are one exact normal package flash,
replacement NVS containing the exact Ultra 205 defaults, owner-supplied Wi-Fi,
and `mineonboot=false`, plus bounded receive-only USB, same-origin HTTP, and
WebSocket observation. The installed exact package and safety-paused NVS are
the intended final state, so cleanup requires process/resource release and no
recovery flash on success. A workflow failure may use at most one exact-package
recovery flash through the reused system-info transaction; recovery never
creates success.

This plan does not authorize mining, ASIC initialization or work submission,
pool connections, frequency or voltage actuation, fan commands, thermal or
power control, self-test execution, OTA, erase-flash, arbitrary raw writes,
network discovery, foreign-process termination, power interruption, direct
UART, pins, pads, headers, GPIO, probes, jumpers, soldering, or injected
signals. It proves configured values only; it makes no actuation, safety-
telemetry, other-board, or release-readiness claim.

## Implementation

- [ ] Generate one private Ultra 205 NVS seed from the typed defaults, owner
      Wi-Fi, and the explicit disabled-mining override.
- [ ] Add a pure loaded-default attestation and emit its closed redacted marker
      after the startup settings snapshot is available.
- [ ] Add the typed defaults evidence contract, validator, automation command,
      and `just` surface by reusing the existing system-info capture boundary.
- [ ] Add behavior-focused seed, attestation, retained-marker, real-process,
      failure, recovery, no-public-evidence, and privacy regressions.
- [ ] Run the complete software gate, push the exact implementation, and build
      the normal package from that clean commit.
- [ ] Spend exactly one fresh detector and at most one conditional attempt-001.
- [ ] Validate and promote only `CFG-001` when every configured-default,
      exact-package, safe-state, cleanup, mode, and redaction fact passes.

## Verification and promotion

Run focused config, flash, firmware-host, automation, and contract targets,
then in order:

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
11. selector, immutable-plan, task-uniqueness, reference-cleanliness,
    sensitive-output, private-mode, no-public-output, and diff checks

After the exact implementation commit is clean and pushed, run exactly:

1. `bazel build //firmware/bitaxe:firmware_image`
2. `test ! -e scratch/cfg001-ultra205-defaults/wrapper-001 && (umask 077; mkdir -m 700 -p scratch/cfg001-ultra205-defaults/wrapper-001 && just detect-ultra205 > scratch/cfg001-ultra205-defaults/wrapper-001/detector.stdout 2>&1)`
3. Only after command 2 succeeds:
   `test ! -e scratch/cfg001-ultra205-defaults/attempt-001 && test ! -e docs/parity/evidence/cfg001-ultra205-defaults/ultra205-defaults-projection.json && (umask 077; just capture-ultra205-defaults-evidence --private-root scratch/cfg001-ultra205-defaults/attempt-001 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/cfg001-ultra205-defaults/wrapper-001/detector.stdout --projection docs/parity/evidence/cfg001-ultra205-defaults/ultra205-defaults-projection.json --capture-timeout-seconds 600 > scratch/cfg001-ultra205-defaults/wrapper-001/capture.stdout 2> scratch/cfg001-ultra205-defaults/wrapper-001/capture.stderr)`

Fresh private and public paths must be absent. Wrapper/attempt roots are
ignored mode-`0700` directories with mode-`0600` private files. Detector
failure stops before writes; any conditional capture start consumes
attempt-001. Preserve the earliest typed failure through cleanup and optional
exact-package recovery. Accepted non-success categories are
`process_failed`, `timeout`, `hardware_blocked`, and `evidence_invalid`.
Release every resource and never retry this ordinal.

Promotion requires exact source/reference/package identity, one detector-
admitted board 205, all 27 pinned configured-default fields matching inside
the firmware attestation, every API-visible default field matching in both
HTTP and WebSocket snapshots, exact retained attestation continuity, the
deliberate `mineonboot=false` safety override, disabled mining and hardware
control, complete cleanup, private modes, redaction, independent validation,
and all gates passing. Otherwise create a truthful closure, withhold public
evidence, and keep `CFG-001` implemented.
