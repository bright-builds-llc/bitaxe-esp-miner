# Fixed Serial/JTAG qualification — attempt 010

Board: Ultra 205. Host: macOS and desktop Chrome with direct Web Serial.
Transport: serial 0.2, Controller 0.4, possession 0.2.

Firmware source: `59f065ed5887e66a69bd8ba24aa2144b2696f321`.
Application ELF SHA-256: `6c02e63e9420441a4e1627bc4f3a1a5f72ef9fe44e2ffdd36eb749f2dd40e75f`.
Gate source: `95fbb9856ccfe1902b03a90d0632fd857282f295`.
Pinned Gate archive SHA-256: `831c138f4da988b806d059009dd1577f9a4b164f092d52ab8a5cd456f9362951`.

## Software and four-cycle qualification

Before hardware, Gate's complete verification passed, including 361 web/CLI
tests and browser conformance. Firmware's ordered Cargo format, lint, build
and test gates passed: 2083 tests passed and one preexisting test was ignored.
All 80 Bazel targets passed, with fixed-USB ownership, reference integrity,
redaction, standards and real ESP32-S3 packaging checks. The clean package
was built after the coordinated commits were published.

The initial installation established exact healthy application execution and
fresh browser possession. All four required serial-0.2 continuity/update
cycles passed. Each cycle included browser release, absent serial holders,
same-device ROM admission, five disjoint state-preserving write segments with
hash verification, a 30-second stable exact-identity startup observation,
CLI cleanup, fresh browser possession and an actual maximum-size exchange.
Both request and response control payloads were 65536 bytes in every receipt.
Device Identity, settings and authorization high-water comparisons matched
throughout all four cycles; mine-on-boot remained false.

An optional extra probe after acknowledged restoration was rejected locally
as `probe_admission`: restoration had cleared possession. No probe was sent
in that operation. It is neither a failed wire exchange nor an additional
passing cycle. The four successful receipts remain the qualification evidence.

## Live Start failure

Normal window 0 was issued and its memory-only signed delivery consumed once
under the original sealed campaign. Start failed as `start_failed` / `timeout`
at the browser's 30000-ms response bound. No running state or qualification
generation sample was observed. No renewal, correlated accepted share or
physical gate-closure/shutdown timing was established.

The repo-owned judge rejected the window with
`running_device_evidence_missing`; no passing window result exists. An earlier
judge invocation using a relative private path failed as
`local_operation_failed`; the absolute-path invocation reached the evidence
judgment above. Both command results are retained separately from the earlier
hardware Start failure.

Actual active mining duration and the durable reservation amount are
unverified. They must not be reported as zero or refunded. No later mining
window, replacement campaign, enlarged budget or unchanged Start retry ran.

## Recovery and final state

Fresh browser recovery first failed at `manifest_identity` with `fields`.
A bounded 30-second receive-only drain released its serial descriptor, but
the following fresh connection timed out at `hello`. Advancing startup
diagnostics showed that diagnostic output remained alive; this did not prove
control readiness or explain the Start failure.

Recovery then reflashed the same exact package using the admitted
NVS-preserving segments. The bounded observer confirmed exact healthy stable
application execution and the safe baseline. Fresh browser possession and
acknowledged restoration confirmed an inactive lease, matching Device
Identity and settings, and mine-on-boot false. The authorization high-water
comparison differed from the pre-Start baseline. A changed comparison after
signed authorization does not itself establish a reset, regression, or the
numeric campaign reservation.

Explicit Stop and Close completed with restoration confirmed, lease inactive,
running false and serial ownership released. The qualification page was
closed, the owned supervisor exited and was reaped, and neither serial node
nor the supervisor listener had a remaining holder. The exact firmware above
remains installed. This recovered baseline does not retroactively prove the
failed generation's ordered shutdown, cooling history or three-second bound.

## Evidence and remaining work

The immutable context, four validated cycle receipts, samples, first-failure
record, both judge command results, recovery evidence and final state/resource
receipts remain in the protected attempt directory. Thirteen original package,
release and browser artifacts were copied byte-for-byte and verified against
the immutable context before subsequent repository changes. Signed runtime
grants and owner credentials were not persisted in evidence. This public
projection contains only closed outcomes, counts and software provenance.

The transport cycles are complete. Overall migration and live acceptance
remain open: diagnose Start completion and recovery admission in software,
retain the consumed window and original budget, and require regression-backed
progress plus an admissible remaining campaign before further hardware work.
The browser's 30-second Start wait and the owner's potentially longer wait
are inspection findings, not a proven root cause or grounds to widen bounds.
No mining, Stratum or unrelated parity criterion is promoted.
