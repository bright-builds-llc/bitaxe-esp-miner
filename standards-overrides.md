# Standards Overrides

Use this file to record deliberate deviations from the canonical coding and architecture standards.

## Active overrides

| Standard                                  | Local decision                                                                                     | Rationale                                                                                            | Owner                  | Review date |
| ----------------------------------------- | -------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- | ---------------------- | ----------- |
| Build orchestration: `just` invokes Bazel | Only `diagnose-host-stalls` starts the installed Node runtime directly. Its tests remain in Bazel. | The recorder must start before Bazel to observe Bazel startup and Cargo workspace-status lock waits. | Repository maintainers | 2026-09-10  |

## Notes

- Prefer narrow, explicit exceptions over broad "this repo is different" statements.
- If local verification is intentionally hook-owned or leaves heavy suites to CI, record that explicitly here.
- Revisit overrides periodically instead of letting them become permanent by accident.
- If an override becomes common across many repos, move it back upstream into the canonical standards repo.

## Test-only POSIX terminal regression

The operator interruption regression may use Python's standard-library `pty`
module in `scripts/str005-v2-serial/operator-rehearsal-pty.py`. Node's standard
library cannot allocate and close a real PTY master; this avoids adding a native
Node dependency solely for that operating-system boundary. Production operator
commands and their orchestration remain repository-owned JavaScript. This
exception permits no device, credential or mining access from the test helper.

Owner: repository maintainers. Review date: 2026-09-19.
