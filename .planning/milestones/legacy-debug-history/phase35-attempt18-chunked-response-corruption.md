---
status: resolved
trigger: "Phase 35 attempt 18 completes a coherent Boot A pre-capture and PATCH, then the retained-log GET fails at a malformed chunked-response boundary."
created: 2026-07-22T00:49:49Z
updated: 2026-07-22T00:54:32Z
---

## Current Focus

hypothesis: Confirmed and repaired in software - the WebSocket helper initiated a close after its terminal frame but returned before the close event, allowing the supervisor to begin the retained-log GET while the prior upgraded connection was still closing.
test: A real loopback WebSocket peer delays its close response after observing the client's close frame. The capture command must remain active until that peer-close boundary is complete and persist an exact closed marker; Phase 35 must require the marker before issuing the retained-log GET.
expecting: Every successful WebSocket capture establishes a completed connection lifecycle before the following HTTP request begins.
next_action: Run the complete software gate, commit the repair, run exact-current-HEAD preflight, and invoke fresh attempt 19 under the standing progress-gated authority.

## Symptoms

expected: Boot A post-PATCH capture obtains a coherent API snapshot, a later same-session WebSocket snapshot, and the actual retained log before constructing the epoch.
actual: Attempt 18 obtained valid API and WebSocket artifacts, then the retained-log client rejected malformed chunk framing and the supervisor preserved `boot_a_capture_failed` as the primary category.
errors: The malformed transport detail remains only in protected local evidence. No raw origin, response, setting, network identity, or device identifier is recorded here.
reproduction: The loopback peer proves the previous helper returned before the server completed the WebSocket close handshake; Phase 35 then had no ordering guard before opening its next HTTP connection.
started: Phase 35 attempt 18 at exact source `065240279c4657945ffce70d2baa501b4da7ceae`.

## Eliminated

- Boot A pre-capture coherence: the API and WebSocket projections shared one session, the WebSocket revision was later, and both retained markers were present.
- Device reset during mutation: serial evidence showed a continuous boot session, unchanged reset category, and monotonic uptime.
- PATCH, restoration, or cleanup failure: mutation completed, restoration was confirmed, and cleanup recorded no secondary failure.
- API or WebSocket schema rejection: both post-PATCH documents passed their structural boundary before the retained-log transport failed.
- Unbounded or redirected request behavior: the adapter retained its direct transport, no-proxy, no-redirect, timeout, and response-size bounds.

## Evidence

- timestamp: 2026-07-22T00:35:00Z
  checked: Attempt-18 protected artifact sizes, permissions, and typed structural projections without rendering operational values.
  found: The pre-PATCH epoch completed. The post-PATCH API and WebSocket documents were structurally valid and coherent, while only the retained-log download stopped at malformed chunk framing.
  implication: The failure is a transport lifecycle boundary after WebSocket capture, not an epoch identity or setting mismatch.
- timestamp: 2026-07-22T00:38:00Z
  checked: Passive serial boot classification across the pre- and post-PATCH capture interval.
  found: Session continuity, reset-category equality, and monotonic uptime held.
  implication: An unexpected device reboot did not produce the malformed response boundary.
- timestamp: 2026-07-22T00:43:00Z
  checked: The Phase 17 WebSocket helper's terminal-frame behavior.
  found: It called close and resolved immediately instead of awaiting the close event; Phase 35 issued the retained-log GET immediately after the helper returned.
  implication: Successful capture did not prove that the upgraded connection had completed its lifecycle before the next request.
- timestamp: 2026-07-22T00:49:49Z
  checked: A real loopback peer that delays its close response plus the focused Phase 17 and Phase 35 suites.
  found: The repaired helper waits through the delayed close response, emits the closed marker only afterward, and Phase 35 requires that marker before retained-log capture. All focused tests pass.
  implication: The missing ordering boundary is now explicit and regression guarded; the complete repository gate remains before commit and hardware.

## Resolution

root_cause: The WebSocket helper treated initiating close as equivalent to completing close. That allowed the next HTTP request to overlap a still-closing upgraded connection, and attempt 18 exposed the boundary as malformed chunk framing on the immediately following retained-log response.
fix: Resolve real WebSocket capture only after the close event, impose a bounded close-handshake timeout, record an exact closed marker, and require that marker before Phase 35 performs its next HTTP request.
verification: JavaScript and Bash syntax/style, ShellCheck, the delayed-close real-process regression, the complete affected Bazel set, canonical firmware build, reference/parity/lifecycle checks, diff check, and the mandatory ordered Rust gate all pass. Staged redaction verification and the commit remain before exact-head preflight.
files_changed: [`scripts/phase17-websocket-capture.mjs`, `scripts/phase17-websocket-close-peer.mjs`, `scripts/phase17-websocket-capture-close-test.sh`, `scripts/phase35-correlated-evidence-effects.sh`, `scripts/phase35-correlated-evidence-test.sh`, `scripts/BUILD.bazel`]
