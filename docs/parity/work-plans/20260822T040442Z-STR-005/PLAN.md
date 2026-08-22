# Parity work plan

- Run ID: `20260822T040442Z-STR-005`
- Parity row: `STR-005`
- Initial status: `deferred`
- Source commit: `d2e0835ab07a1b32521a6bfe5ce4576acda8c974`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-str005-stratum-v2`
- Activation authority: explicit user request on 2026-08-22

## Selection

The required selector command,
`bazel run //tools/parity:report -- next-item --format json`, stopped before
ranking because three verified `SELF-001` retry plans predate mandatory
`Initial status` metadata. Those immutable historical plans remain unchanged.
The implementation will make the selector ignore metadata-less open plans only
after their checklist row is already verified and add regression coverage.

`STR-005` is normally excluded from automatic selection because its checklist
status is `deferred`. The user explicitly selected this stable row and
authorized implementation, network, pool, and Ultra 205 testing. The parity
tool will therefore gain one audited activation path: an immutable open plan
whose recorded initial status is `deferred` may transition only to
`in-progress`; deferred rows remain excluded from automatic ranking and may not
jump directly to `implemented` or `verified`.

The higher automatic candidates are not substituted for this explicit request:
`BAP-001` and `BAP-002` require a live BAP accessory and direct external UART,
while `ASIC-009` and `ASIC-010` require unsupported board families. This plan
does not modify those rows.

## Scope and non-scope

Implement the pinned ESP-Miner Stratum V2 miner subset:

- six-byte SV2 frame parsing and encoding with bounded payloads;
- the official SV2 Noise NX handshake and encrypted header/payload transport;
- `SetupConnection`, success, and error handling;
- standard and extended mining-channel open, success, error, target, new-job,
  new-prev-hash, share-submit, share-success, and share-error behavior;
- bounded channel/job/work state, target conversion, share correlation,
  reconnect, timeout, fallback, and terminal failure policy;
- one firmware owner selected by typed protocol configuration, with exclusive
  pool transport and ASIC work ownership, watchdog progression, secret-free
  health diagnostics, and complete safe stop;
- a deterministic host-owned SV2 Noise pool fixture and one task-gated Ultra
  205 campaign proving the real encrypted channel/job/work/share lifecycle.

Use the official Stratum Reference Implementation `noise_sv2` crate with its
bounded/no-std-compatible feature surface. Keep project-owned message and state
logic limited to the pinned reference behavior. Do not copy C expression from
the GPL reference into MIT Rust files.

Non-scope includes newer specification features absent from the pinned
reference (custom-job negotiation, group-channel management beyond the
reference's standard/extended behavior, template distribution, reconnect
message extensions, channel update/close extensions), Stratum V1 changes,
TLS, profitability, payout correctness, unbounded mining, arbitrary pools,
other boards or ASIC families, OTA/recovery, injected faults, direct external
UART, pins, pads, probes, jumpers, soldering, or electrical measurement.

## Implementation

- [ ] Repair selector compatibility and admit only audited
      `deferred -> in-progress` activation.
- [ ] Add bounded SV2 frame/message/channel/job/share pure logic and
      pinned-reference golden fixtures with provenance.
- [ ] Integrate official `noise_sv2` initiator transport behind a narrow
      project adapter; prove handshake, encrypted framing, tamper rejection,
      nonce exhaustion, truncation, and redaction behavior.
- [ ] Add a sole firmware Stratum V2 session owner and protocol-coordinator
      primary/fallback policy without duplicating ASIC or safety ownership.
- [ ] Add the host-owned deterministic Noise pool fixture, private campaign
      admission, exact settings restoration, safe-stop, evidence projection,
      and independent validator.
- [ ] Build/package the ESP32-S3 firmware and produce the checklist evidence.

## Verification and promotion

Software verification, in order:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `bazel test //...`
7. `just build`
8. `just package`
9. `just parity`
10. `just parity-progress`
11. `just verify-redaction`
12. `just verify-reference`
13. selector, transition-policy, reference-cleanliness, sensitive-value,
    generated-artifact, file-size, and final diff reviews.

`implemented` requires all pinned-reference protocol and lifecycle behavior,
current-source provenance fixtures, the production firmware owner, canonical
package, deterministic host/firmware tests, a redacted evidence summary, and
every software gate above. No hardware or live-network inference may be used to
replace a missing software boundary.

### Ultra 205 Stratum V2 campaign contract

Effects are ineligible until the repo-owned command, private schemas,
independent validator, recovery path, focused tests, complete software gates,
clean exact package, and pushed source commit all exist.

The only permitted effect commands are:

