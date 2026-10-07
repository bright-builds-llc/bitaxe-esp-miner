# Host stall investigation: 2026-10-06

This follow-up to the [2026-09-10 investigation](host-stall-results-20260910.md)
captured the intermittent native-launch stall that run left open. It also found
why the user temp directory had grown to 14 GB.

## Leaked test directories

The user temp directory held about 76,800 entries (14 GB):

- About 13,000 distinct prefixes came from this repository's tests, each
  repeated 180–2,400 times, once per run. Examples are
  `api-command-effects-*`, `bitaxe-snapshot-*`, `bitaxe-corewlan-*` and
  `phase29-generation-*`.
- About 9,300 `open-bitcoin-*` entries belong to another project.
- `fseventsd` ran near 100% CPU and `syspolicyd` near 54%.

Bazel on macOS passes the user's `TMPDIR` to tests. A probe test saw
`TMPDIR=/var/folders/…/T/`; only `TEST_TMPDIR` was inside the sandbox. Most
tests create directories with `os.tmpdir()` or `std::env::temp_dir()` and never
remove them. Under `cargo test` with an isolated `TMPDIR`, only two sources
remained:

- five `phase29-generation-*` directories from one parity test helper;
- the fixed per-user `bitaxe-device-sessions-<uid>` lock directory. That one is
  production state by design and is reused, so it does not accumulate.

Fixes:

- `.bazelrc` runs every test under `//tools/test-tmpdir:test_tmpdir`. That
  wrapper gives each test a private mode-0700 `TMPDIR` under the inherited
  temp root and removes it when the test exits. `TEST_TMPDIR` itself was tried
  first, but its path is about 170 bytes. Three tests then failed, because
  `operator-daemon.mjs` and the v2 operator IPC create Unix sockets under
  `TMPDIR`, and those paths are limited to 103 bytes. The wrapper is
  test-only: `bazel run` and `bazel build` are unchanged, and the trimmed test
  configuration caused no analysis-cache discard when alternating with builds.
  `//tools/test-tmpdir:test_tmpdir_test` checks privacy, length, removal and
  exit status. A wrapper without the cleanup trap makes it fail.
- The parity generation test helper returns a workspace guard that removes its
  directory on drop.

Existing entries were left in place. The owner can reclaim the space with the
command in [the guide](host-stall-diagnostics.md#reclaim-leaked-test-directories).

## First-exec hold

The recorder ran a freshly written one-line script:

| Event                              | Offset from recorder start |
| ---------------------------------- | -------------------------- |
| Spawned                            | 38 ms                      |
| Quiet capture 2 (`sample`, `lsof`) | 33,427 ms                  |
| First stdout                       | 71,580 ms                  |
| Exit                               | 71,581 ms                  |

Capture 2 showed `/bin/sh` in state `Ss` at 0% CPU, with all 92 samples at
`_dyld_start + 0`. The process image was mapped, but its first instruction had
not run.

Ad-hoc timings agreed:

| Executable                                     | First exec | Repeat |
| ---------------------------------------------- | ---------- | ------ |
| Freshly written shell script                   | 62–93 s    | 0 s    |
| Freshly compiled C binary (ad-hoc signed)      | 5 s        | 0 s    |
| Fresh copy of Apple-signed `/bin/echo`         | 0 s        | 0 s    |
| Fresh script read by `/bin/sh <file>`, no exec | 0 s        | —      |

The unified log for `syspolicyd` was not readable from the agent session, so
the responsible service is inferred, not observed. The pattern matches macOS
holding new, non-Apple-signed executables for a security-policy assessment.

Effects observed this session:

- Tests that write and exec fake tools under short bounds failed:
  - the soak real-observer test (10 s stop grace);
  - `validator-boundary` (10 s);
  - `theme-durability` (5 s).
    The soak test now spawns node directly. The other two passed on rerun, or
    were left unchanged because widening test deadlines is out of scope.
- Slow `just` commands spent their 60–110 s in `BazelWorkspaceStatusAction`.
  That runs `cargo run -p xtask`, whose binary is relinked after changes to
  its crates. Two later status runs took 23–33 s with no relink, and six
  recorded runs did not recur, so that part stays unattributed.

The owner-side remedy, and how to confirm it, is in
[the guide](host-stall-diagnostics.md#first-exec-of-a-new-executable).

Private evidence: `scratch/host-stalls-20261006/`. Summary SHA-256
`ad7e16f92579a055e0d183fdf5de9e5eab8421c67092a532124807f00d8876cf`; capture
SHA-256 `b0ee118315d2cbc540ede7fa6fe2a6369c94052c9b75b5cca66729b41e134d52`.

## Verification

The temp directory was snapshotted around a complete gate run:

- the Bright Builds checks;
- `cargo fmt`, Clippy, build and `cargo test`;
- `bazel test //...` (305 targets);
- `just verify-redaction`.

The run added two entries. Both were `kconfgen_tmp*.old` from the ESP-IDF
configuration step of the firmware build; neither came from a test. Before the
fix, every run added about one directory per leaking test. With the new
wrapper, 304 of 305 Bazel targets passed. `//scripts:virtual_emulator_test`
failed once in a descendant-listener release check and then passed 5 of 5
reruns, so it is recorded as a non-reproducing race.

Residual risks:

- A test killed with `SIGKILL`, past Bazel's grace period, can leave its
  private `bzt.*` directory behind.
- Children spawned with an emptied environment fall back to the user temp
  directory.
- The ESP-IDF `kconfgen` leftovers are outside this repository's code.
- The first-exec hold needs the owner-side setting. Until then, tests that exec
  freshly written tools under short bounds can still fail on this host.
