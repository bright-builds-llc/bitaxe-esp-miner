---
status: resolved
trigger: "Investigate and fix the Phase 35 attempt-14 request_transmission_incomplete boundary at exact source 8afbed3248fb00e02d1a09f726b48ec241b552da after HTTP adapter fix 53d8bcee."
created: 2026-07-21T21:56:54Z
updated: 2026-07-21T22:16:04Z
---

## Current Focus

hypothesis: confirmed root cause and adjacent shell guard are fixed and independently verified
test: complete — independent review confirmed Bash syntax, shfmt, ShellCheck, all 14 focused Rust tests, both focused Bazel targets, redaction, privacy, and diff checks
expecting: complete — human verification accepted the software boundary without hardware or another Phase 35 attempt
next_action: archive this resolved debug record with the verified software fix; no push

## Ranked Hypotheses

1. **H1 — receive-phase exit overrides a zero diagnostic counter:** If curl exit 56 means a failure receiving network data after a bodyless GET entered the receive phase, then the projection should mark request transmission complete even when `%{size_request}` is zero and proceed to `response_status_missing`.
2. **H2 — exit 56 may precede full GET transmission:** If exit 56 can occur before the bodyless protocol request is fully sent, then a loopback peer forced to produce exit 56 will observe zero or only partial request bytes, and using the exit code as completion evidence would be unsafe.
3. **H3 — normalization belongs in the shell adapter:** If the byte counter alone is malformed at the process boundary, normalizing only the shell-emitted `request_bytes` for exit 56 will make the regression green without Rust classifier changes.
4. **H4 — semantic inference belongs in the Rust classifier:** If the raw curl facts are valid but incomplete as independent signals, combining `curl_exit_code` and `request_bytes` only inside the pure classifier will distinguish exit 56 from exit 55 while preserving the raw metric.
5. **H5 — the existing schema cannot distinguish the boundary:** If neither documented semantics nor loopback observation proves completion, then no safe inference from current fields exists and an additional independent metric is required.

## Symptoms

expected: The original-settings GET should transmit the GET and receive a typed response, ideally `ready`; at minimum the adapter must classify curl's real semantics accurately rather than infer an incomplete send from a misleading write-out metric.
actual: The classifier reports `request_transmission_incomplete` because curl `%{size_request}` is zero even though TCP connected and curl later exited 56 while receiving network data.
errors: `non-promotion.seal category=request_transmission_incomplete`; no restoration/cleanup secondary category.
reproduction: Build a fast deterministic replay from the sealed attempt-14 HTTP inputs at `<sealed-attempt-14-root>/raw/http-original/` through the real built adapter/classifier seam. Never output or commit protected raw data.
started: Attempt 13's generic invalid fallback was repaired in `53d8bcee`; attempt 14 on 2026-07-21 exposed this precise next category.

## Eliminated

- hypothesis: H2 — curl exit 56 may occur before the bodyless protocol GET is fully transmitted.
  evidence: A local loopback peer read a complete non-empty request through the header terminator before resetting the connection; curl then returned exit 56 with `%{size_request}=0` and no response status, headers, or body.
  timestamp: 2026-07-21T22:04:02Z
- hypothesis: H3 — normalize the shell `request_bytes` value for exit 56.
  evidence: Curl documents `%{size_request}` as a raw byte count, and the strict projection exposes that same counter. Replacing zero with a fabricated positive byte count would destroy the observed fact; semantic classification belongs in the pure Rust core.
  timestamp: 2026-07-21T22:04:02Z

## Evidence

- timestamp: 2026-07-21T21:58:00Z
  checked: repository instructions, active lessons, debug disciplines, and relevant Bright Builds standards
  found: local policy requires immutable private-first evidence, strict typed boundary separation and earliest-failure precedence; this diagnosis stage forbids hardware, credential, phase-artifact, policy, lesson, and push operations
  implication: investigation and verification must stay within deterministic fake-curl or loopback seams and preserve the existing redacted metrics schema
