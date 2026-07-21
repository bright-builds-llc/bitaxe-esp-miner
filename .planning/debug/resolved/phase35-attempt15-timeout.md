---
status: resolved
trigger: "Investigate Phase 35 attempt 15's request_transmission_incomplete after the attempt-14 receive-error fix; diagnose only with no protected attempt access, credentials, device, USB, external network, production edits, or evidence edits."
created: 2026-07-21T22:38:20Z
updated: 2026-07-21T23:14:45Z
---

## Current Focus

hypothesis: confirmed root cause is fixed by replacing the unusable host-tool counter with an explicit positive client send-completion boundary
test: completed local silent-peer and valid-response socket regressions, short-write and TLS-failure tests, real adapter/runfiles coverage, supervisor regression, Clippy, redaction, and lifecycle checks
expecting: software repair complete; attempt 15 remains immutable and the repeated-boundary policy still governs hardware continuation
next_action: commit the verified schema-v2 probe and resolve the explicit policy decision before any attempt 16

## Symptoms

expected: after TCP connection, the HTTP client transmits a non-empty request and receives or classifies a response within the attempt budget
actual: Phase 35 attempt 15 terminated as request_transmission_incomplete after TCP connected; no response status, headers, or body were observed; the root was sealed and non-reusable
errors: terminal_category=request_transmission_incomplete; tcp_connected=true; curl_exit_code=28; request_bytes=0; tcp_connect_millis=434; total_millis=10005; cleanup_secondary=none
reproduction: use deterministic fake-curl and local loopback-only experiments that exercise the attempt-15 request-send path; do not access protected attempt contents or hardware
started: immediately after the attempt-14 receive-error fix, at exact source 1c4979f67c0b12daee356ae5df1c1c5468ba1013

## Eliminated

- hypothesis: curl genuinely did not send the attempt-shaped GET because the connection stalled before request transmission
  evidence: a loopback peer independently captured the complete 93-byte GET while unmodified curl produced the same decisive facts: exit 28, positive TCP connect, size_request zero, and no response material
  timestamp: 2026-07-21T22:46:06Z

- hypothesis: shell parsing or Rust projection converted a positive curl request counter to zero
  evidence: raw curl write-out itself reported size_request=0 before either shell parsing or Rust classification, both on timeout and on success
  timestamp: 2026-07-21T22:46:06Z

## Evidence

- timestamp: 2026-07-21T22:38:20Z
  checked: caller-provided sealed attempt summary
  found: TCP connected in 434 ms, curl exited 28 at 10005 ms, request_bytes remained 0, and no response material or cleanup error was recorded
  implication: failure occurred after connection establishment but before the harness observed any transmitted request bytes; response parsing and cleanup are downstream or secondary

- timestamp: 2026-07-21T22:39:26Z
  checked: debug knowledge base and active lessons
  found: attempt 14 established that curl's size_request can remain zero after a bodyless GET and that exit 56 is receive-side; no resolved entry covers exit 28 after a successful connection
  implication: the prior request-counter semantic bug is the first candidate, but exit 28 must be differentiated from receive error 56 rather than assumed equivalent

- timestamp: 2026-07-21T22:40:10Z
  checked: exact-head implementation map and repository state
  found: HEAD matches 1c4979f67c0b12daee356ae5df1c1c5468ba1013; shell and Rust declare transmission complete only when request_bytes is positive or curl exit is exactly 56; exit 28 with request_bytes zero is already represented in tests as request_transmission_incomplete
  implication: attempt 15 followed the current classifier exactly; the investigation must determine whether that classification is semantically justified or is an intentionally encoded but invalid assumption

- timestamp: 2026-07-21T22:41:11Z
  checked: installed curl 8.7.1 manual
  found: size_request is documented as the total bytes sent in the HTTP request, while max-time bounds the entire transfer rather than a specific send or receive phase
  implication: exit 28 alone cannot locate the timeout phase; size_request should be independent evidence in principle, but its failure-path behavior still needs a real socket-boundary check

