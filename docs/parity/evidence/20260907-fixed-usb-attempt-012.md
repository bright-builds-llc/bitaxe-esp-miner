# Fixed Serial/JTAG attempt-012 qualification and final-window failure

## Exact artifacts

- Installed firmware: `a90b436e38e8f36bdd1de4afb5fbe51933d5bda7`.
- Firmware ELF SHA-256: `c43fd0853a6fc558fb1353b58837a7bb421a46a64896a0d204abc48239aa9dee`.
- Gate browser implementation: `c8ffb63420b0c7e5e6fed072469a9bcda4cd5194`.
- Qualification correction: `96c6c218` (tooling only; runtime artifacts unchanged).
- Board/browser: Ultra 205, macOS desktop Chrome, direct Web Serial.

All thirteen exact package/browser artifacts are retained in protected storage.
The original campaign, contexts, consumed attempts and first failures are
unchanged. No credentials, signed windows, private identity bindings or owner
pool values are included here.

## No-mining qualification

Initial admitted NVS-preserving installation and all four current-image
update/reconnect cycles passed. Each cycle proved the exact stable runtime,
fresh browser possession, actual 65536-byte request and response payloads,
unchanged Device Identity/settings/authorization marks, mine-on-boot false,
acknowledged restoration and browser/CLI ownership release. These are physical
observations, not inferred from matching USB descriptors.

The fan-only production-owner operation acknowledged full fan and measured
fresh post-command 2361 RPM. It then proved qualified cooling and acknowledged
30% fan restoration. It reported no ASIC effects and no budget reservation.
Authenticated ledger reports before and after matched the original campaign:
210000 ms charged, reserved/completed masks 3, no pending reservation.

Successor creation initially failed as `successor_baseline`: the harness
incorrectly required the historical recovered attempt to remain connected.
The predecessor's immutable record correctly proved serial release. A tested
correction distinguishes released historical recovery from connected current
admission. A version-2 qualification amendment binds the published tooling
commit, unchanged runtime/browser artifacts, four cycle receipt hashes and
original campaign/limits. Its exact source allowlist excludes runtime changes.
The original failure is retained; no lease was issued before this correction.

## Final original window

Only original window 2 was issued and consumed, with its existing 30000-ms
reservation, 5000-ms renewals and 15550-ms shutdown reserve. The production
browser Start failed as `start_failed` with serial category `timeout`. No
running qualification sample, renewal confirmation or heartbeat suppression
was recorded. Closed device diagnostics showed preparation, readiness mask 63
and 240000 ms reserved, followed by a new boot reporting reset reason `panic`.
Those diagnostics locate the failure boundary; they do not establish its root
cause or actual mining duration.

Fresh browser recovery authenticated the same installed firmware and Device
Identity, with preserved settings, acknowledged restoration, inactive lease
and mine-on-boot false. Authorization marks changed after signed Start and
were not reset. A fresh authenticated ledger review confirmed the original
campaign exhausted: 240000 ms charged, reserved/completed masks 7, no pending
reservation. Reserved charge is not a measurement of actual active mining.

The repository judge rejected this window as
`running_device_evidence_missing`; no passing window result was produced.
Actual active milliseconds, ASIC work, accepted share, renewal and physical
three-second revocation/shutdown timing remain unverified. The original normal
and foreground-loss windows also remain consumed/unverified. There is no
refund, replacement campaign, unchanged mining retry or automatic parity
promotion.

## Final safe state and evidence

After recovery, a fan-only check proved fresh post-command 7681 RPM at full
fan, then acknowledged qualified cooling and restoration to 30%, with no ASIC
or budget effects. This proves the final safe cooling state; it does not prove
the failed live generation's shutdown timing across the panic.

The new Serial/JTAG firmware remains installed, mining disabled and the lease
inactive. Browser streams, locks and page were closed; the exact supervisor
was reaped; both serial nodes and the supervisor listener were verified free.
Volatile signed-window inputs were released. No reflash, factory reset,
erase-flash, credential provisioning or extra mining followed the failure.

Protected attempt-012 evidence includes the original context, artifact
snapshot, five install/update receipts, four cycle reports, cooling and budget
reviews, failed and corrected successor commands, qualification amendment,
last-window issuance/consumption, fourteen browser state samples, first-failure
record, failed judge output, final ledger/cooling/browser state and resource
review. Historical evidence remains immutable. Further mining requires a
separate explicit budget decision; offline preparation-panic diagnosis can
proceed without device effects.

## Offline triage and remaining work

The inspected preparation adapter returns typed errors for ordinary failures;
no direct panic cause was established. The preparation marker precedes the
first step and cannot distinguish fan proof, voltage, settling, ASIC enable,
detection or initialization. Per-step progress currently uses ordinary logging
and is absent from the closed browser replay. Retained Rust panic/allocation
receipts exist, but none was visible in the captured closed diagnostic views;
that absence does not identify a cause.

The next useful offline change is reset-retained, allocation-free per-step
breadcrumbs with browser replay tests, including a simulated native abort that
bypasses the Rust panic hook. Any available panic receipt must be decoded
against the exact installed ELF. Allocation failure, native assertion and
stack fault remain hypotheses; do not treat any as confirmed or use them to
justify an unchanged hardware retry.