- timestamp: 2026-07-21T21:58:29Z
  checked: current repository state and text search across non-protected source/tests
  found: working HEAD is `c10fa5c84420b9a7e0be906ddd53ae292e6d9332`; only this debug session is untracked; the shell adapter writes `%{size_request}` into `request_bytes`, the Rust classifier treats zero as `request_transmission_incomplete`, and a direct Bazel shell test already exercises the built adapter/runfiles classifier seam
  implication: the correct fast feedback loop should extend the existing `//scripts:phase35_http_boundary_read_test` fake-curl path, while the attempt-14 source commit remains historical input provenance rather than the current checkout state
- timestamp: 2026-07-21T21:59:49Z
  checked: knowledge base, prior attempt-13 debug session, complete shell adapter, complete shell regression suite, and complete Rust classifier/tests
  found: no two-keyword knowledge-base match applies; `%{size_request}` is copied unchanged into `request_bytes`, and both terminal classification and `request_transmission_complete` use only `request_bytes > 0`; the existing fake-curl seam can model exit 56 with all protected values absent
  implication: a minimal attempt-14 regression can isolate the semantic mapping without modifying schema, reading the sealed root, or touching hardware/network state
- timestamp: 2026-07-21T22:01:15Z
  checked: new attempt-14 fake-curl regression through the real adapter and built classifier seam
  found: `scripts/phase35-http-boundary-read-test.sh` fails deterministically in about 4.5 seconds because stdout lacks `category=response_status_missing`; current production instead emits the reported `request_transmission_incomplete` symptom
  implication: this is a fast, agent-runnable, red-capable replay of the exact semantic boundary; the production implementation remains unchanged
- timestamp: 2026-07-21T22:02:44Z
  checked: official curl error-code and write-out documentation
  found: curl defines error 55 as failure sending network data and error 56 as failure receiving network data; `%{size_request}` is documented as total bytes sent in the HTTP request, and write-out occurs independently of transfer success
  implication: attempt 14 contains semantically conflicting signals, so a loopback peer observation is required to determine whether exit 56 can coexist with a completed GET and zero `%{size_request}` in the real curl process
- timestamp: 2026-07-21T22:04:02Z
  checked: installed curl against an isolated local protocol loopback peer that emitted no request content or endpoint data
  found: the peer observed 93 request bytes and a complete request-header terminator before forcing a reset; curl returned process/write-out exit 56 while reporting request bytes zero and response status/header/body metrics zero
  implication: H1 is confirmed and H2 is refuted for the exact bodyless GET contract; `%{size_request}=0` on exit 56 is not evidence that the request was unsent
- timestamp: 2026-07-21T22:05:50Z
  checked: new pure Rust attempt-14 regression before production changes
  found: the focused test fails with actual `RequestTransmissionIncomplete` versus expected `ResponseStatusMissing`; projection completion is therefore never reached
  implication: H4 is confirmed at the pure decision seam; a Rust-only semantic derivation can preserve the raw counter and avoid shell-side fabrication
- timestamp: 2026-07-21T22:06:54Z
  checked: focused Rust regression after the classifier fix
  found: the exact attempt-14 classifier test passes as `ResponseStatusMissing`, sets request transmission complete, and preserves raw request bytes at zero
  implication: the pure semantic decision is corrected without fabricating the curl metric
- timestamp: 2026-07-21T22:06:54Z
  checked: rebuilt Bazel classifier plus direct full adapter test
  found: all adapter scenarios pass, including the existing exit-55 `request_transmission_incomplete` guard and new exit-56 `response_status_missing` regression
  implication: the real built-tool/runfiles seam distinguishes send failure from response receive failure and retains strict contract/redaction assertions
- timestamp: 2026-07-21T22:09:36Z
  checked: focused and adjacent verification before sealed replay
  found: Cargo formatting, shell formatting, all 14 focused Rust tests, package Clippy with warnings denied, Bazel adapter/parity tests, repository redaction verification, diff whitespace checks, and debug-log cleanup all pass
  implication: only the exact immutable sealed-input post-fix replay and final diff/privacy review remain before the human-verification checkpoint
