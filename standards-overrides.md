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
