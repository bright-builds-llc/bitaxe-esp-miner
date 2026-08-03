# SYS-004 live version-reporting result

- Parity row: `SYS-004`
- Final status: `verified`
- Implementation and package commit:
  `66cf184943d7f3a5aedfc99e692a9f500707de9e`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Board: Ultra 205

## Evidence and verification

The package-owned AxeOS `version.txt`, mounted-file platform adapter, extended
build provenance, and typed version-evidence workflow passed their focused and
repository-wide regressions. Attempt 008 then used a fresh package from the
clean pushed implementation commit and one detector-admitted Ultra 205. The
closed committed
[version projection](../../evidence/sys004-version-reporting/version-projection.json)
records:

- exact source commit, pinned reference commit, and package-manifest identity;
- observed safe boot with mining and hardware control disabled;
- one complete same-origin `/api/system/info` response;
- `version` equal to the manifest build label;
- installed `axeOSVersion` equal to the same package build label;
- semantic version, source/reference commits, application ELF digest, build
  channel, dirty state, and release tag equal to manifest provenance;
- `idfVersion` equal to the manifest ESP-IDF version; and
- a later WebSocket projection from the same boot, with equal-or-later positive
  snapshot revision and identical version/provenance fields.

The projection schema and Rust validator expose only closed comparison facts.
Raw detector, USB, serial, origin, network, API, WebSocket, and credential data
remain in ignored mode-`0700` private roots with mode-`0600` files. The local
Wi-Fi credential file was passed only as an opaque input to the repo-owned
workflow; its contents were not read, printed, summarized, copied, or committed.

Attempt 006 stopped before effects when a legacy detector-output delimiter was
used. Attempt 007 completed the exact-package flash and observations but
correctly withheld public evidence because its host validator required exact
revision equality across sequential HTTP and WebSocket captures. Focused
real-file and later-revision regressions corrected both host boundaries before
Attempt 008. No optional recovery flash ran because the device completed each
effectful run, remained reachable, and cleanup passed.

## Conclusion

The exact current Rust package now reports the canonical firmware build label,
installed static-asset version, ESP-IDF version, and extended provenance
consistently through both HTTP and WebSocket surfaces on a real Ultra 205. This
satisfies the observable version-reporting claim selected by `SYS-004` with
`unit,workflow,api-compare,hardware-smoke` evidence.

## Non-claims and residual risks

This result does not promote broader operator-snapshot, runtime-health,
network-longevity, mining, ASIC, voltage, fan, thermal, OTA, recovery,
non-205-board, direct-UART, or pin-manipulation claims. Their checklist rows and
task-specific evidence gates remain unchanged.
