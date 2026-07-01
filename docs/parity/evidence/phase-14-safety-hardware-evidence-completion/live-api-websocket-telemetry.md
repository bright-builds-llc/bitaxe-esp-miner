# Phase 14 Live API WebSocket Telemetry Evidence

## Scope

This component pack covers live API and WebSocket safety telemetry for checklist
rows `API-006`, `STAT-002`, `PWR-006`, `THR-001`, and `THR-002`. It records the
detector, package, and allow-manifest gate for an explicit-device telemetry
probe. It does not verify live API values, WebSocket frames, telemetry cadence,
statistics producer samples, power sensor readings, thermal sensor readings, or
fan RPM readings because no explicit `DEVICE_URL` was supplied.

## Metadata

| Field | Value |
| --- | --- |
| Board | `205` |
| Selected port | `/dev/cu.usbmodem1101` |
| Detector command | `just detect-ultra205` |
| Board-info command | `espflash board-info --chip esp32s3 --port /dev/cu.usbmodem1101 --non-interactive` |
| Board-info status | passed |
| Source commit | `ef580c71a178c3101385a476e6964f5af80da575` |
| Reference commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| Package manifest | `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` |
| Allow manifest | `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/live-api-websocket-telemetry/allow-live-telemetry.json` |
| Exact command | `scripts/phase14-live-telemetry.sh --manifest docs/parity/evidence/phase-14-safety-hardware-evidence-completion/live-api-websocket-telemetry/allow-live-telemetry.json --out-dir docs/parity/evidence/phase-14-safety-hardware-evidence-completion/live-api-websocket-telemetry` |
| Raw artifact | `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/live-api-websocket-telemetry/live-telemetry.log` |
| Redaction review | pending in `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/redaction-review.md` |

## Observed Status

The allow manifest passed through `tools/parity safety-allow` for surface
`live-api-websocket-telemetry` and claim tier `api-websocket-projection`.

`live-telemetry.log` records:

- `DEVICE_URL status: blocked - missing DEVICE_URL`
- `api_telemetry_status: blocked`
- `websocket_frame_status: pending - maintained WebSocket client unavailable`
- `network_scan: disabled - DEVICE_URL must be explicit`

No curl request was attempted because `DEVICE_URL` was missing. Therefore these
routes were not queried:

- `/api/system/info`
- `/api/ws`
- `/api/ws/live`

No sanitized response body, selected response headers, safety telemetry fields,
or WebSocket frames were captured in this task.

## Conclusion

`API-006` remains below verified. Route registration, missing-URL evidence, or
blocked telemetry evidence does not verify live API/WebSocket safety telemetry.

`STAT-002` remains below verified for live statistics producer samples.

`PWR-006`, `THR-001`, and `THR-002` remain below verified for live power,
thermal, and fan telemetry values.

`api_telemetry_status: blocked`

`websocket_frame_status: pending - maintained WebSocket client unavailable`

Non-claims: this evidence does not prove `/api/system/info` response content,
WebSocket upgrade behavior, `/api/ws/live` frame content, 500 ms cadence,
mining/ASIC telemetry, power freshness, thermal freshness, fan RPM, or runtime
safety-control state beyond the absence of an explicit reachable device URL.

## Redaction

The wrapper includes redaction for Wi-Fi credentials, pool credentials, private
endpoints, NVS-style values, API tokens, IP addresses, and MAC addresses. This
blocked run did not capture API bodies, headers, or WebSocket frames, so no
secret-bearing response artifact was committed. The final Phase 14 redaction
ledger must still close this artifact row before evidence is cited as reviewed.
