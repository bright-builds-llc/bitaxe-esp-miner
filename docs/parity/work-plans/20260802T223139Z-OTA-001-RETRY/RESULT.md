# OTA-001 bounded hardware retry result

- Parity row: `OTA-001`
- Final evidence conclusion: `passed`
- Implementation and package commit:
  `2541818aa23120dd85c711386efadb69a1415ad3`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Board: Ultra 205
- Attempt count: one invalid-plus-valid OTA invocation; zero retries

## Admission and commands

The committed Phase A detector-only gate selected exactly one Ultra 205 and
passed board-info. The freshly bound Phase B contract was committed before the
package, wrapper flash-monitor, HTTP, or OTA effects. `just package` produced a
manifest whose source and reference identities matched the clean Phase B commit
and pinned reference. The manifest and OTA artifact passed digest admission.

- Package manifest SHA-256:
  `ab63ff31c192cf81fea451d8b1fbe4f03068c8c004254a83d0c1cf7f5e6fdf0f`
- `esp-miner.bin` SHA-256:
  `5317896017a0039ca08e6c4437dde81ba0ab5a9b25bda643e9925376551377de`
- Phase A detector log SHA-256:
  `7262c900f315b74744e1bd870eac975f4b3d0e60079d117a216460560a80e176`
- Wrapper flash evidence SHA-256:
  `19d9ccd5d93320e082f7b3b03f53bc82a80ae0ec2750cb64443888e77e26ebec`
- Cleanup detector log SHA-256:
  `d743a4ec4d6919e6cf2cded4c4c9b0da16fb55c607dc03e83d877fea0bfa0aa9`

The local Wi-Fi credential file was passed only to the repo-owned wrapper as an
opaque input. Its contents were not read, printed, summarized, copied, or
committed. The Phase 18 helper derived exactly one origin-only device URL from
the same fresh trusted flash-monitor evidence and did not expose it.

## OTA and reboot evidence

The one authorized invocation produced:

- invalid firmware rejection: HTTP 500, curl status zero, body `Write Error`;
- valid firmware upload: HTTP 200, curl status zero, body
  `Firmware update complete, rebooting now!`;
- valid upload deadline: bounded 120 seconds;
- passive post-OTA observation: qualified OS-native reader, no reset, serial
  writes disabled, raw flash writes disabled, 480-second capture;
- exact post-reboot implementation and pinned-reference identities;
- fail-closed safe state with mining, ASIC work submission, and hardware control
  disabled;
- `ota_boot_validation=complete` and `ota_boot_validation=marked_valid`;
- successful cleanup detection on the same qualified target.

The smoke evidence and monitor transcript have SHA-256 digests
`3a59498d810b98e3fe221ff56c65b28e68a19aca68bc1dea72c720ff66d470a0`
and `d9ee8521f575115ac30d1ef0bdebc5eb76ca48aeeed06ea2b67bcfaa83be9332`.
They remain private ignored artifacts and are not copied into this result.

## Conclusion

The current admitted Rust package rejected an invalid firmware image, accepted
the valid OTA image through the implemented route, returned the upstream-visible
success response, rebooted into the exact current package, remained in its
safe fail-closed state, completed ESP-IDF OTA boot validation, marked the image
valid, and remained detectable afterward. This closes the exact observable
firmware OTA claim required by `OTA-001` with
`unit,workflow,api-compare,hardware-smoke` evidence.

## Privacy, recovery, and non-claims

All raw USB, serial, HTTP, device-origin, IP/MAC, Wi-Fi, and network evidence is
retained only under ignored `target/` roots. No secrets or credential values are
present in this result. Cleanup passed, so the conditional recovery flash was
prohibited and did not run. The single OTA attempt is consumed; no retry ran.

This result does not claim selected-partition internals, rollback, destructive
or interrupted-update recovery, OTAWWW, network longevity, mining, pool access,
active voltage/fan/power behavior, other boards, direct UART, or pin
manipulation.
