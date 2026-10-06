# Parity work result

- Parity row: `OTA-002`
- Final status: `verified`
- Implementation commit: `96a7935e19a70763ba24e18659fe7f55df4e92a0`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware ordinal: `002`, authorized by the `task-parity-ota002-www-hardware-verification`
  amendment after attempt `001` stopped at the run baseline before any OTAWWW request

## Evidence and verification

One detector-admitted Ultra 205 (BM1366), package `96a7935e19a7-dev`
(ESP-IDF v5.5.4), ran the six commands of the task contract with
`W=scratch/ota002-otawww/wrapper-002` and `A=scratch/ota002-otawww/attempt-002`:
`just detect-ultra205`; `just capture-otawww-evidence --phase install`;
`just otawww-endpoint preflight|serve|finish` with one Connect in the dedicated
Gate tab (Gate `f3c7f5a`, endpoint-handoff mode); and `just
capture-otawww-evidence --phase run --capture-timeout-seconds 420`.

Committed projection:
[`docs/parity/evidence/ota002-otawww/otawww-projection.json`](../../evidence/ota002-otawww/otawww-projection.json)
(`bitaxe-otawww-evidence-v1`), byte-identical to the private
`final-evidence.private.json`. `validate_otawww_evidence` exits 0 and `just
verify-redaction` passes with the projection in scope.

An independent review re-derived every projection claim from the private
root, which holds only mode-0600 files under mode-0700 directories:

- The probe upload returned 200 `WWW update complete` with one retained
  `Starting...`, `Working (0..100%)`, `Finished...` sequence. After a proven
  restart, `axeOSVersion` equalled the probe label and the served `version.txt`
  and `index.html` digests equalled the probe image's.
- The interrupted upload (reset before FIN after the request header and a
  prefix) produced `Starting...`, `Working (0%)`, `Working (1%)` and
  `www_update_status=Protocol Error` in the same boot session. Because the
  firmware erases all 3 MiB before its first body read, `Working (0%)` proves
  the erase completed. After the next proven restart `axeOSVersion` was
  `Unavailable` and `/version.txt` and `/recovery` both served the embedded
  recovery page.
- The recovery upload returned 200 with one complete status sequence. After a
  proven restart, `axeOSVersion`, `version.txt` and `index.html` equalled the
  package's.
- Build identity was unchanged in all six system-info snapshots. The hostname
  and the 17-field stored-settings digest were unchanged from baseline to final.
- The boot ordinal advanced by exactly one per restart. There were three
  restarts, each a `ready` `device-session reboot-live` session with one
  request, a ROM banner and a `software_cpu` reset, on the same physical device.
- There were exactly three OTAWWW handler entries and no recovery flash, Wi-Fi
  credential, factory reset or erase. The exact `safe_state` line was retained
  in every boot, every snapshot showed mining paused and 0 hashrate, and the
  Gate closed record proved an inactive Worker lease.

## Conclusion

The firmware performs the upstream whole-`www` OTAWWW update on a real Ultra
205: the partition is erased and rewritten, new assets are served after
restart, an interrupted-update leaves the UI unavailable while `/recovery`
remains, and a following complete upload restores the exact package assets,
all with firmware and stored settings unchanged. With the earlier `unit`
evidence this meets the row's `hardware-regression` and `interrupted-update`
requirements.

## Non-claims and residual risks

- One board, one run, on a local private LAN; the origin came from the Gate
  endpoint handoff.
- One interruption shape: a TCP reset mid-body after the erase. An
  interruption during the erase, power loss, a Wi-Fi drop, an HTTP timeout and
  a wrong but correctly sized body are not covered.
- Owner-approved divergence: only a body exactly the partition size is
  accepted; anything else is refused before the erase. Upstream writes short
  bodies at the end of the erased partition.
- There is no staging copy. The SPIFFS partition stays mounted while it is
  rewritten, as upstream; what the same boot serves between erase and restart
  was not observed, and `axeOSVersion` only refreshes after a restart.
- Recovery used a direct POST to `/api/system/OTAWWW`. The `/recovery` page's
  own upload form and browser caching were not exercised.
- Observations follow software restarts only. Cold boot, power-cycle
  persistence and long-term durability are not covered.
- The 4096-byte prefix is a code constant, not a measured value, and the
  session `duration_millis` field is a default, not a measured duration.
- `index.html` is identical in the probe and package images; image identity
  rests on `version.txt` and `axeOSVersion`.
- Mining, pool and hardware control stayed disabled; OTAWWW during mining is
  not covered. Stored-settings preservation covers the hostname and 17
  settings fields, not a full NVS readback.
