# Fixed Serial/JTAG qualification — attempt 009

Board: Ultra 205. Host: macOS and desktop Chrome with direct Web Serial.

Firmware source: `eb67af455bb2cb60ca4a43c613bd8f01b49c380c`.
Application ELF SHA-256: `ec1d924de8604b1a696d1fbc539487baeca46690be2fab8bbe7a63bab0c9b365`.
Gate source: `368adbd8f4c25fd6729e443314a02d7231a685d9`.

## Observations

The clean installation and four subsequent state-preserving flashes returned
to exact, healthy, stable application execution. Each CLI process exited and
serial ownership was checked before browser reuse. No factory reset or NVS
provisioning occurred.

Cycle receipts 1–3 were recorded after fresh possession, preservation and actual
maximum exchanges. Cycle 3 retained an earlier failed probe, a synthetic
pending-status overlap reproduction, and a later successful fresh connection.
Cycle 4's probe failed despite separately observed displayed readiness. There
is no cycle-4 receipt. Readiness sequencing alone was therefore insufficient
to explain the failures; no earlier failed check is relabeled as successful.

The browser originally exposed `probe_failed` / `operation_failed` while
releasing ownership. A source-hash-matched temporary debugger observation
recorded only fixed categories, booleans and bounded counts, without deliberate
pauses or payload capture. In one fresh session, three immediate maximum probes
produced pass/pass/fail. The failed exchange reported:

| Observation                                | Bytes |
| ------------------------------------------ | ----: |
| Expected request padding                   | 65376 |
| Firmware-reported received request padding | 63584 |
| Missing request padding                    |  1792 |
| Expected and received response padding     | 65396 |

The response padding was entirely the expected fixed character. The browser's
request-count comparison rejected the exchange as `probe_mismatch`. This proves
loss on the request path; it does not identify the exact physical loss point or
establish uninstrumented timing.

An earlier, separate diagnostic episode reported `admission_required` after a
successful probe. Firmware's possession context has a 60000-ms freshness limit;
heartbeats do not extend it. That episode does not explain the rapid mismatch
above. Its observations remain separate.

## Supported mechanism and follow-up

The exact ESP-IDF 5.5.4 driver drains hardware packets, attempts enqueue into a
4096-byte ring, and ignores enqueue failure. The observed deficit equals
twenty-eight 64-byte packets. This supports receive overflow as a mechanism;
no hardware overflow counter was measured. Native browser write completion
does not acknowledge firmware consumption.

The receive path also allocated 66560-byte capacity for each small record and
wiped that capacity before returning to reads. A deterministic production-
channel model reproduced shortened valid JSON despite successful native writes.
ADR-0023 / Gate ADR 0096 require bounded receive credit, exact payload integrity
before dispatch, and removal of unnecessary full-capacity wipes.

## Cleanup and non-claims

All temporary breakpoints and observation arrays were removed. Fresh possession
and acknowledged restoration confirmed Mining Baseline, inactive lease,
mine-on-boot false and matching identity/settings/authorization preservation.
Browser ownership was released; the exact supervisor exited; neither serial
node nor its listener had a remaining holder. The tested firmware remains
installed safely.

No mining window was issued in attempt 009. No mining, accepted share or
three-second mining-stop result is claimed. The original campaign remains;
no reservation is refunded and no replacement campaign is minted. Thirteen
original runtime artifacts and all attempt evidence were preserved and verified
before subsequent source changes. These serial-0.1 observations do not qualify
the successor serial-0.2 runtime or promote unrelated parity criteria.