1. `just detect-ultra205`
2. `just package`
3. `just stratum-v2-campaign --board 205 --port <detector-port> --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --private-root scratch/str005-stratum-v2/attempt-001 --projection docs/parity/evidence/str005-stratum-v2/stratum-v2-projection.json --duration-seconds 180 --redact-evidence`

Objective: admit exactly one detected Ultra 205 and the exact canonical
package; start one host-owned local-network SV2 Noise pool fixture with an
ephemeral authority key and independently verified easy target; temporarily
install the exact package and lease-bound SV2 configuration; prove one genuine
Noise handshake, setup success, channel success, target plus job/prev-hash,
fresh watchdog and safety truth, initialized BM1366 work, a locally target-
qualified nonce, one correlated encrypted share submission, one fixture-
validated success response, and complete terminal safe stop and restoration.

Private artifacts live only below a fresh ignored mode-0700 root with mode-0600
files. Wi-Fi contents, ephemeral private keys, authority keys, pool endpoint,
interface/address data, usernames, workers, passwords, device URL, IP, MAC,
SSID, NVS secret values, raw frames, and nonces are never printed, summarized,
committed, or copied into public evidence. The projection may contain only
closed categories, booleans, counts, bounded durations, digests, package/source/
reference provenance, and redaction status. The campaign must classify private
evidence before producing the redacted projection.

Preconditions: a fresh clean pushed implementation commit; every named software
gate passes; the exact package and manifest are frozen; `wifi-credentials.json`
exists and remains unread by the operator transcript; the private root and
projection do not exist; the host can bind one explicitly selected local
interface without scanning; and `just detect-ultra205` admits exactly one board
205. PSRAM, fresh safety telemetry, watchdog, fan tachometer, input voltage,
power, and supported BM1366 identity must pass before mining effects.

Allowed effects: create the private fixture and ephemeral authority; local-LAN
TCP traffic between the fixture and device; exact package flash; repo-owned USB
reset/re-enumeration; temporary private NVS Wi-Fi/SV2 settings and a consume-
before-use one-shot campaign lease; persistence of `mineonboot=false`; fan 100%;
DS4432U 1100 mV; ASIC enable/reset; conservative BM1366 initialization at
400 MHz; bounded work/result traffic; one qualified encrypted share submission
and success response; read-only runtime observation; and device-local safe stop.

Safety and stop limits: maximum 180 active seconds; fresh 4.5-5.5 V input,
power at most 15 W, ASIC temperature below 70 C, and fresh nonzero fan RPM after
the 100% command. Any identity, provenance, lease, fixture, Noise, transport,
message, target, correlation, safety, watchdog, telemetry, actuation, evidence,
or restoration fault blocks work, preserves the earliest typed category, and
begins safe stop. Success also begins safe stop immediately after the single
accepted response.

Recovery must invalidate work and shares, close only owned transports, hold
ASIC reset low, frequency-down when reachable, disable core voltage and ASIC
enable, retain 100% fan until a fresh temperature is at or below 45 C and then
set 30%, clear the one-shot lease, restore every captured non-secret setting
exactly with `mineonboot=false`, restore the exact prior package when different,
terminate the owned fixture, and release USB/process ownership. Every
independent safe-stop step is attempted even after another fails. Public
evidence is withheld unless cleanup, restoration, validation, and redaction
all pass.

Prohibited effects: third-party or external-pool connection, DNS/network/port
scanning, more than one pool fixture or campaign, mining beyond the lease,
Stratum V1 fallback during the proof, TLS, arbitrary profiles, overclocking,
automatic fan mode, erase-flash, raw writes outside the packaged NVS path, OTA,
recovery upload, foreign-process termination, fault injection, non-205
hardware, direct external UART, pins/pads/headers/GPIO, probes, jumpers,
soldering, injected signals, or raw secret output.

Attempt-001 is consumed when the campaign command starts. There is no unchanged
retry. One later ordinal requires a targeted regression-backed fix or objective
environment change that alters the failed boundary; recurrence after that
change stops the campaign. Missing Wi-Fi, unreachable host interface, detector
failure, unsafe telemetry, or absence of effect eligibility leaves `STR-005`
at `implemented` without weakening the contract.

`verified` requires the complete software result plus one accepted attempt-001
hardware projection with exact source/reference/package identity, advancing
watchdog and telemetry, full encrypted lifecycle, genuine ASIC work/result and
accepted share, terminal safe stop, exact restoration, cleanup, independent
validation, and redaction. On that evidence, transition only `STR-005` to
`verified` with `unit,golden,workflow,hardware-regression`, synchronize parity
progress, create `RESULT.md`, archive the completed task, final-verify, commit,
and push. External production-pool interoperability remains a non-claim.