- timestamp: 2026-07-21T22:43:07Z
  checked: deterministic loopback silent-peer experiment with installed curl 8.7.1
  found: the peer independently captured the complete 93-byte request through its header terminator, then withheld the response; curl exited 28 after 1.010 seconds while its raw write-out reported size_request=0, time_connect positive, and time_starttransfer zero
  implication: request_bytes=0 plus exit 28 does not prove incomplete request transmission; the exact attempt-15 metric shape can be produced after complete transmission, before any response status, headers, or body

- timestamp: 2026-07-21T22:45:01Z
  checked: success counterfactual and unmodified adapter against an immediate valid loopback response
  found: installed curl still reported size_request=0 after the peer captured the complete 93-byte GET and curl returned 200 successfully; the adapter consequently rejected the otherwise valid response as http_diagnostic_invalid
  implication: size_request is unusable as a request-transmission counter on this exact host curl build, not merely unreliable for exit 28; fake fixtures with request_bytes=128 do not represent the real curl boundary

- timestamp: 2026-07-21T22:46:06Z
  checked: unmodified production adapter against a loopback peer that captured the GET and withheld every response byte for 11 seconds
  found: the peer captured all 93 request bytes, but the adapter emitted request_transmission_incomplete with tcp_connected=true, curl_exit_code=28, and request_bytes=0
  implication: the original terminal category is reproduced as a false send-boundary classification in about 10 seconds without hardware, credentials, protected evidence, or external network

- timestamp: 2026-07-21T22:47:16Z
  checked: uncached repository tests with downloads disabled
  found: `//scripts:phase35_http_boundary_read_test` and `//tools/parity:tests` both passed; fixtures set successful GET request_bytes to 128 and explicitly expect exit 28 plus zero to classify as request_transmission_incomplete, but no test independently observes peer receipt
  implication: the green suite confirms internal consistency while exposing the missing production-representative socket boundary that allowed the defect

## Resolution

root_cause: At exact source 1c4979f67c0b12daee356ae5df1c1c5468ba1013, the adapter maps curl `%{size_request}` to request_bytes and Rust treats positive request_bytes or exit 56 as the only proof that a bodyless GET was transmitted. The installed `/usr/bin/curl` 8.7.1 reports size_request=0 for this GET even after a loopback peer received all 93 bytes, on both success and exit-28 response timeout. The attempt-14 exit-56 special case therefore fixed one symptom but left exit 28 falsely classified, while fake fixtures concealed the host behavior by inventing request_bytes=128 for success.
fix: Replaced the production curl request with a repo-owned Rust client that supports plain and verified TLS origins, writes exactly one bounded GET, and records `request_send_complete_millis` only after every request byte is accepted and the transport flush succeeds. Schema v2 carries the typed transport outcome, positive completion timing, and actual written-byte count; the pure classifier never infers completion from timeout, connection, or curl counters. The shell retains protected no-clobber artifacts, direct built-tool invocation, redacted projection handling, and earliest typed failure behavior.
verification: The original unmodified adapter remains the red reproduction. The repaired real adapter now reaches `ready` against a valid local response and `response_status_missing` when a peer receives the full request but stays silent; a synthetic short writer retains zero completion and partial bytes, and a failed TLS handshake never claims a request send. Direct Rust, uncached Bazel/runfiles, correlated-supervisor, shell syntax/format/style, reference, parity, lifecycle, redaction, diff, and Clippy checks pass. No hardware, credential, device request, admission, promotion, or push occurred after the repair.
files_changed: [Cargo.toml, Cargo.lock, MODULE.bazel.lock, scripts/phase35-http-boundary-read.sh, scripts/phase35-http-boundary-read-test.sh, tools/parity/Cargo.toml, tools/parity/BUILD.bazel, tools/parity/src/main.rs, tools/parity/src/phase35_http.rs, tools/parity/src/phase35_http/tests.rs, tools/parity/src/phase35_http_probe.rs, tools/parity/src/phase35_http_probe/tests.rs]
