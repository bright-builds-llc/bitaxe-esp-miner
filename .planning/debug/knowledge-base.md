# GSD Debug Knowledge Base

Resolved debug sessions. Used by `gsd-debugger` to surface known-pattern hypotheses at the start of new investigations.

***

## phase35-http-diagnostic-invalid-attempt12 — Positive curl duration collapsed into the zero sentinel

- **Date:** 2026-07-21
- **Error patterns:** http_diagnostic_invalid, original settings, sub-millisecond duration, curl metrics, classifier boundary
- **Root cause:** Nearest-integer millisecond conversion mapped a positive curl duration below half a millisecond to `0`, which the strict schema reserves for an absent boundary; later request facts then made the observation inconsistent before Rust classification.
- **Fix:** Preserve exact zero as absence while mapping every positive duration to at least one millisecond, including the derived TLS handshake duration, with direct and Bazel/runfiles fake-curl regressions.
- **Files changed:** scripts/phase35-http-boundary-read.sh, scripts/phase35-http-boundary-read-test.sh

***

## phase35-request-transmission-incomplete-attempt14 — Receive failure misclassified from a zero curl request counter

- **Date:** 2026-07-21
- **Error patterns:** request_transmission_incomplete, size_request zero, curl exit 56, response receive failure, request bytes zero
- **Root cause:** Curl exit 56 is a receive-side failure and can occur after a complete bodyless GET even while `%{size_request}` remains zero. The classifier treated that raw counter as the sole proof of request transmission and emitted the earlier inaccurate category.
- **Fix:** Preserve raw request bytes, but derive bodyless-GET transmission completion from positive request bytes or curl receive error 56 in both shell and Rust. Keep exit 55 as the send-failure boundary and cover missing-response plus partial-response receive failures.
- **Files changed:** scripts/phase35-http-boundary-read.sh, scripts/phase35-http-boundary-read-test.sh, tools/parity/src/phase35_http.rs, tools/parity/src/phase35_http/tests.rs

***
