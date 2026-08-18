# Parity work log

## 2026-08-18T04:42:13Z | panic diagnostic implementation

- Source commit: `944cf00bc026432978aceab051b059fe1def3158`
- Actions: added a pure complete-line classifier for seven closed ESP-IDF/Rust
  panic signatures and twelve closed task families; integrated first-signature,
  task-family, and saturating-count facts into private serial diagnostics v4;
  added unknown-on-panic-reset behavior without retaining raw text.
- Verification: classifier vocabulary/task/ordinary-line tests; fragmented
  CRLF/coalesced signature ordering; privacy serialization; unknown reset
  fallback; flash/contract/automation/parity suites; evaluator source/runfiles
  admission; firmware package; Bright Builds; redaction and reference checks.
- Evidence: source inventory now binds 21 paths, including the serial reducer,
  diagnostic schema, and panic vocabulary. Synthetic real-process workspaces
  include all three new transitive sources.
- Outcome: software diagnostic is complete and focused gates pass; no hardware,
  credentials, protected attempt data, or public projection were accessed.
- Blocker or next safe action: run the mandatory full gate, commit the exact
  implementation as `SOURCE_COMMIT`, then close this software-only plan without
  a checklist transition or attempt-019 authorization.
