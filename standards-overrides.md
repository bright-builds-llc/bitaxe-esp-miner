# Standards Overrides

Use this file to record deliberate deviations from the canonical coding and architecture standards.

## Active overrides

| Standard                                    | Local decision                                                                                                          | Rationale                                                                                                                                                                                                                                 | Owner                  | Review date |
| ------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------- | ----------- |
| Build orchestration: `just` invokes Bazel   | Only `diagnose-host-stalls` starts the installed Node runtime directly. Its tests remain in Bazel.                      | The recorder must start before Bazel to observe Bazel startup and Cargo workspace-status lock waits.                                                                                                                                      | Repository maintainers | 2026-09-10  |
| TS/JS UI library: MysticUI default          | The SolidJS web UI variant (`firmware/bitaxe/web/solid`) uses no component library; it reuses the `current` stylesheet. | The UI ships in a 3 MiB SPIFFS partition and over OTAWWW, with a hard gzip budget (`firmware/bitaxe/web-ui-budget.json`). It must also match `current` one to one. A component library would add size and diverge from the shared markup. | Repository maintainers | 2027-01-07  |
| TS/JS package manager: Bun for new projects | The SolidJS variant uses a pnpm lockfile through `aspect_rules_js` `npm_translate_lock`, not Bun.                       | The repository already builds JavaScript hermetically with `aspect_rules_js` and pnpm lockfiles, and Bazel needs a lockfile it can translate.                                                                                             | Repository maintainers | 2027-01-07  |

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