- timestamp: 2026-07-21T22:12:11Z
  checked: exact sealed attempt-14 post-fix replay through the rebuilt production adapter/runfiles classifier
  found: replay returns `response_status_missing`, preserves raw request bytes zero, marks request transmission complete, creates no private hostname, and leaves all sealed-input digests unchanged
  implication: the original sealed boundary is fixed in software; final review must also align the shell pre-classifier consistency guard with the confirmed exit-56 semantic for partial-response receive failures
- timestamp: 2026-07-21T22:13:00Z
  checked: partial-response exit-56 fake-curl regression before shell guard alignment
  found: the built adapter test fails because stdout lacks `response_body_incomplete_or_over_limit`; the shell emits its generic invalid fallback before Rust classification
  implication: the shell guard independently encodes the same disproven zero-counter assumption and must recognize receive error 56
- timestamp: 2026-07-21T22:13:10Z
  checked: direct adapter suite after aligning the shell guard
  found: all cases pass; exit 55 remains `request_transmission_incomplete`, exit 56 without response facts becomes `response_status_missing`, and exit 56 after partial response becomes `response_body_incomplete_or_over_limit`
  implication: the deployed shell and pure Rust core now share the same closed send-versus-receive semantics and earliest category order
- timestamp: 2026-07-21T22:14:09Z
  checked: final focused verification, simplification pass, privacy scrub, cleanup, and diff review
  found: Cargo formatting, shfmt, 14 focused Rust tests, package Clippy with warnings denied, both Bazel tests, repository redaction verification, whitespace checks, debug-log cleanup, temporary-harness cleanup, and private-path scrub all pass; only the four owned source/test files and this debug record are changed
  implication: the fix is minimal, root-cause directed, schema-preserving, and ready for human verification; no hardware, detector, credential, USB/serial, device/network request, PATCH, reboot, admission, promotion, commit, or push occurred
- timestamp: 2026-07-21T22:16:04Z
  checked: independent human-verification checkpoint
  found: review confirmed Bash syntax, shfmt, ShellCheck, all 14 focused Rust tests, both focused Bazel targets, redaction, protected-path scrub, and diff checks pass
  implication: the session is resolved and may be archived with the verified production fix; no further hardware, network, credential, or push action is part of this debug record

## Resolution

root_cause: Curl exit 56 is a receive-side failure and can occur after a complete bodyless GET even while `%{size_request}` remains zero. The Rust classifier treated that raw counter as the sole proof of request transmission, so it overwrote curl's stronger phase signal with the earlier, inaccurate `request_transmission_incomplete` category.
fix: Preserve raw `request_bytes` unchanged. Add a closed receive-error semantic to the shell adapter and `CurlExitCode`, and derive bodyless-GET request completion from either positive request bytes or curl exit 56. Use the same fact for shell/Rust consistency validation, terminal-category order, and the redacted projection. Add fake-curl built-seam and pure Rust attempt-14 regressions, including partial-response coverage.
verification: The shell and Rust regressions were red before their respective fixes and green afterward. A real local loopback peer proved a complete 93-byte request can precede curl exit 56 with `%{size_request}=0`. Exact sealed attempt-14 replay now returns `response_status_missing`, preserves raw request bytes zero, marks request transmission complete, creates no hostname artifact, and leaves sealed digests unchanged. Cargo format, Bash syntax, shfmt, ShellCheck, 14 focused Rust tests, Clippy with warnings denied, Bazel adapter/parity tests, redaction verification, whitespace/privacy/debug-log/temporary-harness checks, and final diff review all pass, including independent human verification.
files_changed: [scripts/phase35-http-boundary-read.sh, scripts/phase35-http-boundary-read-test.sh, tools/parity/src/phase35_http.rs, tools/parity/src/phase35_http/tests.rs]
