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
