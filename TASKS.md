# Tasks

This is the repository's sole active work tracker. Use one stable, timestamped
task block per unit of work. Update only that block as work progresses, record
the verification performed, and finish with a concise completion review.

`TASKS.archive.md` is the append-only historical store, not an active tracker.
After a task is completed, explicitly cancelled, or superseded, append its full
final record there and remove it from this file in the same commit. Keep
blocked, deferred, terminal-blocker, future, and otherwise unresolved tasks
here. Stable task IDs must be unique across both files. Never restore or select
an archived task; create a new active task with a new ID and an archived-task
reference for follow-up work.

Task blocks under `## Future` remain incomplete and become ordinary automatic
selection candidates after higher-priority active work. Move the selected task
to `## Active` before implementation. Dependencies, environment, verification,
safety, and evidence gates still apply; no task or fresh progress-backed
attempt ordinal requires repeated user confirmation.

Historical plans, milestones, debug sessions, and task records under
`.planning/milestones/` are evidence and context only. They do not authorize
new work.

## Active

### task-parity-asic009-bm1368-core | 2026-08-20 | Implement the pure BM1368 protocol core

- [x] Add typed BM1368 init, work, result, register, framing, and error
      behavior under the pure ASIC crate.
- [x] Add pinned-reference provenance plus golden protocol fixtures and focused
      behavioral coverage.
- [x] Keep BM1368 firmware dispatch deferred, run every gate, and transition
      only `ASIC-009` to at most `implemented` with `unit,golden` evidence.

Plan: `docs/parity/work-plans/20260820T060848Z-ASIC-009/PLAN.md`

Authorization: local Rust/fixture/docs edits, tests, build/package, Git commit,
and push only. No credentials, protected attempt roots, detector,
USB/device/network runtime, flash, monitor, mining, restart, recovery, hardware
attempt, fault injection, external UART/BAP, pins, or electrical work.

Evidence and status boundary: `implemented` requires the complete pure protocol
surface, pinned fixture provenance, current tests, deferred dispatch, unchanged
Ultra 205 packaging, and all mandatory gates. `verified` additionally requires
a supported BM1368 board, firmware adapter, detector-gated hardware smoke/
regression, safe stop, and redaction; none are authorized or claimed here.

Verification: Implementation commit
`1dc17d9b9a8e12319b5ca01db297d6800bd38d46`; 12 focused BM1368 tests,
124 ASIC-crate tests with 1 existing ignored test, the deferred-dispatch
regression, Bazel ASIC tests, reference/package checks, ordered Rust gates, and
Bright Builds checks passed. Evidence:
`docs/parity/evidence/asic009-bm1368-core/summary.md`.

Completion review: The authorized pure software scope is complete and supports
`implemented` with `unit,golden` evidence only. Transition
`20260820T063300Z-ASIC-009` and progress sync accepted the exact plan and source
commit while leaving parity at 88/94 verified. This task remains active and
unarchived because firmware dispatch, a supported BM1368 board, detector-gated
hardware regression, safe-stop proof, and redaction are outstanding. Automatic
selection may treat ASIC-009 as hardware-blocked until those prerequisites
exist and continue to the next software-actionable row.

### task-parity-asic010-bm1397-core | 2026-08-20 | Implement the pure BM1397 protocol core

- [x] Add typed BM1397 init/frequency, work, result, framing, and error
      behavior under the pure ASIC crate.
- [x] Add pinned-reference provenance plus golden one/four-midstate protocol
      fixtures and focused behavioral coverage.
- [x] Keep BM1397 firmware dispatch deferred, run every gate, and transition
      only `ASIC-010` to at most `implemented` with `unit,golden` evidence.

Plan: `docs/parity/work-plans/20260820T064119Z-ASIC-010/PLAN.md`

Authorization: local Rust/fixture/docs edits, deterministic tests,
build/package, Git commit, and push only. No credentials, protected attempt
roots, detector, USB/device/network runtime, flash, monitor, mining, restart,
recovery, hardware attempt, fault injection, external UART/BAP, pins, or
electrical work.

Evidence and status boundary: `implemented` requires the complete pure BM1397
surface, pinned fixture provenance, current tests, deferred dispatch, unchanged
Ultra 205 packaging, and every mandatory gate. `verified` additionally requires
a supported BM1397 board, firmware adapter, detector-gated hardware regression,
safe stop, and redaction; none are authorized or claimed here.

Verification: Implementation commit
`3909a304213f81babf9d3fed38800bd2b515c0a5`; 16 focused BM1397 tests,
140 ASIC-crate tests with 1 existing ignored helper, the deferred-dispatch
regression, Bazel ASIC tests, reference/package checks, ordered Rust gates, and
Bright Builds checks passed. Evidence:
`docs/parity/evidence/asic010-bm1397-core/summary.md`.

Completion review: The authorized pure software scope is complete and supports
`implemented` with `unit,golden` evidence only. Transition
`20260820T070850Z-ASIC-010` and progress sync accepted the exact plan and source
commit while leaving parity at 88/94 verified. This task remains active and
unarchived because firmware dispatch, a supported BM1397 board, detector-gated
hardware regression, safe-stop proof, and redaction remain outstanding.
Automatic selection may treat ASIC-010 as hardware-blocked until those
prerequisites exist and continue to the next software-actionable row.

### task-gamma-601-bring-up | 2026-10-21 | Bring up the Bitaxe Gamma 601 (BM1370)

Owner decisions 2026-10-21:
- the staged bring-up, "Safe boot, then mining";
- the standing hardware authority "Same as the 205";
- the stock AxeOS firmware is not backed up.

Plan `delightful-foraging-quill` (owner approved). ADR-0041 records the
board identity guard and build-time board selection.

The 601, from the pinned reference (`reference/esp-miner/main/device_config.h`):
- one BM1370 (chip id 0x1370, default 525 MHz / 1150 mV);
- a TPS546 PMBus power controller at I2C 0x24, in place of the DS4432U and
  INA260;
- an EMC2101 whose external diode is the chip temperature (ideality 0x24);
- the VR temperature from the TPS546;
- no ASIC-enable or plug-sense GPIO; an ESP32-S3 N16R8.

The deferred parity rows are CFG-002, ASIC-008 and PWR-004.

**The urgent finding:** nothing could tell a 601 from a 205. The detector's
physical identity digests the host USB port, and ROM admission checked only
that the chip is an ESP32-S3. Phase A gates writes on the device's
registered MAC digest. The detector still cannot tell the boards apart, so
only the board a task names stays attached while it runs.

The 601 did not enumerate on the 205's USB-C cable. Upstream notes some
Bitaxe boards need a USB-A host port (bitaxeGamma#37); that is an owner
action.

- [x] **A. Board identity guard (software).**
  - A local `boards.local.json` maps board ids to digests of the ESP32-S3
    base MAC.
  - ROM admission refuses an unregistered or mismatched device before any
    effect.
  - A register command; the 205 is seeded from its kept `board-info`
    output.
  - Tests; the full gate; an independent review.
  - Done 2026-10-21: `tools/flash` `board_identity` (21 tests) gates ROM
    admission and the core-dump erase, refusing before the reset when no
    device is registered for the board. The Ultra 205 is registered from
    its kept `board-info` output. Two review rounds; full gate.
- [x] **B. Identify the 601 (hardware, its own contract).**
  - The detector, one ROM admission (chip, MAC, flash, features), then a
    hard reset back to stock.
  - The NVS read in memory, extracting only `boardversion`, `asicmodel` and
    `devicemodel`. `just identify-board` is the read-only identification
    admission: the ROM gate requests board 205, so it enters ROM without
    the gate, writes nothing, and registers only on a full match.
  - Register the 601's digest. No writes.
  - Done 2026-10-21, attempt 001:
    - `just identify-board` (`9453a1a2`) found `601`/`BM1370`/`gamma` and
      16MB flash, and registered the 601.
    - The board enumerated through a USB-C hub.
    - Three effects: one ROM entry, one NVS read, one hard reset; no
      writes.
    - Summary: `docs/parity/evidence/20261021-gamma601-identification.md`.
- [x] **C. Board seam and 601 safe boot.** Plan `gamma-601-phase-c` (owner
      approved 2026-10-21; state-preserving install). Order: no flashable
      601 package before the safe-boot build exists.
  - [x] C1a: firmware board identity, 205 behaviour unchanged.
    - [x] The stored-board identity, its report and the guard.
      - NVS `boardversion` is read once at boot and classified
        (`match | mismatch | unknown`).
      - Reported in the `usb_board_identity` record (replay slot 23) and the
        API fields `storedBoard` and `boardIdentity`, defaulted so host
        readers accept older firmware.
      - The replay cycle grew to 25 slots, and its interval shrank to
        240 ms, so it still lasts 6000 ms, the gap host judges allow.
      - The guard: another known board withholds the ASIC production handle
        and the self-test; `board_identity_source_ownership_tests` pins it.
    - [x] Built-board facts selected together in one `cfg` block in
      `firmware/bitaxe/src/board_identity.rs`:
      - the marker;
      - the boot line;
      - `BoardTarget`/`AsicTarget` (now with `Gamma601`/`Bm1370`);
      - the runtime attestation board (205/BM1366 or 601/BM1370);
      - the catalog entry behind the API's `boardVersion` and `ASICModel`,
        and the screen.
      - Not yet board-aware: the platform identity's `PlatformBoard`, which
        stays 205.
  - [x] C1c: tools board threading (`tools/flash`).
    - `BoardId::Gamma601`, parsed only by `flash`, `flash-monitor`, `monitor`
      and `detect`.
    - Per-board package target, manifest and file names.
    - Manifest board and ELF marker admission, required for both boards;
      the 205 firmware now carries `firmware_board=205`. C1b must define
      the marker in the same `cfg` block as the built board.
    - ROM admission requires the session's board.
    - A factory install stays 205 only.
    - Moved to C1b: xtask packaging for 601 and the runtime attestation
      parser. The automation runner (`invocation.ts`) and the release gate
      stay 205 only.
  - [x] C1b and C2 together. Verified in software: both images build,
    `//scripts:firmware_board_isolation_test` passes, and two independent
    reviews were applied. Not yet on hardware (C3).
    - The `board-gamma-601` feature, with `firmware_gamma601` targets and
      packages.
    - The 601 safe boot:
      - reset held low;
      - TPS546 `OPERATION=0x00` then `ON_OFF_CONFIG=0x1F` (upstream's order),
        confirmed off;
      - EMC2101 external-diode config;
      - the fan at 100 % until the output is confirmed off, then 30 %;
      - the records logged every 5 s (the 601 has no Worker replay);
      - mining structurally absent;
      - the `usb_gamma601_safe_boot` record.
    - Isolation tests per board.
  - [x] A 205 regression check on hardware after C1: `--soak regression-001`
    (contract below), which applies `watchdog-001`'s checks (a superset of
    `stopping-002`'s) and the board report. Passed 2026-10-22, attempt 002
    (`ff6b11ef`):
    - 181 s of hashing, 11 accepted shares and 0 rejected;
    - the stop moved only from `active` through `stopping` to `paused`,
      with the hashrate zeroed;
    - the watchdog was fresh in every sample, and the board report
      matched;
    - an independent recomputation agrees;
    - summary: `docs/parity/evidence/20261022-ultra205-board-seam-regression.md`.
  - [x] C3 on the 601 (contract below; owner check-in 2026-10-22: the
    install-window risk is accepted). Done 2026-10-22, attempt 001:
    - two state-preserving installs (the second is the update cycle);
    - both passed, judged under the pass criteria amended between them
      (`8fce6458`): the reset held, the TPS546D24S output confirmed off at
      10 mV from the first record (about 6.2 s of uptime), the fan at 30 %,
      and `built=601 stored=601 identity=match`;
    - Wi-Fi did not join, and the TPS546 reports a latched `CML` bit
      (follow-ups below);
    - summary: `docs/parity/evidence/20261022-gamma601-safe-boot.md`.
  - [x] Follow-up: why the 601 does not join Wi-Fi with its stock settings.
    Answered 2026-10-22, follow-up attempt 001: the stored credentials are
    valid, but the station reports `access_point_unavailable`. The owner
    reports that the 601 holds placeholder network values. Our firmware
    reads the stock settings correctly. Joined with real credentials
    through a factory install (`task-gamma-601-wifi-provisioning`,
    archived).
    The safe-boot record (schema v2) now carries the Wi-Fi owner's status
    category (`wifi=`, closed tokens only). If that is not enough, a
    key-presence report from the NVS read (names only, no values) would
    show which stored keys differ from ours.
  - [ ] Follow-up: find where the TPS546 `CML` status bit comes from.
    Narrowed 2026-10-22, follow-up attempt 001:
    - after a cold power-on, `STATUS_CML = 0x82` is already set by the
      first status read, which follows only the identity reads;
    - the sweeps add nothing;
    - the same capture showed the TPS546 output on, in regulation, at
      power-on until the safe boot's off writes. Upstream does the same.
    - A read of `STATUS_CML` before the identity read would show whether
      the identity reads set the bit.
    - Summary: `docs/parity/evidence/20261022-gamma601-follow-ups.md`. The
    record now carries `STATUS_CML` each sweep (`cml=`), and `STATUS_WORD`
    and `STATUS_CML` as read at boot before the off writes
    (`start_status=`, `start_cml=`). After a power cycle these show
    whether the safe boot's own reads set the bit.
- [x] **D. TPS546.** Plan `gamma-601-phase-d` (owner approved 2026-10-23).
  - [x] D1: the pure configuration core (`bitaxe-safety`):
    - the LINEAR11 and ULINEAR16 encoders;
    - upstream's ordered Gamma writes as data, golden-tested;
    - the `VOUT_MODE` gate (`0x97` only);
    - a target clamp to 1.00-1.20 V;
    - readback by value (raw exact, VOUT to one count, LINEAR11 to half a
      step);
    - the enable-window judge;
    - no NVM store.
    An independent review recomputed all 32 writes from upstream, byte for
    byte, and found holes that are now closed:
    - readback no longer accepts a value the read's own exponent covers;
    - the judge aborts on a stalled clock and at a sample cap, and latches;
    - the baseline may carry only CML;
    - there are plausibility floors;
    - the plan needs a confirmed-off token;
    - the NVM guard covers 0x11-0x18 and 0xF0+.
    `VOUT_MAX` is now 1.25 V (a second deviation, ADR-0041 item 6).
  - [x] D2: the safe boot reads STATUS_WORD and STATUS_CML before the
    identity read (`pre_id_status`, `pre_id_cml`; record schema v3).
  - [x] D3: the test-only variant `gamma601-vout-check` (ADR-0041 item 6):
    - the configuration, then readback and `CLEAR_FAULTS`;
    - one 60 s enable at 1.15 V with the ASIC in reset, through the judge;
    - then off, confirmed;
    - the variant plumbing, end to end, as `overheat-trip-test`.
    A second independent review led to these guards:
    - the check requires the ASIC reset held, and re-reads it every
      sample;
    - it admits only a power-on, software, USB or external reset;
    - it refuses an unfinished earlier check (RTC latch), and a
      controller that already holds the check's `VOUT_MAX` (a hardware
      anchor no rebuild moves);
    - it latches the reset pad low across the window;
    - it re-sends off before configuring, and re-confirms off after;
    - only an armed token turns the output on;
    - the thread stack is 12 KiB, and the record reports the watermark;
    - the record keeps the post-write status, read before `CLEAR_FAULTS`;
    - the production ELFs must not carry the check's record.
  - [x] D4: hardware under its own contract (below): the CML answer after
    a power cycle, the one-shot check, production reinstalled. Attempt
    002 passed (`docs/parity/evidence/20261023-gamma601-vout-check.md`).
    The configuration writes and reads back, and 1.15 V holds for 60 s
    with the ASIC in reset. Not shown: anything with the ASIC out of reset
    (phase E/F).
- [x] **E. BM1370 pure core (ASIC-008):** init, PLL, job ids and results,
      the nonce errata, version rolling; golden fixtures. Plan
      `gamma-601-phase-e` (owner approved 2026-10-23); parity work plan
      `docs/parity/work-plans/20261023T180521Z-ASIC-008/`.
  - [x] E1: the shared PLL search takes a divider range; the BM1366 path is
    unchanged, and upstream's own PLL test vector passes.
  - [x] E2: `crates/bitaxe-asic/src/bm1370*`:
    - the init plan, split at chip detection;
    - the ramp in both directions;
    - the nonce space with the errata;
    - the max baud, the job interval and the expected hashrate;
    - work frames and 24-step job ids;
    - strict result, register and chip-id decoding.
    Dispatch stays deferred.
  - [x] E3: `fixtures/bm1370/protocol-cases.json`, computed by an
    independent reimplementation of upstream's algorithms. All 99 init
    frames match, and so do the CRCs in upstream's hand-dump comments.
  - [x] E4: an independent review, then evidence, the ASIC-008 transition
    to `implemented` (`unit,golden`), and progress.
    - The review found no correctness defects. It recomputed every fixture
      value and swept the PLL and ramp against upstream.
    - The source commit is `aa7081d1`.
    - The two transitions are `deferred` → `in-progress` → `implemented`,
      as the parity tool requires for an activation.
    - Evidence: `docs/parity/evidence/asic008-bm1370-core/summary.md`.
  - Phase F prerequisites, outside this pure core:
    - the stratum job-id step, hard-coded to 8
      (`crates/bitaxe-stratum/src/v1/live_runtime.rs`);
    - firmware's `bitaxe_asic::bm1366` types and its single-chip
      `ultra_205_*` helpers;
    - the BM1366-only config validators;
    - the UART adapter;
    - a chip-counting loop for the BM1370: upstream's skip, stop and
      exact-count rules; the pure core has the per-frame parser only;
    - runtime ramps starting from an achieved frequency that is not a
      quarter-MHz value. The core's ramp takes quarter-MHz, and init's
      50 MHz start is exact.
    - fractional target frequencies (upstream stores a float), if the 601
      needs them.
    Each needs a chip-generic seam or a BM1370 twin.
- [ ] **F. BM1370 on hardware, then mining.** Chip detection, mining-ready
      init, board-specific live limits, a short local run, stops,
      self-test and soaks; the parity rows with their evidence. Plan
      `gamma-601-phase-f` (owner approved 2026-10-23), in four gated
      sub-phases.
  - [x] F1: the shared crates, pure and host-tested (`f6a623ff` and the
    commit after it).
    - [x] F1.1: board-parameterized frequency and voltage validators.
      The 601's voltage stops at the TPS546 ceiling of 1.20 V, which a
      cross-crate test pins.
    - [x] F1.2: the catalog family facts (max power, power offset,
      nominal voltage). The API's `maxPower` comes from them, and
      `expectedHashrate` is computed upstream's way. It is zero while
      paused or after a power fault; the 205 reported 0 before.
    - [x] F1.3: the neutral ASIC types (`bitaxe_asic::asic`), with the
      BM1366 names kept as aliases. The BM1370 core uses them.
    - [x] F1.4: `AsicFamily` sets the job-id step and the job interval in
      stratum. The 205's behavior is unchanged.
    - [x] F1.5: board profiles, presets and device-settings parsing. The
      validators are ranges, so upstream's minus-100/100 overheat pair is
      valid on the 601 (425/1050); no substitute pair is needed.
    - [x] F1.6: a board-parameterized safety verdict
      (`MiningSafetyLimits`):
      - power up to the catalog target plus 3 W (the 205 still 15 W; the
        601 22 W);
      - a fresh regulator temperature up to 105 C on TPS546 boards (a new
        `VrTemperature` fact).
      The 205's entry points are wrappers.
    - [x] F1.7: the BM1370 safe shutdown, lowered into adapter actions,
      and per-board admission and bounds. Every 601 frequency's plan
      matches its closed tail; at 625 MHz that is 24,550 ms.
    - [x] F1.8: the API names the 601 and the BM1370 (`PlatformBoard`,
      `PlatformAsic`), and has a per-board safe snapshot. Tests pin
      upstream's 601 values.
    - [x] F1.9: the virtual board's oracle also answers as a BM1370. A
      host test runs the core's init, chip detection, the baud switch and
      a genesis-block job to a decoded, target-meeting nonce.
  - F2 carries over from F1's review:
    - [x] the firmware's family and chip count come from board identity
      (`board_identity::ASIC_FAMILY`), and so does the platform identity's
      board and ASIC. The phase-34 guard pins the board-identity source.
      `hashrate.rs`'s single chip and four domains hold for the BM1370
      too, so it stays as pinned;
    - [x] production result classification and the boot register
      snapshot go through `AsicFamily::classify_production_result`, at the
      chain's address interval (256 over the chip count). A compile-time
      check holds the BM1370's frame at the BM1366's 11 bytes. The bounded
      diagnostic work/result path in `asic_adapter.rs` stays BM1366-only;
    - [x] the BWG and serial V2 paths' `SetupConnection` hardware version
      is the built board's version ("205" or "601"), so the 205's bytes
      do not change. Upstream sends the ASIC name and a fixed 1e12 nominal
      hashrate (`stratum_v2_task.c`); aligning all three V2 paths with that
      is a separate parity change, since it changes the 205 on the wire;
    - the V1 subscribe model `bitaxe-ultra` in `bwg.rs` is also the BWG
      capability identity (`bwg_worker_usb.rs` checks "bitaxe-ultra" and
      "205" at Worker start), so both change with the 601's BWG capability;
    - the native SV2 settings path takes its hardware version from
      `ultra_205_defaults()` (`settings_adapter/stratum_v2.rs`); it takes
      the built board's defaults;
    - the accepted-state snapshot reads the chip-id register, which the
      BM1370 register map refuses. The snapshot has no caller today; drop
      or map that read before one is added on the 601;
    - the bounded diagnostic work/result mode in `asic_adapter.rs` still
      decodes BM1366 frames; a 601 build refuses that mode or routes it
      through the family;
    - [x] V2's first ASIC job id stays 0 on both boards. Upstream's BM1370
      first sends 24; id 0 is a valid slot for both families and the
      choice is not observable outside the chip link;
    - overheat recovery is board-aware, with VR-temperature cooling on the
      601 and `reduced_profile` taking the board;
    - revocation's reported `SHUTDOWN_BOUND_MS` becomes the board's
      (`catalog_maximum_pre_reset_tail_ms_for`) when the 601's owners
      start. The fixed-USB judges pin 22,550 for the 205. Admission uses
      `profile_admissible_for_board`;
    - [x] the firmware's safety verdict call sites, and the cooling core
      through `CoolingBackend::safety_limits`, take the built board's
      `MiningSafetyLimits` (`board_identity::MINING_SAFETY_LIMITS`, now a
      `const fn` of the catalog). The safe10 inventories pin the new calls.
      The host simulation and the bitaxe-api tests keep the 205's limits;
    - [x] the TPS546 status word faults the 601's power fact
      (`tps546_power_fault`), so the verdict refuses mining, on upstream's
      `TPS546_check_status` bits: VOUT_OV, IOUT_OC, VIN_UV and TEMP, and
      `OFF` while the output should be on. The latched `CML` bit seen on
      the bench is not one of them. An unread status word is a failed
      regulator read. The safe boot passes "output expected off"; the
      production power backend passes its real state, from the raw status
      word the report now keeps. `sensor_acquisition.rs` is at 605 of 628
      lines, so the next addition there splits it;
    - the production power backend, for the regulator fault:
      - sends `CLEAR_FAULTS` at init, as upstream's `TPS546_init`, before
        polling; a VIN_UV or TEMP bit latched at power-up otherwise keeps
        the power fact faulted for the whole boot;
      - passes `output_expected_on = true` once it enables the output;
      - fills the API's `power_fault` field (`maybe_power_fault`, always
        `None` today), which upstream reports from
        `VCORE_get_fault_string`;
    - the simulation interpreter drives the BM1370 through its lowered
      adapter actions;
    - the closed safety-fact vocabularies follow the board's
      `requires_vr_temperature()`:
      - `scripts/str005-v2-serial/safety-diagnostics.mjs` `FACTS`;
      - `campaign_status.rs` `CampaignObservationRequirements::ULTRA_205`;
      - `tools/flash/src/campaign/markers.rs` and its serial tests;
    - [x] the 601 publishes the shared observations from its safe-boot
      thread (`gamma601/observations.rs`), now sweeping every 500 ms:
      - power is VOUT × IOUT plus the family's 5 W offset from the raw
        IOUT, as upstream; VIN is the input voltage; the current is IOUT;
      - with the output confirmed off, an IOUT down to -0.5 A reads as
        0 A: the idle 601 reads about -0.2 A (safe-boot evidence), which
        would otherwise fault the power fact whenever the board is idle.
        With the output on, any negative or non-finite IOUT is an invalid
        sample, so the offset can never hide power while mining. This is
        a choice for the owner to check;
      - the core voltage is VOUT; the regulator temperature is the
        TPS546's; the chip temperature is the EMC2101 external diode
        (ideality 0x24, no offset); a failed regulator read faults power
        as `tps546_read_failed`;
      - the 205's sensor runtime is unchanged; its projection and boot
        session helper are shared, not copied;
      - the safe-boot thread's stack is 8 KiB, as the 205's sensor owner
        that runs the same publish chain. The build's stack audit bounds
        `Producer::publish` (992 bytes) with `run`, and the thread logs `gamma601_safe_boot_stack free_bytes=` every 5 s for
        F3 to record;
    - F3 must show what the external diode reads with the BM1370 held in
      reset. If it faults, the idle 601 fails `baseline_safe` and the idle
      fan grant, which the board-aware overheat item must handle;
    - parity differences to record in the checklist: upstream reports the
      chip temperature as -1 until the ASIC is initialised
      (`Thermal_get_chip_temp`), and the raw negative idle current; the
      601 reports the diode reading and 0 A;
    - the vout-check variant's 250 ms samples now also read the external
      diode, two more EMC2101 reads than its passed hardware run made;
    - the 601's VIN read 5.41 to 5.47 V on the bench, inside the shared
      5 V ±10 % limit (5.5 V) with little margin;
    - `operator_sensor_runtime.rs`'s `BOARD_POWER_TARGET_WATTS` (12 W) is
      the 205's catalog target and runs only on the 205.
  - F2 mining variant:
    - [x] the test-only `gamma601-mining` variant (ADR-0041 item 7,
      `just package-gamma601-mining`), wired like the core-voltage check
      through Cargo, Bazel, the packager, the manifest, the flash tool
      (`--allow-test-firmware gamma601-mining`), the release refusal and
      the isolation gate. Only it and the check compile the TPS546
      enabling writes. The isolation gate also refuses the enable's own
      context literal (`tps546_operation_on_enable`) in a production ELF,
      and a source-ownership test keeps that literal in the gated enable.
    - [x] `gamma601/regulator.rs` holds the TPS546 steps the test variants
      share (the vout check now calls them unchanged): `VOUT_MODE` and
      status reads, the configuration's write and readback, and off with
      confirmation. The mining variant adds the production core voltage:
      - `power_on`: off until confirmed, configure and read back,
        re-confirm off, `CLEAR_FAULTS`, a clean off baseline, `OPERATION`
        on through `enable::arm_production` with the ASIC reset held, then
        after `SETTLE_MS` `OPERATION` read back on, VOUT within 50 mV of
        the target and no new fault (`enable::judge_settled`);
      - `set_vout`: a live `VOUT_COMMAND` within the 1.00-1.20 V clamp,
        only while `VOUT_MAX` and `VOUT_SCALE_LOOP` still read as this
        boot wrote them (`config::live_guard_writes`), then read back;
      - `power_on` stops at the enable and returns the baseline;
        `verify_on` judges the settled output later, so the bus owner never
        blocks for the settle (the preparation's 500 ms stabilization wait
        sits between them, as upstream's `mining_start` waits 500 ms);
    - [x] the 601's safe-boot thread is the board's safety actuation owner
      (`gamma601/actuation.rs`): between sweeps it services the shared
      queue (`service_next_board_actuation_request`). Fan commands go
      through the authority the 205 also uses (`decide_fan_duty`, factored
      out of the 205's apply with the same results) and keep the safe
      boot's rule: full while the output is neither deliberately on nor
      confirmed off (`SafeBootReport::fan_duty_with`). In the mining
      variant only, `PowerOnRegulator`, `VerifyRegulator`,
      `SetRegulatorVoltage` and `TurnRegulatorOff` drive the regulator;
      any failed step turns the output off. While an owner has the output
      on, the safe boot stops re-asserting off; once the enable is
      verified, the power fact counts `OFF` as a fault. The output never
      outlives its authority: on every pass, at least each sweep, the
      thread runs the revocation deadline check itself (every 100 ms
      while the output is on, and after each regulator command, so a
      lapsed 2.8 s heartbeat starts the off within 3 s) and turns the
      output off when the enabling permit is revoked (a lapsed Worker
      heartbeat, or an unsafe observation through the verdict) or the
      enable is unverified after 5 s. Only a Worker generation's permit,
      which lapses with its heartbeat, may turn the output on. A command
      with a revoked permit is refused and turns a live output off. An owner's fan duty is cleared
      on every enable and off. The thread sweeps and publishes just before and
      after each regulator command, so the observations stay fresh for
      the waiting owner. Production 601 builds refuse regulator commands;
      - their callers still keep the fan at full before
        the enable, holds the reset pad latched through it, turns the
        output off on any `Err`, and never runs on the thread that owns
        the 2.8 s USB heartbeat deadline (`power_on` blocks about 1 s). The vout check's off-command warning now
        logs as `gamma601_regulator`; It boots the safe boot for now; the production power
      path, preparation and dispatch land in it next;
  - F2 owners and Worker:
    - [x] the 205 regression check after F2 (`--soak regression-001`,
      contract below) passed on 2026-10-24, attempt 001 (`09490a0f`);
    - [x] dispatch admission per board: `board_identity::
      MINING_DISPATCH_ADMITTED` (the 205 always; the 601 only in
      `gamma601-mining`) gates `production_handle_available`, so a
      production 601 refuses at the preparation's first step, before the
      fan;
    - [x] the 205-only live voltage and frequency path stays off on the
      601 (`LIVE_HARDWARE_CHANGES`; its BM1366 PLL frames would reach the
      BM1370 at the shared 490, 525 and 550 MHz), and the live and local
      profile checks use `profile_admissible_for_board`. The saved device
      profile is parsed against the built board's catalog
      (`from_device_settings_for_board`), so a 601 admits its 600 and
      625 MHz and caps the voltage at its regulator's 1.20 V up front;
    - [x] the 601 starts the safety supervisor, the production owner and
      the fan controller (`gamma601::start_runtime_owners`), skipping the
      205-only self-test and diagnostic admissions; ADR-0041 item 5 is
      amended and the board-isolation gate now requires the 601's owners
      and still forbids the 205's sensor owner and ADC. The baseline stays
      unconfirmed, so no Worker. An owner that fails to start (an SV2
      protocol setting, a failed spawn) is logged and boot continues to
      HTTP. Only a build that admits dispatch grants idle fan control, so
      a production 601 keeps the safe boot's idle fan;
    - [x] the campaign status record's observation requirements follow
      the built board's limits (`vr_temp_celsius` true on the 601). The
      flash tool's campaign parser still expects the 205's set; it learns
      the 601's with the F3 campaign admission, which today admits only
      board 205. The Worker's revocation-detail grammar
      (`scripts/str005-v2-serial/safety-diagnostics.mjs` `FACTS`, a
      mirror of the Gate's) gains `vr_temperature` with the Worker work;
    - [x] the panic cutoff on the 601 is recorded as an ADR-0041 item 7
      residual risk: the existing handler drives the reset low at once and
      drives nothing on GPIO10; the regulator stays on until the next
      boot's safe boot, with the reset undriven between reboot and the
      next boot's first act, as upstream. A pad hold through the panic is
      a later change with its own fault-injection evidence;
    - F3 records the 601's boot heap and stack high-water marks in both
      builds: it now runs the owner (24 KiB), two pool workers, the ASIC
      worker, the fan controller and the supervisor beside its safe-boot
      thread;
    - [x] the thread stack audits run on the 601's production ELF too
      (`//scripts:firmware_stack_audits_gamma601_test`: control, writer,
      link, HTTP and BAP all fit, the writer with 672 bytes to spare, as on
      the 205), and the owner-entry, realignment and atomics audits pass
      on the 601 production and `gamma601-mining` ELFs;
    - the Worker needs owner decisions first: the 601 capability's model
      and revision strings (upstream's `devicemodel` is `gamma`), whether
      the V1 subscribe model follows them, the Gate repository's schema
      change and Update Authority signature, re-pinning Gate, and whether
      a production 601 runs a Worker that refuses every Start or none
      until F4;
  - F2 chip bring-up:
    - [x] `bm1370::plan` builds the bring-up as adapter actions in the
      firmware's two phases: chip detection (status, reset pulse, 115200
      baud, the version masks and `CHIP_ID` read, then the count of the
      expected chips) and the mining-ready init (upstream's frames, the
      ramp, the nonce space, then the max-baud switch, a cleared receive
      buffer and `InitializedNoMining`). Its frames match the golden
      fixture. The executor's chip count classifies frames through
      `AsicFamily::classify_chip_id_count_frame`, so a 601 counts BM1370s;
      the 205's planner and actions are unchanged;
      The 601's chip detection also takes upstream's UART flush and
      100 ms before the reset and 100 ms after the baud reset
      (`mining_start`, `asic_initialize`);
    - [x] the 601's board steps in the shared preparation and safe stop
      (`mining_actuation_adapter/gamma601.rs`), beside the 205's arms,
      which keep their pinned text:
      - `SetCoreVoltage` is `PowerOnRegulator`, the shared 500 ms wait
        follows, and `EnableAsic` is `VerifyRegulator` (the 601 has no
        enable line);
      - `ResetAndDetectExactlyOneChip` and the mining-ready init run the
        `bm1370::plan` actions through the shared executor with the
        catalog's chip count, then wait upstream's 2000 ms in 100 ms
        progress-reporting slices;
      - the stop skips the frequency ramp, holds the reset and turns the
        regulator off without the session's admission. This deviates from
        upstream's `mining_stop` order (ramp, reset, voltage off): the
        revoked lease has already cut the regulator within 100 ms, so the
        ramp would drive an unpowered chip;
      - the preparation refuses at its first step unless the executor
        holds the UART and the reset, so the regulator is never powered
        for a chip nothing can drive;
      - a production 601 refuses the power steps;
      - admission is `profile_admissible_for_board`, the same answer on
        the 205;
      - the shared `await_preparation_actuation` holds the reply wait;
    - [x] the 601's boot gate (`asic_adapter/board_601.rs`) hands the ASIC
      UART (TX 17, RX 18, as upstream's `serial.c`) and the reset, still
      held low, to the executor; the executor needs no enable line on the
      601 (`power_line_retained`). The reset pin is output-only once the
      executor owns it, so the reset owner tracks whether it holds the
      reset low (`AsicReset::holds_reset_low`, counted released before the
      release write), and `gamma601::reset_held()` asks it first. Once
      the handover starts, nothing but the retained owner confirming the
      hold reads as held: a failed handover, a busy or poisoned lock all
      read as not held, and a reset driver whose handover fails is kept
      driving low rather than dropped (a drop disables the pin). The 601 baseline
      stays unconfirmed, so no Worker starts with the 205's capability
      until the 601's owners and Worker capability land; the cooling proof after a stop needs the board-aware rule;
      the adapter struct keeps its `Ultra205` name until a rename;
    - the 205's plan lacks upstream's pre-init UART flush and the two
      100 ms settle waits; adding them changes the verified 205 path, so
      it is a separate change. Both boards fail closed on a trailing
      partial frame after the count, where upstream stops counting;
  - F3 must show the 601's 22 W power stop against real mining power at
    its catalog settings before relying on it above the default.
  - [ ] F2: the firmware seams; the 601 is still refused mining.
  - [ ] F3: hardware on the 601, each step under its own contract.
    - [x] F3.1 software: the test-only `gamma601-chip-check` variant
      (ADR-0041 item 8) and its check (`gamma601/chip_check.rs`, record in
      `bitaxe_safety::gamma601_chip_check`), wired like the mining
      variant; the shared reset-reason guard moves to
      `gamma601/check_guard.rs`; the count loop retains its last count
      (`asic_adapter::last_counted_chips`) for the record; the build's
      stack audit bounds the check's frame;
    - [x] F3.1 hardware: passed on attempt 002, exactly one BM1370
      (`docs/parity/evidence/20261024-gamma601-chip-check.md`);
      production reinstalled;
    - [x] F3.2 software: the test-only `gamma601-hash-check` variant
      (ADR-0041 item 9), the shared powered-check runner
      (`gamma601/check_power.rs`), the hash stage (`gamma601/hash_check.rs`),
      its judges and record (`bitaxe_safety::gamma601_hash_check`), the
      genesis known answer (`bm1370::known_answer`) and the firmware result
      verifier (`bitaxe_stratum::v1::verify_asic_result`);
    - [x] F3.2 hardware: passed on attempt 001, 8 of 8 results verified and
      the genesis nonce returned
      (`docs/parity/evidence/20261024-gamma601-hash-check.md`); production
      reinstalled;
    - [x] 601 Worker software (ADR-0041 item 10): every 601 build runs the
      Worker. The baseline confirms with the reset retained, the output
      confirmed off and the safe-boot thread running; the board's signed
      capability and model; a production 601 refuses Start before budget;
      the Gate pinned at `b775145`;
    - [x] 601 Worker hardware: passed on attempt 001, the Worker started
      with the baseline confirmed and the board held safe
      (`docs/parity/evidence/20261025-gamma601-worker-boot.md`);
    - [ ] 205 regression at the 601 Worker commit (`2bf484b8`) and the F3.3
      chip-temperature rule, at the next session with the 205 attached,
      before anything else on the 205;
    - [x] F3.3 software (ADR-0041 item 11): the 601's chip temperature by
      stage, the board-aware verdict, cooling sample, stop proof, overheat
      recovery and run admission, the 20 s init-window fault, the board's
      reduced profile; the soak `gamma601-first-001`;
    - [x] F3.3 hardware: passed on attempt 002, 10 minutes at 400 MHz with
      32 shares accepted and none rejected
      (`docs/parity/evidence/20261025-gamma601-first-local-mining.md`);
      production reinstalled;
    - [x] F3.2 open item from F3.1: after the BM1370 init the EMC2101
      external channel reads 37.5 to 40.0 C while hashing; the 127.75 C
      before is the uninitialized channel. The cooling proof's board rule
      can use the diode after the init;
  - [x] F3.4 (owner decisions 2026-10-25): the overheat and lease-interrupt
    stops on the 601; heartbeat loss, fault and outage injection are out of
    scope.
    - [x] Overheat software: the `gamma601-overheat-trip-test` variant
      (ADR-0041 item 12) and the overheat soak made board-aware, soak
      `gamma601-overheat-001` at 500 MHz / 1150 mV;
    - [x] Overheat hardware: passed on attempt 001, outcome `restarted`
      (`docs/parity/evidence/20261025-gamma601-overheat-check.md`);
      production reinstalled;
    - [x] Lease-interrupt software: the lease soak and the Gate soak made
      board-aware (the board from the package manifest; the 601's 22 W,
      its 400/1150 default never restored to 485/1200, its chip temperature
      required only while the work gate is open and active), soak
      `gamma601-lease-001` at 400 MHz;
    - [x] Lease-interrupt hardware: passed on attempt 005, both soaks
      `passed` (`docs/parity/evidence/20261026-gamma601-lease-interruption-check.md`);
      production reinstalled. Its fixes: the Gate tab's detached
      visibility check, the 601's soak prerequisite (ADR-0041 item 13,
      `2d4158fa`) and the held break log (`9323ff2d`).
  - [ ] F4: default profile, soak, self-test and promotion (plan
    `delightful-foraging-quill`, owner decisions 2026-10-26: port the 601
    self-test before promotion, a 4 h soak at 525/1150, the panic residual
    accepted as upstream, live changes deferred).
    - [x] F4.1 software: soaks `gamma601-default-001` (10 min) and
      `gamma601-soak-001` (4 h) at 525/1150; the stopped proof takes the
      installed board's idle limit;
    - [x] F4.1 hardware: passed on attempt 001, 10 minutes at 525/1150,
      29 shares, power 20.4 W, chip 69.5 C, regulator 76 C, the fan at its
      top (`docs/parity/evidence/20261026-gamma601-default-profile-check.md`);
      production reinstalled;
    - [x] F4.2 hardware: passed on attempt 001, 4 h at 525/1150, 440
      shares, peaks 20.9 W, chip 71.5 C, regulator 80 C, one 55.7 s Wi-Fi
      outage recovered on its own
      (`docs/parity/evidence/20261026-gamma601-4h-soak.md`); production
      reinstalled;
    - [x] F4.3 software: the self-test made board-aware (ADR-0041 item 14):
      the pure `HardwareSelfTestContract` from the catalog (the 205's values
      pinned to its verified constants), the 601's regulator fault and
      temperature checks, its bounded local-generation authority and fan
      path, the 601 start path, and per-board flash intents, campaign,
      evidence and validator, with the 601's plan
      (`docs/parity/work-plans/20261026T000000Z-SELF-001-GAMMA601/PLAN.md`);
    - [ ] F4.3 hardware: the 601's two-phase lifecycle on `gamma601-mining`;
    - [ ] F4.4: promotion (software): production 601 mines, mines on boot
      by its saved preference, `gamma601-mining` retired;
    - [ ] F4.5: promotion hardware: boot resume, the production self-test,
      a production smoke soak;
    - [ ] F4.6: parity proposals; the 205's regression and self-test at
      the final commit.

Each phase ends with the full gate, an independent review, commit and
push. Each hardware step needs its own contract and enable line below.

Phase B identification contract (attempt ordinals 001 onward):
- **Objective:** name the attached board from its own records and register
  it. `just identify-board --board 601 --port <port> --register` reports:
  - the flash size from `board-info`;
  - the stored `boardversion`, `asicmodel` and `devicemodel`, as known
    values or the categories `other` / `missing`;
  - whether they match the 601 (`601`, `BM1370`, `gamma`).
  It registers the base-MAC digest only on a full match. The command
  refuses unless this block is active and carries its enable line.
- **Precondition:** only the 601 attached, with barrel power on and USB
  through a USB-A host port or adapter. `just detect-ultra205 --port
  <port>` finds exactly one Espressif Serial/JTAG device; it cannot tell
  the boards apart.
- **Allowed effects:**
  - one reset into the ROM downloader (`board-info`);
  - one read of the NVS partition (0x9000, 0x6000);
  - one hard reset back to the installed stock AxeOS, which then runs as
    before, including whatever its own settings do.
- **Prohibited effects:** any flash write or erase, any NVS write, any
  firmware install, any power, fan or ASIC command from the host, and
  network discovery.
- **Evidence and privacy:**
  - The NVS partition holds Wi-Fi and pool secrets (`NeverPersistRaw`). It
    passes from esptool to `nvs_tool` only through private named pipes
    (`memory_fifo`) and memory; it is never a stored file.
  - The command prints categories only, and the registry holds a digest.
  - The device-session trace keeps the raw `board-info` output, MAC
    included, as protected operational data in its private root. No MAC,
    digest, SSID or other NVS value is printed to the terminal or
    committed.
- **Evidence capture:** each attempt runs in a new mode-0700 parent under
  `scratch/gamma-601-identification/`, with distinct mode-0600 stdout and
  stderr files for the detector and the command. The record names board
  601, the selected port, the source and reference commits, the exact
  commands, the categories reported and the conclusion.
- **Recovery:** if the device stays in the ROM downloader after a failure,
  the owner power-cycles it (barrel and USB). That is a non-invasive manual
  remediation.
- **Stops:**
  - zero or several ports;
  - records that are missing (`records_missing`) or name another board
    (`records_name_another_board`);
  - a digest registered for another board;
  - any refusal of the detector or the gate.
  Retries follow the progress-gated attempt policy.

Phase C3 safe-boot contract (attempt ordinals 001 onward; owner accepted the
install-window risk on 2026-10-22):

Closed after attempt 001 passed (2026-10-22); no enable lines remain.

- **Objective:** install the 601 safe-boot package (`8a58c262` or a later
  verified fix) state-preserving, observe the safe boot, then install the
  same package again as the update cycle and observe it again.
- **Commands, in order, each attempt:**
  1. `just detect-ultra205 --port <port>`: exactly one Espressif
     Serial/JTAG device.
  2. `just identify-board --board 601 --port <port>` (no `--register`): the
     records still name `601`/`BM1370`/`gamma`, and the digest is the
     registered 601's.
  3. `just flash-monitor --board 601 --port <port> --capture-timeout-seconds
     360`: the flash tool builds the 601 package at HEAD, admits only the
     registered 601, writes its five update segments state-preserving and
     captures the receive-only console.
  4. The detector again, then step 3 again (the update cycle).
- **Precondition:** only the 601 attached, barrel power on, USB through the
  USB-C hub. The working tree is clean and pushed.
- **Allowed effects:**
  - the ROM entries, the NVS read and the hard resets of steps 1 and 2;
  - two state-preserving installs of the 601 package (bootloader, partition
    table, application, `www`, otadata), each with its resets;
  - the installed firmware's own effects (ADR-0041 item 5):
    - the ASIC reset held low;
    - the TPS546 commanded off;
    - the EMC2101 diode configuration and fan duty;
    - Wi-Fi with the stored settings.
- **Prohibited effects:**
  - a factory install, any NVS write from the host, `--wifi-credentials`;
  - any TPS546 write other than the off writes, any VOUT enable, any ASIC
    release or command, mining;
  - network discovery, HTTP requests to the device;
  - direct UART or pins.
- **Pass, per install,** from the captured console:
  - the build: the flash tool admitted the package only with
    `firmware_board=601` in its application, and
    `usb_board_identity built=601 stored=601 identity=match` is reported.
    Amended after attempt 001: the receive-only console carries the
    device's replay and diagnostic records, not the general log. So the
    boot line, `firmware_board=601` and `mining_owner=absent` log lines
    are not observable there.
  - `usb_gamma601_safe_boot` with `reset_held=true`, a recognised `tps546`
    (`d24a` or `d24s`), and `off=confirmed` within 30 s of boot, staying
    confirmed. The records carry no uptime, so a record's uptime is taken
    from the nearest `usb_startup` record before it.
  - `vout_mv` under 200, `fan_duty_pct=30`, `fan_rpm` above 0, and a board
    temperature between 10 and 60 C.
  - Wi-Fi joining is recorded but not required.
  - `faults` is recorded but not judged: the safe boot never clears
    latched bits, so they may predate this boot.
- **Stops:**
  - zero or several ports, an identification mismatch, or any refusal of
    the flash tool;
  - after 30 s of boot, `off` other than `confirmed`, a `tps546` other
    than `d24a`/`d24s`, `reset_held=false`, or a VR temperature above 90 C
    or a board temperature above 70 C. Then the owner removes barrel power
    and the attempt stops (`stop_hardware_blocker`).
  - Retries follow the progress-gated attempt policy.
- **Evidence and privacy:**
  - Each attempt runs in a new mode-0700 parent
    `scratch/gamma-601-safe-boot/attempt-NNN/`, with distinct mode-0600
    stdout and stderr files per command.
  - The console may contain SSIDs, IPs and MACs, so it stays private.
  - A committed summary names board 601, the commits, the commands,
    categories and the readings above; it passes `just verify-redaction`.
- **Recovery:**
  - If the safe boot misbehaves, the owner removes barrel power.
  - If the device stays in the ROM downloader, the owner power-cycles it.
  - Returning to stock AxeOS (an upstream release flashed with upstream
    tools) is the owner's choice and outside this contract.

Follow-up diagnostics contract (attempt ordinals 001 onward):

Closed after attempt 001 (2026-10-22); no enable line remains.

- **Objective:** observe the Wi-Fi status category and the TPS546 `STATUS_CML`
  readings from the schema v2 safe-boot record. First after an install,
  then after a power cycle of the board, so that `start_cml` shows what the
  safe boot's own identity reads leave behind.
- **Commands, in order, each attempt:**
  1. `just detect-ultra205`: exactly one device;
  2. `just flash-monitor --board 601 --port <port>
     --capture-timeout-seconds 360`: one state-preserving install of the
     601 package at HEAD, with its capture;
  3. the owner removes barrel power and USB, waits about 10 s, then
     restores barrel power and then USB. This is a manual, non-invasive
     power cycle;
  4. the detector, then `just monitor --board 601 --port <port>
     --capture-timeout-seconds 360`: a receive-only capture, with no reset
     and no write.
- **Precondition:** only the 601 attached, as in C3; the tree clean and
  pushed.
- **Allowed effects:** the detector checks, one state-preserving install
  with its resets, the owner's power cycle, a receive-only capture, and the
  installed safe boot's own effects (ADR-0041 item 5).
- **Prohibited effects:** as in the C3 contract. In particular, no
  `CLEAR_FAULTS` or any TPS546 write beyond the off writes, no NVS writes,
  and no HTTP or network discovery.
- **Observations, not pass criteria:**
  - the `wifi=` category over time;
  - `start_status`, `start_cml` and `cml`, after the install and after the
    power cycle.
  The C3 safety readings must still hold: reset held, off confirmed, fan
  running.
- **Stops:** the C3 contract's stops. The capture and evidence rules are
  the C3 contract's, under `scratch/gamma-601-follow-ups/attempt-NNN/`.
- **Recovery:** as in C3.

Phase D4 core-voltage check contract (attempt ordinals 001 onward; plan
`gamma-601-phase-d`, owner approved 2026-10-23):

Closed after attempt 002 passed (2026-10-23); no enable line remains.

- **Objective:** three steps.
  - Observe `pre_id_status` and `pre_id_cml` after a power cycle, on
    production with D2.
  - Run the one-shot check on `gamma601-vout-check`.
  - Reinstall production and confirm the safe boot.
- **Commands, in order, each attempt:**
  1. `just detect-ultra205`: exactly one device.
  2. `just flash-monitor --board 601 --port <port>
     --capture-timeout-seconds 360`: production at HEAD, installed
     state-preserving.
  3. The owner power-cycles the board: barrel and USB off, about 10 s,
     then barrel, then USB. Then the detector again, and `just monitor
     --board 601 --port <port> --capture-timeout-seconds 360`, a
     receive-only capture.
  4. `just package-gamma601-vout-check`, then the detector, then `just
     flash-monitor --board 601 --port <port> --manifest
     bazel-bin/firmware/bitaxe/gamma601/bitaxe-gamma601-package.json
     --allow-test-firmware gamma601-vout-check --capture-timeout-seconds
     360`: the variant, installed state-preserving. The capture spans the
     check, which starts 30 s after boot and lasts about 65 s. The
     check's `usb_gamma601_vout_check` record takes turns with the
     safe-boot record in the replay. While the check record reads
     `running`, the safe-boot record reads `off=unread`, its readings
     `unavailable` and `fan_duty_pct=100`. That is expected, and exempt
     from the C3 stops: `off` is judged from the check record then.
  5. The detector, then step 2 again: production reinstalled, and the safe
     boot confirmed.

  An attempt after a verified fix may begin at step 3 when the previous
  attempt's step 5 left production with D2 installed and the owner has
  just removed all power. The owner's power cycle is then step 3's.
- **Precondition:** only the 601 attached; the tree clean and pushed. The
  owner is present and can remove barrel power at any moment.
- **Allowed effects:**
  - the detector checks;
  - three state-preserving installs with their resets;
  - one owner power cycle;
  - a receive-only capture;
  - the safe boot's own effects (ADR-0041 item 5);
  - the variant's check (ADR-0041 item 6), once per boot:
    - the Gamma configuration writes, volatile and never stored;
    - their readback;
    - `CLEAR_FAULTS`;
    - fan at 100 %;
    - one enable of at most 60 s at 1.15 V with the ASIC held in reset;
    - then off.
- **Prohibited effects:**
  - any NVM store or restore, and any factory install or NVS write;
  - an ASIC reset release, ASIC UART traffic, mining;
  - a second enable in a boot, and a target other than 1.15 V;
  - leaving the variant installed;
  - HTTP to the device, and network discovery;
  - direct UART or pins.
- **Pass:**
  - Step 3 records `pre_id_status` and `pre_id_cml`, whatever they read.
  - Step 4's `usb_gamma601_vout_check` shows:
    - `result=passed`, `readback=match` and `samples` of at least 230;
    - `vout_mv_min`/`max` within 1100-1200. This includes samples taken
      before the 1 s settle, which the judge allows; where the two
      disagree, this criterion wins;
    - `iout_ma_max` under 5000 and `vin_mv_min` of at least 4800;
    - `vr_temp_c_max` under 80 and `board_temp_c_max` under 60;
    - `off=confirmed` and `stack_free` of at least 2048.
    `config_faults` and `config_cml` are recorded, not judged, and so is
    the fan speed. The
    safe-boot record then shows `off=confirmed`, `reset_held=true` and
    `fan_duty_pct=30`.
  - Step 5 shows the C3 safe-boot readings on production.
- **Stops:**
  - `result=aborted` or `refused`: investigate before any retry.
    - A refusal named `previous_reset_*`, `previous_check_unfinished` or
      `controller_already_configured` is a hardware stop: no software
      retry until the owner has removed all power (barrel and USB).
    - Any other refusal before the enable is a software finding, and a
      fix may retry under the progress-gated attempt policy. The
      controller keeps the check's `VOUT_MAX` until its power is removed,
      so each retry after a configuration write also needs that power
      cycle.
    - A first refusal at readback is possible: the controller may store
      some limits at its own fixed exponent. The record names the register
      and both values.
    - If `controller_already_configured` repeats on the first boot after
      all power was removed, the stock `VOUT_MAX` matches the check's
      value. That is a software finding, not another hardware stop.
  - A reboot or new boot during step 4's capture, other than the
    install's own reset at its start, or a check record still `running`
    at the end of the capture: the owner removes all power, barrel first,
    then USB.
  - No `usb_gamma601_vout_check` record in the capture, or one still
    `pending` at its end: an evidence failure. Stop, reinstall production
    and fix before any retry.
  - `off` other than `confirmed` after the check: the owner removes all
    power, barrel first, then USB (`stop_hardware_blocker`).
  - The C3 stops.
  - Any refusal of the detector or the flash tool.
- **Evidence and privacy:** as in C3, under
  `scratch/gamma-601-vout-check/attempt-NNN/`. The committed summary names
  board 601, the commits, the commands, each install's manifest
  `firmware_variant` and the record values above, and passes `just
  verify-redaction`.
- **Recovery:**
  - The owner removes all power (barrel and USB). That clears the
    controller's volatile configuration and the RTC latch.
  - Reinstall production (step 5) after any attempt, passed or not.

D4 attempts:
- **Attempt 001** (2026-10-23, `cc6bf00f`):
  - **Steps 1-3 passed.** Production's warm install read
    `pre_id_status=0x0842 pre_id_cml=0x82`. After the owner's power
    cycle, the first transfer read `pre_id_status=0xffff pre_id_cml=0x80`,
    the start read `0x0002`/`0x80` (output on, as in phase C), and the
    steady state read `cml=0x82`.
  - **Step 4 was an evidence failure.** The check ran: `faults` and `cml`
    went to `0x0000`/`0x00`, which only its `CLEAR_FAULTS` after a
    readback can do. The safe-boot records then showed `off=confirmed`,
    `reset_held=true` and `vout_mv=20`. But its record went only to the
    general log, which the receive-only console does not carry, so the
    outcome is unknown.
  - **Step 5 passed.** Production was reinstalled and confirmed. It read
    `pre_id_cml=0x00 start_cml=0x00`, then `cml=0x02`: the safe boot's
    own transfers after the start read set CML bit 1.
  - **Fix:** the record is now replayed in turn with the safe-boot record
    (a pure `ReplayTurn`, tested). It reads `running` while the check
    runs. Meanwhile the safe-boot record carries no readings and the
    check's fan duty.
  - **Outcome:** `continue_after_verified_fix`. Evidence is under
    `scratch/gamma-601-vout-check/attempt-001/`.
  - The controller keeps the check's configuration, so attempt 002 needs
    all power removed first.
- **Attempt 002** (2026-10-23, `4a7c5717`): passed.
  - It began at step 3, after the owner removed all power.
  - The check passed: `readback=match`, 240 samples, 1144-1146 mV, peak
    710 mA, `off=confirmed` and 8236 bytes of stack free.
  - Production was reinstalled and confirmed.
  - The cold boot read `pre_id_cml=0x00`. CML bit 7 needs the too-early
    first read seen in attempt 001; bit 1 comes from the safe boot's own
    transfers.
  - Evidence: `scratch/gamma-601-vout-check/attempt-002/` and
    `docs/parity/evidence/20261023-gamma601-vout-check.md`.
  - Outcome: `complete`.

Phase C 205 regression contract (attempt ordinals 001 onward):

Closed after attempt 002 passed (2026-10-22); no enable line remains.

- **Objective:** `--soak regression-001` on production firmware at the
  clean, pushed HEAD, installed state-preserving. It runs `watchdog-001`'s
  checks:
  - 3 minutes of hashing with a nonzero hashrate and an accepted share;
  - a pause whose status moves only from `active` to `stopping` to
    `paused`;
  - a zeroed hashrate while stopping;
  - a fresh task watchdog in every sample.
  Before any write, the baseline must also report `storedBoard` `205` with
  `boardIdentity` `match`. On the 205's real paths, this checks the phase C
  changes: the board-aware flash admission and ROM gate, the USB replay
  changes, the attestation board and the board guard.
- **Precondition:** only the 205 attached (barrel power and its direct
  USB-C cable), registered as board 205; the tree clean and pushed.
- **Commands, with `P` a new mode-0700 parent under
  `scratch/local-mining-soak/` and `umask 077`:**
  1. `just detect-ultra205`;
  2. `just flash-monitor --board 205 --port <port>
     --capture-timeout-seconds 360`, state-preserving;
  3. `just otawww-endpoint preflight|serve|finish --private-root
     P/endpoint`, with a Connect in the dedicated Gate tab;
  4. `just local-mining-soak preflight --soak regression-001 --private-root
     P/attempt-NNN --endpoint-root P/endpoint --package-manifest
     bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json
     --pool-credentials pool-credentials.json`;
  5. `just local-mining-soak run`, detached under `caffeinate`, as the soak
     README describes.
- **Allowed effects:**
  - the install, with its resets;
  - the Gate Connect for the handoff;
  - the soak's own effects: saving the owner's pool, up to 15 minutes of
    local mining at the default pair, one pause resent idempotently, and
    the recovery pause.
- **Prohibited effects:**
  - a factory reset;
  - a Gate Connect during the run;
  - any other hardware command;
  - network discovery;
  - direct UART or pins;
  - printing or copying the pool credentials.
- **Evidence, recovery, retries and accepted outcomes:** as `watchdog-001`
  (archived `task-preparation-rollback-cooling-watchdog`). Recovery is
  `just local-mining-soak pause --private-root P/attempt-NNN`.

Attempt 001 (2026-10-22): `soak_failed` before any write; outcome
`continue_after_verified_fix`.
- **Before the run:**
  - the production install (`335e9f6c`) reported
    `built=205 stored=205 identity=match`;
  - a first endpoint handoff was cut short by a Chrome crash, and its
    server was released;
  - a second handoff and the soak preflight passed.
- **The failure:** the run's first `GET /api/system/info` threw before the
  baseline was journaled. A failure to reach the device carried no closed
  code, so the soak reported only `soak_failed`. The device stayed paused,
  with no write.
- **Fix (instrumentation of the failed boundary):** the soak's device
  client now reports `soak_device_timeout` or `soak_device_unreachable`
  (`device.test.mjs`). The next attempt uses a fresh handoff.

Phase F 205 regression contract (attempt ordinals 001 onward):

Closed after attempt 001 passed (2026-10-24); no enable line remains.

- **Objective:** `--soak regression-001` on production Ultra 205 firmware
  at the clean, pushed HEAD after the phase F2 shared-code changes,
  installed state-preserving. It runs `watchdog-001`'s checks:
  - 3 minutes of hashing with a nonzero hashrate and an accepted share;
  - a pause whose status moves only from `active` to `stopping` to
    `paused`;
  - a zeroed hashrate while stopping;
  - a fresh task watchdog in every sample;
  - before any write, a baseline reporting `storedBoard` `205` with
    `boardIdentity` `match`.
  On the 205's real paths this checks the F2 changes: the chip family and
  platform identity from board identity, results through `AsicFamily`, the
  board-aware safety limits and cooling core, the shared fan authority,
  the extracted preparation reply wait, per-board dispatch admission and
  profile parsing.
- **Precondition:** only the 205 attached (barrel power and its direct
  USB-C cable), registered as board 205; the tree clean and pushed.
- **Commands, with `P` a new mode-0700 parent under
  `scratch/local-mining-soak/` and `umask 077`:**
  1. `just detect-ultra205`;
  2. `just flash-monitor --board 205 --port <port>
     --capture-timeout-seconds 360`, state-preserving;
  3. `just otawww-endpoint preflight|serve|finish --private-root
     P/endpoint`, with a Connect in the dedicated Gate tab;
  4. `just local-mining-soak preflight --soak regression-001 --private-root
     P/attempt-NNN --endpoint-root P/endpoint --package-manifest
     bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json
     --pool-credentials pool-credentials.json`;
  5. `just local-mining-soak run`, detached under `caffeinate`, as the soak
     README describes.
- **Allowed effects:**
  - the install, with its resets;
  - the Gate Connect for the handoff;
  - the soak's own effects: saving the owner's pool, up to 15 minutes of
    local mining at the default pair, one pause resent idempotently, and
    the recovery pause.
- **Prohibited effects:**
  - a factory reset;
  - a Gate Connect during the run;
  - any other hardware command;
  - network discovery;
  - direct UART or pins;
  - printing or copying the pool credentials.
- **Evidence, recovery, retries and accepted outcomes:** as `watchdog-001`
  (archived `task-preparation-rollback-cooling-watchdog`). Recovery is
  `just local-mining-soak pause --private-root P/attempt-NNN`. Retries
  follow the progress-gated attempt policy.

Attempt 001 (2026-10-24, `09490a0f`): passed; outcome `complete`.
- 181 s of hashing, 13 accepted shares and 0 rejected, no outages;
- the stop moved only from `active` through `stopping` to `paused`, with
  the hashrate zeroed and no `active` sample at idle power;
- the watchdog was fresh in every sample, and the board report matched;
- an independent recomputation agrees on every metric;
- summary: `docs/parity/evidence/20261024-ultra205-f2-regression.md`.

Phase F3.1 chip check contract (attempt ordinals 001 onward), closed:
attempt 002 passed (`docs/parity/evidence/20261024-gamma601-chip-check.md`)
and production is reinstalled.

- **Objective:** the first BM1370 chip detection. The test-only
  `gamma601-chip-check` firmware (ADR-0041 item 8), installed
  state-preserving at the clean, pushed HEAD, runs its one check 30 s after
  boot and replays `usb_gamma601_chip_check`. Passing means
  `result=passed`, `chips=1`, VOUT within 50 mV of 1150 mV at the verify,
  and `off=confirmed`; the record also gives the diode before and after,
  the peak IOUT and regulator temperature and the window from the
  power-on sequence to the off command (`on_ms`).
- **Precondition:** only the 601 attached (barrel power and USB through a
  USB-A port), registered as board 601; all power removed at least once
  since the core-voltage check, so the controller holds no earlier
  configuration; the tree clean and pushed.
- **Commands, with `P` a new mode-0700 parent under
  `scratch/gamma-601-chip-check/` and `umask 077`:**
  1. `just detect-ultra205`;
  2. `just package-gamma601-chip-check`;
  3. `just flash-monitor --board 601 --port <port> --manifest
     bazel-bin/firmware/bitaxe/gamma601/bitaxe-gamma601-package.json
     --allow-test-firmware gamma601-chip-check --capture-timeout-seconds
     360`, state-preserving; its capture holds the replayed records.
     Without `--manifest` the tool builds the production package and
     refuses the allowance (attempt 001, no effect);
  4. `just package --board 601`, then `just flash --board 601 --port
     <port>`, state-preserving, and confirm the boot reports the production
     variant.
- **Allowed effects:** the two installs and their resets; the one check:
  the TPS546 configured and on at 1.15 V, about 6 s to the off command on
  a healthy bus and at most about 16 s on a degraded one (ADR-0041 item
  8), the ASIC reset released only for the chip count; the fan at 100 %
  for the check. The firmware refuses a second check until all power is
  removed.
- **Prohibited effects:** work frames, a frequency change, mining, a
  second check without all power removed, a restart, OTA or other HTTP
  request, flash, reset or other USB reset while the check may run (from
  30 s after boot until its record concludes; step 4 starts only after),
  a factory reset, any other hardware command, network
  discovery, direct UART or pins.
- **Evidence and privacy:** the capture stays in `P` (mode 0600 files);
  committed evidence is a redacted summary of the closed record's fields,
  checked independently, with `just verify-redaction`.
- **Recovery:** the production reinstall in step 4. If the record shows the
  output not confirmed off, remove all power (barrel and USB) before
  anything else; the next boot's safe boot commands it off.
- **Stop rules:** `refused`, `aborted`, a count other than one, or off not
  confirmed: stop after the production reinstall and analyse before another
  attempt (progress-gated policy).

Phase F3.2 hash check contract (attempt ordinals 001 onward), closed:
attempt 001 passed (`docs/parity/evidence/20261024-gamma601-hash-check.md`)
and production is reinstalled.

- **Objective:** the BM1370's first hashing. The test-only
  `gamma601-hash-check` firmware (ADR-0041 item 9), installed
  state-preserving at the clean, pushed HEAD, runs its one check 30 s after
  boot and replays `usb_gamma601_hash_check`. Passing means
  `result=passed`, `chips=1`, at least 3 verified results with invalid ones
  at most a tenth, and `off=confirmed`; the record also gives the diode
  after the init (`diode_gate_c`, `diode_max_c`), the telemetry extremes,
  the best difficulty and the on time. `genesis_hit` is information only.
- **Precondition:** only the 601 attached (barrel power and USB through a
  USB-A port), registered as board 601. The owner removes all power (barrel
  and USB) once before attempt 001 and before any later attempt, since the
  firmware refuses a second check until then. The tree clean and pushed.
- **Commands, with `P` a new mode-0700 parent under
  `scratch/gamma-601-hash-check/` and `umask 077`:**
  1. `just detect-ultra205`;
  2. `just package-gamma601-hash-check`;
  3. `just flash-monitor --board 601 --port <port> --manifest
     bazel-bin/firmware/bitaxe/gamma601/bitaxe-gamma601-package.json
     --allow-test-firmware gamma601-hash-check --capture-timeout-seconds
     360`, state-preserving; its capture holds the replayed records;
  4. `just package --board 601`, then `just flash --board 601 --port
     <port>`, state-preserving, then `just monitor --board 601 --port
     <port> --capture-timeout-seconds 360` to confirm production
     (`identity=match`, no check record).
- **Allowed effects:** the two installs and their resets; the one check:
  the TPS546 configured and on at 1.15 V for at most 45 s, chip
  detection, the BM1370 init and ramp to 300 MHz, one genesis job hashed
  for at most 20 s; the fan at 100 % for the check.
- **Prohibited effects:** a pool or any Stratum traffic, any other
  frequency or voltage, a second check without all power removed, a
  restart, OTA or other HTTP request, flash, reset or other USB reset while
  the check may run (from 30 s after boot until its record concludes; step
  4 starts only after), a factory reset, network discovery, direct UART or
  pins.
- **Evidence and privacy:** the capture stays in `P` (mode 0600 files);
  committed evidence is a redacted summary of the closed record's fields,
  checked independently against the captures, with `just
  verify-redaction`.
- **Recovery:** the production reinstall in step 4. If the record shows the
  output not confirmed off, remove all power (barrel and USB) before
  anything else; the next boot's safe boot commands it off.
- **Stop rules:** `refused`, `aborted` (including `diode_unusable`), a
  failed pass rule, or off not confirmed: stop after the production
  reinstall and analyse before another attempt (progress-gated policy).

Phase F3 601 Worker contract (attempt ordinals 001 onward), closed:
attempt 001 passed (`docs/parity/evidence/20261025-gamma601-worker-boot.md`).

- **Objective:** the production 601 package at the clean, pushed HEAD
  boots with its Worker ready, its board still held safe.
- **Precondition:** only the 601 attached, registered as board 601; the
  tree clean and pushed.
- **Commands, with `P` a new mode-0700 parent under
  `scratch/gamma-601-worker/` and `umask 077`:**
  1. `just detect-ultra205`;
  2. `just package --board 601`;
  3. `just flash-monitor --board 601 --port <port>
     --capture-timeout-seconds 360`, state-preserving.
- **Passing means:** the capture shows the Worker prepared on the 601 (no
  `boot_baseline_unconfirmed`, `recovery_failed` or Worker start failure),
  `built=601 stored=601 identity=match`, the safe-boot records with
  `reset_held=true` and `off=confirmed`, and one boot.
- **Allowed effects:** the one production install and its reset; the
  first Worker boot writes its Device Identity seed to NVS.
- **Prohibited effects:** any test variant, a Gate session, a Worker Start,
  a factory reset, HTTP requests, network discovery, direct UART or pins.
- **Evidence and privacy:** the capture stays in `P` (mode 0600 files);
  committed evidence is a redacted summary of closed fields, with `just
  verify-redaction`.
- **Recovery:** if the Worker is unavailable or the board is not held
  safe, the production package from before this change
  (`59c178a9`) is reinstalled with the same command.
- **Stop rules:** any failed criterion: stop and analyse before another
  attempt (progress-gated policy).

Phase F3.3 first local mining contract (attempt ordinals 001 onward),
closed: attempt 002 passed
(`docs/parity/evidence/20261025-gamma601-first-local-mining.md`) and
production is reinstalled.

- **Objective:** the 601's first local mining. The test-only
  `gamma601-mining` firmware at the clean, pushed HEAD mines for 10 minutes
  at 400 MHz / 1150 mV with the owner's pool through the soak
  `gamma601-first-001`, with at least one accepted share, then pauses and
  proves stopped; production is then reinstalled.
- **Precondition:** only the 601 attached, registered as board 601; the
  tree clean and pushed; the owner's dedicated Gate tab available for the
  endpoint handoff.
- **Commands, with `P` a new mode-0700 parent under
  `scratch/gamma-601-first-mining/` and `umask 077`:**
  1. `just detect-ultra205`;
  2. `just package-gamma601-mining`;
  3. `just flash-monitor --board 601 --port <port> --manifest
     bazel-bin/firmware/bitaxe/gamma601/bitaxe-gamma601-package.json
     --allow-test-firmware gamma601-mining --capture-timeout-seconds 360`,
     state-preserving;
  4. `just otawww-endpoint preflight|serve|finish --private-root
     P/endpoint`, with Connect in the dedicated Gate tab;
  5. `just local-mining-soak preflight --soak gamma601-first-001
     --private-root P/attempt-NNN --endpoint-root P/endpoint
     --package-manifest bazel-bin/firmware/bitaxe/gamma601/bitaxe-gamma601-package.json
     --pool-credentials pool-credentials.json`; then `just
     local-mining-soak pause --private-root P/attempt-NNN`, which turns the
     saved start-on-boot preference off if it reads on (the factory default)
     and confirms the device paused and stopped, the baseline `run`
     requires; then `run --private-root P/attempt-NNN --pool-credentials
     pool-credentials.json` under `caffeinate`;
  6. `just package --board 601`, `just flash --board 601 --port <port>`,
     state-preserving, then `just monitor --board 601 --port <port>
     --capture-timeout-seconds 360` to confirm production.
- **Allowed effects:** the installs and their resets; the endpoint
  handoff; the start-on-boot preference saved off, the pool and the
  400 MHz / 1150 mV profile saved over HTTP; 10 minutes of local mining
  with the owner's pool; the soak's pause and stop. The mining build never
  mines on boot (ADR-0041 item 11), so nothing mines before the soak's
  resume.
- **Prohibited effects:** any other frequency or voltage, a Worker Start
  or lease, a factory reset, network discovery beyond the handoff, direct
  UART or pins. Pool values never leave `P` or the local credential file.
- **Stops:** the soak's live stops (22 W, chip 75 C, regulator 90 C, input
  outside 4.5 to 5.5 V, fan, stale readings, a reset, a fault) and the
  firmware's own. Any stop ends the attempt: pause, reinstall production,
  analyse before another attempt (progress-gated policy).
- **Evidence and privacy:** the captures and journals stay in `P` (mode
  0600 files); committed evidence is a redacted summary of closed fields,
  checked independently, with `just verify-redaction`; no pool URL, port,
  user, worker or password.
- **Recovery:** `just local-mining-soak pause --private-root P/attempt-NNN`,
  then the production reinstall in step 6. If the board does not stop,
  remove all power (barrel and USB).
- **Attempt 001 (`771f24e7`), stopped:** the endpoint handoff, the soak's
  preflight and pause (start-on-boot saved off, the device paused and
  stopped) and the run's pool and 400 MHz profile all passed. About 1 s
  after the resume the run's generation was revoked on the power fact
  (`state=fault`, about 5.0 W: the offset alone, so the regulator's output
  was barely up), the run ended `Failed`, the 601's 30 s cooling proof held
  it, and the soak was stopped and the device paused (`mine_on_boot_off`,
  `paused`, `asic_stopped`). Cause, most likely: with the output no longer
  confirmed off, any negative IOUT was an invalid power sample, and the
  regulator's offset (about -0.2 A on this 601) reads negative before the
  BM1370 draws a load. The log could not name the cause.
  - **The fix:** the offset floor also applies while the BM1370 draws no
    hashing load (`gamma601_diode::chip_unloaded`: not yet initialized, or
    the regulator commanded off); an initialized chip's negative reading
    with the regulator on stays invalid. The 601's producer now logs each
    change of the power fact's fault cause with the raw IOUT, VOUT and
    status (at most one line per 10 s), so a recurrence names its cause.
  - **Outcome:** `continue_after_verified_fix` once the fix is gated,
    reviewed and pushed; attempt 002 takes a fresh handoff (the reinstall
    changes the boot) and a new private root.
  - **Deviation:** the stop rule's production reinstall did not happen
    before the analysis. The device stayed paused with the chip off on the
    mining build until attempt 002's install; production was reinstalled
    after attempt 002. The device also retried the run once after its
    restart backoff, stopping the same way, before the soak was
    interrupted and the device paused.

Phase F3.4 overheat contract (attempt ordinals 001 onward), closed:
attempt 001 passed (`docs/parity/evidence/20261025-gamma601-overheat-check.md`)
and production is reinstalled.

- **Objective:** the 601's overheat stop and recovery on hardware. The
  test-only `gamma601-overheat-trip-test` firmware (55 C local trip) at the
  clean, pushed HEAD mines at 500 MHz / 1150 mV through the soak
  `gamma601-overheat-001` until the trip; the device stops, cools under the
  601's rule (the chip unpowered, the regulator at or below 95 C, at least
  30 s), restarts at the reduced 400 MHz / 1050 mV and hashes. A second
  trip there stays stopped, since 300 MHz is below the catalog. Passing
  means the soak's `passed` with outcome `restarted` or
  `restarted_then_stays_stopped`, hashing after the restart in either.
- **Precondition:** only the 601 attached, registered as board 601; the
  tree clean and pushed; the owner's Gate tab available for the endpoint
  handoff; the device's saved pair 400/1150 (F3.3's).
- **Commands, with `P` a new mode-0700 parent under
  `scratch/gamma-601-overheat/` and `umask 077`:**
  1. `just detect-ultra205`;
  2. `just package-gamma601-overheat-trip-test`;
  3. `just flash-monitor --board 601 --port <port> --manifest
     bazel-bin/firmware/bitaxe/gamma601/bitaxe-gamma601-package.json
     --allow-test-firmware gamma601-overheat-trip-test
     --capture-timeout-seconds 360`, state-preserving, confirming the
     boot marker;
  4. `just otawww-endpoint preflight|serve|finish` with the owner's
     Connect in the Gate tab;
  5. `just local-mining-soak preflight --soak gamma601-overheat-001
     --private-root P/attempt-NNN --endpoint-root P/endpoint-NNN
     --package-manifest <manifest> --pool-credentials pool-credentials.json`,
     then `pause`, then `run` under `caffeinate`;
  6. `just package --board 601`, `just flash --board 601 --port <port>`,
     state-preserving, and `just monitor --board 601 --port <port>
     --capture-timeout-seconds 360` to confirm production. The settings
     themselves are confirmed before this step by the soak's own restore:
     it writes back the journaled baseline (400/1150, the baseline's fan
     settings, `overheat_mode 0`) and requires it to hold for 60 s with the
     device stopped; a state-preserving install keeps them.
- **Allowed effects:** the installs and their resets; the handoff;
  start-on-boot saved off; the pool and the 500/1150 profile saved over
  HTTP; mining at 500/1150 until the trip; the device's own overheat
  writes; the restart at 400/1050, mining for up to 5 minutes or until a
  second trip; the harness's settings restore, one clearing resume at
  400/1150 and the pause.
- **Prohibited effects:** any other pair, a Worker Start or lease, a
  factory reset, a manual resume, network discovery beyond the handoff,
  direct UART or pins. Pool values never leave `P` or the credential file.
- **Stops:** the soak's live stops (22 W, chip 75 C, regulator 90 C, input
  outside 4.5 to 5.5 V, fan, stale readings, a reset, a fault) and a missed
  trip (the chip at 60 C or more while hashing). Any stop: pause, reinstall
  production (step 6) **before** analysing, then analyse (progress-gated
  policy).
- **Evidence and privacy:** captures and journals stay in `P` (mode 0600);
  committed evidence is a redacted summary of closed fields, checked
  independently against the captures, with `just verify-redaction`.
- **Recovery:** `just local-mining-soak pause --private-root P/attempt-NNN`
  (it restores the settings an overheat run changed); if the recovery does
  not settle, keep the device paused, let it cool and run `pause` again;
  then step 6. If the board does not stop, remove all power.
- **Risks:** power at 500 MHz is unmeasured (about 19 W by scaling F3.3,
  under the 22 W stop); 1050 mV at 400 MHz is untested on this chip. The
  cleanup's clearing resume at 400/1150 follows the 601's cooling rule,
  which cannot read the chip; if the die is still at 55 C it trips again
  and the device pauses itself with `overheat_mode` set, so the restore
  reads unconfirmed. The 55 C and 75 C trips still protect the chip;
  accepted, with `pause` as the recovery.

Phase F3.4 lease interrupt contract (attempt ordinals 001 onward):

Closed after attempt 005 passed (2026-10-26); no enable line remains.

- **Objective:** a Gate Worker lease takes over from local mining on the
  601 and hands back. The local soak `gamma601-lease-001` (400 MHz /
  1150 mV) mines 5 minutes; the Gate soak's first Connect is refused and
  stops local mining; after the reload, a signed soak lease mines 600
  active seconds; local mining is back within 3 minutes of the lease's end
  and mines 5 more minutes; the device ends paused. Passing means the
  local soak's `passed` and the Gate soak's `passed`.
- **Precondition:** only the 601 attached, registered as board 601; the
  tree clean and pushed; the dedicated Gate tab in the Chrome profile that
  holds the 601's Web Serial grant; the device's saved pair 400/1150.
- **Commands, with `P` a new mode-0700 parent under
  `scratch/gamma-601-lease/` and `umask 077`:**
  1. `just detect-ultra205`; `just package-gamma601-mining`; `just
     flash-monitor --board 601 --port <port> --manifest
     bazel-bin/firmware/bitaxe/gamma601/bitaxe-gamma601-package.json
     --allow-test-firmware gamma601-mining --capture-timeout-seconds 360`;
  2. the endpoint handoff (`just otawww-endpoint preflight|serve|finish`);
  3. `just local-mining-soak preflight --soak gamma601-lease-001
     --private-root P/local-NNN --endpoint-root P/endpoint-NNN
     --package-manifest <manifest> --pool-credentials pool-credentials.json
     --lease-root P/gate-NNN`, then `pause`, then `run` detached;
  4. once the local soak reports `lease_ready`: the Gate's browser bundle
     built at the pinned commit (`bun run build:browser` in the Gate repo
     at `b7751451`, if not already current); `just detect-ultra205 >
     P/detector.log`; within 60 s of it, `just ultra205-soak preflight
     --private-root P/gate-NNN --firmware-root <repo> --gate-root <gate
     repo> --firmware-commit <HEAD> --gate-commit
     b7751451cf51c787d7523dbd44076d8e5c3ea882 --manifest <manifest>
     --authority-directory <the acceptance authority directory whose lease
     key the firmware trusts> --detector P/detector.log
     --interrupt-local-mining`; `serve` detached with `--pool-credentials
     pool-credentials.json`; the page's steps in the dedicated tab (the
     refused Connect, the reload and Connect, then `startObserver()`, the
     reload and Connect as the 15-second step below, the idle
     proof, the reviews, `prepareStartAuthorization()`,
     `loadSignedWindow()`, `startWindow()`, `submitSoakCompletion()`), the
     tab to `about:blank`; then `owner-finish` or `finish`;
  5. after the local soak's verdict, `just package --board 601`, `just
     flash --board 601 --port <port>`, state-preserving, and `just monitor
     --board 601 --port <port> --capture-timeout-seconds 360`.
- **Allowed effects:** the installs; the handoff; start-on-boot saved off;
  the pool and the 400/1150 profile saved; local mining before and after;
  one refused Connect and its ordered safe-stop; one signed soak lease
  (600 active seconds at the saved 400/1150) signed through the repo's
  authority tooling from the protected directory; the soak ledger charged
  once; the Gate soak's start-of-soak settings write, the 601's default
  400/1150 with the automatic fan (no other profile is admitted, so nothing
  else is restored).
- **Prohibited effects:** live changes, any other profile, a second lease,
  a factory reset, network discovery beyond the handoff and the Gate soak's
  own station read, direct UART or pins. Pool values and lease secrets
  never leave `P`; no key file is read except by the repo's signing
  tooling.
- **Stops:** the local soak's live stops (the 601's 22 W, chip 75 C,
  regulator 90 C, input voltage, fan, stale readings in the local phases, a
  reset, a fault) and the Gate soak's judges. Any stop: pause, reinstall
  production before analysing, then analyse (progress-gated policy).
- **Evidence and privacy:** captures, journals and Gate results stay in
  `P` (mode 0600); committed evidence is a redacted summary, checked
  independently, with `just verify-redaction`.
- **Recovery:** `just local-mining-soak pause --private-root P/local-NNN`
  (a pause during the lease holds and confirms once the lease ends); stop
  the Gate server and seal with `finish`; then the production reinstall.
- **The 15-second step (amended after attempt 001):** the README's rule is
  reload and Connect within 15 s of `startObserver()`. Take a screenshot of
  the connected page first for the Connect button's position. Then run
  `startObserver()` (awaited), the reload and the Connect click at that
  position as one browser batch, with no other tool call between them.
  Read the page state only afterwards. The Gate journal's times show the
  gap.

Lease attempts:
- **Attempt 001** (2026-10-25, `f8ef921e`): an operator timing failure.
  - **Passed:** the `gamma601-mining` install and boot, the endpoint
    handoff, the local soak to `lease_ready` (5 minutes mining), the Gate
    preflight. The first Connect was refused at `hello` and local mining
    went through its ordered safe stop to `safe_blocked` at 5.0 W. The
    reload and Connect then reached `ready`.
  - **Failed:** the Gate journal puts the reload 21.5 s after
    `startObserver()` and the next Connect 67 s after the reload, 88 s
    against the 15-s rule. The delay was tool round trips on the
    operator's side. Local mining resumed after the quiet link, as the
    README says, so the second Hello was refused after the observer
    started. No grant was signed and the ledger was not charged.
  - **Verdicts:** the Gate soak `unverified` (`completion_missing`); the
    local soak `failed` (`lease_check_not_restored`,
    `lease_check_lease_failed`, `lease_check_after_short`). No live stop,
    reset or fault. The device ended paused, with start-on-boot off.
  - **Recovery:** the page released to `about:blank`, `owner-finish`
    sealed the Gate soak, the local pause confirmed, and production was
    reinstalled and confirmed (57 safe-boot records, `identity=match`,
    `reset_held=true`, `off=confirmed`, one boot) before the analysis.
  - **Outcome:** `continue_after_manual_remediation`: the 15-second step
    above. Attempt 002 shows the boundary changed only if its gap is under
    15 s. Captures are under `scratch/gamma-601-lease/`.
  - **Corrected by attempt 002:** the page's `disconnected` 21.5 s after
    `startObserver()` was not the reload. It carried
    `serialFailureCategory: operation_failed`, the same signature as
    attempt 002 below. The late reload was not the cause.
- **Attempt 002** (2026-10-25, `6e810a56`): the same link loss; the cause
  found.
  - **Passed:** the install and boot, the handoff, `lease_ready`, the
    refused first Connect and its safe stop, the reload and Connect, and
    the 15-second step: the reload and Connect reached `ready` 2.7 s after
    `startObserver()`.
  - **Failed:** 20.4 s after `ready`, with no page call in flight, the page
    went `disconnected` (`operation_failed`). Local mining resumed 17 s
    later, so the idle proof read `mining_observed` and could not pass.
    Verdicts as attempt 001: the Gate soak `unverified`
    (`completion_missing`), the local soak `failed`. No live stop, reset
    or fault. No grant was signed and the ledger was not charged.
  - **Recovery:** as attempt 001. Production was reinstalled and confirmed
    (56 safe-boot records with `reset_held=true off=confirmed`,
    `identity=match`, one boot) before the analysis.
  - **Cause:** in the Gate, `foreground()` is `document.visibilityState ===
    "visible"`, and a foreground loss surfaces as `operation_failed`. Claude
    in Chrome keeps a tab visible only while it is attached. A listener in
    the dedicated tab, with no tool call for 45 s, recorded `hidden` 20.0 s
    after the last call and `visible` again only when the next call
    attached. Both failures came about 20 s after the last browser call.
    The Chrome window holding the tab is covered, so the page lost the
    foreground each time the extension detached. The pre-click checks read
    `visible` because they ran while attached
    (`lesson-extension-visibility-is-not-the-owner-view`).
  - **Outcome:** `continue_after_manual_remediation`: the owner brings the
    window forward. The proof that the boundary changed is the same
    listener test reading `visible` throughout, run before attempt 003's
    Gate steps. Captures are under `scratch/gamma-601-lease/attempt-002/`.
- **Attempt 003** (2026-10-25, `47921e18`): the link held; the device
  refused the Start.
  - **The tab:** the 40-s detached test first read `hidden` again. The
    Gate tab was a background tab in the extension's own window. Once the
    owner selected it, the test read `visible` for 50 s.
  - **Passed:** the install and boot, the handoff, `lease_ready`, the
    refused first Connect and its safe stop, the reload and Connect, the
    15-second step, the idle proof (85.9 s, 43 HTTP samples, no broken
    events, no mining), the cooling and budget reviews (soak ledger
    `next_ordinal 1`, nothing charged), `prepareStartAuthorization()` and
    `loadSignedWindow()`.
  - **Failed:** `startWindow()` was rejected in 227 ms (`command_rejected`,
    `control_failure: session_failed`), and the page closed the link. No
    mining under the lease. Verdicts as before: the Gate soak `unverified`,
    the local soak `failed`.
  - **Recovery:** as attempt 001; production reinstalled and confirmed (56
    safe-boot records, `identity=match`, one boot) before the analysis.
  - **Likely cause (code reading; the refusal reason goes only to the USB
    console, which the page held):** `worker_soak_budget::admit` refuses
    unless the original acceptance campaign is complete at 240,000 ms
    (`soak_budget=original_incomplete`). The 205 completed it in
    September. The 601's Worker has never run it, so its soak ledger is
    fresh. The F3.4 plan missed this prerequisite.
  - **Outcome:** `stop_authority_boundary`: running a 601 acceptance
    campaign or changing the soak prerequisite needs the owner's decision.
    Captures are under `scratch/gamma-601-lease/attempt-003/`.
  - **Owner decision (2026-10-25):** the 601 skips the original-campaign
    prerequisite (ADR-0041 item 13). The fix makes it a per-board
    `SoakPrerequisite` (pure, tested) and keeps every Start refusal's label
    in the retained log. Attempt 004 continues as
    `continue_after_verified_fix`; it also reads the retained log after the
    Start, so a different refusal is named rather than guessed.
- **Attempt 004** (2026-10-25, `2d4158fa`): the lease ran; one judge
  failed.
  - **Passed:**
    - the detached visibility test (530 s, never hidden);
    - the install and boot, the handoff, `lease_ready`, the refused Connect
      and its safe stop, the reload and Connect, the 15-second step, the
      idle proof (84.2 s), the reviews and the Start;
    - the lease: 600,000 active ms at 400/1150, 34 shares accepted and 0
      rejected, 28 renewals, ended by its budget
      (`lease_or_budget_expired`), safe stop complete at `fan_paused`,
      restoration and baseline confirmed, soak ordinal 1 charged once;
    - after it, local mining back 21 s after the lease result, its 5
      minutes, and the pause.
  - **Failed:** the Gate soak `unverified` (`window_websocket_gap_exceeded`),
    so the local soak `failed` (`lease_check_lease_failed` only).
    - About 11 s after the Start, as the lease's mining began, the Gate
      observer saw a 6.36 s WebSocket gap against the 5 s bound, and HTTP
      missed polls for 8.2 s (two failed).
    - **Cause (corrected after analysis):** our own local soak, not the
      device. Its break watcher (added `5e2ef1b7`, the day after the 205's
      lease check) took the handover as a mining break and, on the first
      lease-mining sample, read the whole 512 KiB retained log. The device
      serves that read in 4 KiB chunks on its single web server task, so
      HTTP and the queued WebSocket frames waited about 6 s. The capture
      (`break_log` at 492,906 ms, 524 KB) and the next local sample 8.1 s
      later bracket the gap. The owner did not stall: its watchdog feeds
      and telemetry checkpoints kept their rates through the gap, and the
      Gate observer received the queued frames within 225 ms after it.
    - For the rest of the lease the largest gaps were 0.9 s (WebSocket)
      and 2.2 s (HTTP). The 205's lease check predates the break watcher.
  - **Recovery:** the page released, `owner-finish`, the local pause,
    production reinstalled and confirmed (56 safe-boot records,
    `identity=match`, one boot). The production boot read
    `boot_ordinal=1`: its RTC boot counter began a new epoch (a power
    interruption or a discarded record). The run itself kept one boot
    session throughout. Unexplained; recorded.
  - **Outcome:** `continue_after_verified_fix`. The lease check now holds
    the break log from the local stop until the Gate soak's result, and
    reads it once local mining is back (a run-level regression asserts no
    read during the lease). Captures are under
    `scratch/gamma-601-lease/attempt-004/`.
- **Attempt 005** (2026-10-26, `9323ff2d`): passed.
  - The detached visibility test read visible for 530 s.
  - The local soak `passed`: 300 s before, the refused Connect 25 s after
    `lease_ready`, local mining back 20.8 s after it read the Gate soak's
    result,
    300 s after, no outage, at most 16.09 W, chip 60.75 C, regulator
    61.5 C.
  - The Gate soak `passed`: idle proof 83.9 s (42 HTTP samples), a 600-s
    lease at 400/1150
    with 41 shares accepted and 0 rejected, 28 renewals, 20 of 20 windows
    credited, gaps at most 772 ms (WebSocket) and 2,187 ms (HTTP), soak
    ordinal 2 (attempt 004's lease was ordinal 1), cleanup confirmed.
  - The held break log was read once, after the restore.
  - Production reinstalled and confirmed (57 safe-boot records,
    `identity=match`, one boot, the Worker baseline confirmed).
  - Evidence: `docs/parity/evidence/20261026-gamma601-lease-interruption-check.md`.
- **Before each Gate session (amended after attempt 002):** in the
  dedicated tab, record `visibilitychange` events, make no browser call
  for 40 s, then read the record. Begin the Gate steps only if the tab
  stayed `visible`. Otherwise hold, with the device paused, until the owner
  brings the window forward. Never keep the page visible through tool
  keep-alives.

Phase F4.1 default profile contract (attempt ordinals 001 onward):

Closed after attempt 001 passed (2026-10-26); no enable line remains.

- **Objective:** the 601 at upstream's default. The test-only
  `gamma601-mining` firmware at the clean, pushed HEAD mines 10 minutes at
  525 MHz / 1150 mV with the owner's pool through the soak
  `gamma601-default-001`, with at least one accepted share, then pauses and
  proves stopped; production is then reinstalled. It measures power,
  temperatures and hashrate at the default before the 4 h soak.
- **Precondition:** only the 601 attached, registered as board 601; the
  tree clean and pushed; the dedicated Gate tab passing the detached
  visibility check (40 s with no browser call, never hidden).
- **Commands, with `P` a new mode-0700 parent under
  `scratch/gamma-601-default/` and `umask 077`:**
  1. `just detect-ultra205`; `just package-gamma601-mining`; `just
     flash-monitor --board 601 --port <port> --manifest
     bazel-bin/firmware/bitaxe/gamma601/bitaxe-gamma601-package.json
     --allow-test-firmware gamma601-mining --capture-timeout-seconds 360`,
     state-preserving;
  2. `just otawww-endpoint preflight|serve|finish --private-root
     P/endpoint-NNN`, with Connect in the dedicated Gate tab;
  3. `just local-mining-soak preflight --soak gamma601-default-001
     --private-root P/local-NNN --endpoint-root P/endpoint-NNN
     --package-manifest <manifest> --pool-credentials
     pool-credentials.json`, then `just local-mining-soak pause
     --private-root P/local-NNN`, then `just local-mining-soak run
     --private-root P/local-NNN --pool-credentials pool-credentials.json`
     under `caffeinate`;
  4. `just package --board 601`, `just flash --board 601 --port <port>`,
     state-preserving, then `just monitor --board 601 --port <port>
     --capture-timeout-seconds 360` to confirm production.
- **Allowed effects:** the installs and their resets; the endpoint
  handoff; start-on-boot saved off, the pool and the 525/1150 profile
  saved over HTTP; 10 minutes of local mining; the soak's pause and stop.
  After a stop, as capture for its analysis (ADR-0030): `just
  local-mining-soak stopped-proof`, then one core-dump read with `just
  hardware-operator`'s recovery helper, before the production reinstall;
  the analysis itself waits for the reinstall.
- **Prohibited effects:** any other frequency or voltage, a Worker Start
  or lease, a factory reset, network discovery beyond the handoff, direct
  UART or pins. Pool values never leave `P` or the local credential file.
- **Stops:** the soak's live stops (22 W, chip 75 C, regulator 90 C, input
  outside 4.5 to 5.5 V, fan, stale readings, a reset, a fault) and the
  firmware's own. Any stop: pause, reinstall production before analysing,
  then analyse (progress-gated policy).
- **Evidence and privacy:** captures and journals stay in `P` (mode 0600);
  committed evidence is a redacted summary, checked independently, with
  `just verify-redaction`; no pool values.
- **Recovery:** `just local-mining-soak pause --private-root P/local-NNN`,
  then the production reinstall. If the board does not stop, remove all
  power (barrel and USB).

Phase F4.2 4 h soak contract (attempt ordinals 001 onward):

Closed after attempt 001 passed (2026-10-26); no enable line remains.

- **Objective:** 4 hours at the 601's default. The test-only
  `gamma601-mining` firmware at the clean, pushed HEAD accumulates 240
  minutes of mining at 525 MHz / 1150 mV within 300 minutes, with the
  owner's pool, through the soak `gamma601-soak-001` (soak 006's rule:
  outages the device recovers from within 60 s are tolerated), then pauses
  and proves stopped; production is then reinstalled.
- **Precondition:** as F4.1; it runs unattended.
- **Known margin (F4.1):** at steady state 20.3 W and chip 69 C with the
  fan at its top. A warmer room can reach the 75 C stop; that ends the
  attempt as a stop, not a pass.
- **Commands:** F4.1's, with `P` a new mode-0700 parent under
  `scratch/gamma-601-soak/` and the soak `gamma601-soak-001`. `run` goes
  detached under `nohup caffeinate -is`, because the agent's background
  tasks end at 2 h; its journal and exit are watched.
- **Allowed effects, prohibited effects, stops, evidence, recovery:**
  F4.1's, with 240 minutes of local mining at 525/1150 in place of 10.

Phase F4.3 self-test contract (attempt ordinals 001 onward):

Not enabled: the factory installs apply the Wi-Fi credentials, which needs
the owner's request (the Wi-Fi rule above). The enable line is added once
the owner asks.

- **Objective:** SELF-001's lifecycle on the 601, per the immutable plan
  `docs/parity/work-plans/20261026T000000Z-SELF-001-GAMMA601/PLAN.md`: one
  exact-package two-phase BOOT-button campaign under a mode-0700 root. The
  planned failure reaches `cancel_ready` with `safe_state:true`; the owner's
  BOOT press cancels it; the pass phase passes, restarts automatically, and
  the settings are restored.
- **Precondition:** only the 601 attached, registered as board 601; the
  tree clean and pushed; the owner's request to apply the Wi-Fi
  credentials; the owner available at the board for one BOOT press.
- **Commands, with `R` = `scratch/self001-gamma601/attempt-NNN` (absent
  before the start) and `umask 077`:**
  1. `just detect-ultra205`; `just package-gamma601-mining`;
  2. `just self-test-campaign start --board 601 --private-root R
     --package-manifest <manifest> --wifi-credentials wifi-credentials.json
     --pool-credentials pool-credentials.json --detector-output <detector>
     --plan <plan> --projection
     docs/parity/evidence/self001-gamma601/attempt-NNN/self-test-projection.json`;
  3. at `cancel_ready`, the owner holds BOOT for two seconds;
  4. `just self-test-campaign resume` with the same options;
  5. `just package --board 601`, `just flash --board 601 --port <port>`,
     state-preserving, and `just monitor --board 601 --port <port>
     --capture-timeout-seconds 360` to confirm production.
- **Allowed effects:** the factory installs with the owner's Wi-Fi
  credentials and the tuple; the settings backup and restore over HTTP;
  the regulator on at 1150 mV for at most 240 s per phase under the local
  generation; 525 MHz; fan 10 to 100 %; diagnostic work only; the
  campaign's recovery install; the production reinstall.
- **Prohibited effects:** pool work, a Worker Start or lease, any other
  profile or test build, direct UART or pins, a retry without a recorded,
  verified change.
- **Stops:** the plan's (any unplanned failure; the output not confirmed off
  within 4 s; chip 70 C, regulator 105 C, 22 W; the 240 s bound; a reset or
  panic in a phase window; a missing checkpoint or receipt). Any stop:
  production reinstalled before analysing.
- **Evidence and privacy:** captures stay in `R` (mode 0600 under 0700); the
  validated projection holds digests, constants and booleans only;
  committed evidence is checked independently, with `just
  verify-redaction`.
- **Recovery:** the campaign's own recovery install and settings restore,
  then the production reinstall.

## Future

### task-stop-paths-hardware-evidence | 2026-10-21 | Hardware evidence for stops other than a pause

The `stopping` status (`d95d7002`), the zeroed hashrate (`38b9e932`), the
polled cooling proof (`ba22b7fb`) and the fed task watchdog (`bf98d091`) have
hardware evidence for a local pause only. A Gate lease interrupt, a network or
pool outage, an overheat trip and a fault share the same paths.

- [ ] One batched run under its own contract. The lease interruption check
      (`lease-001`) could gain the `stopping-002` and `watchdog-001` rules.
- [ ] Then cite the October firmware evidence in the parity checklist (a
      passage-only revision), together with the stopping, polled-cooling and
      preparation-watchdog summaries.

### task-owner-blocking-followups | 2026-10-21 | Remaining owner-thread stalls and progress cleanups

Residuals of `task-safe-stop-polled-cooling` and
`task-preparation-rollback-cooling-watchdog`:
- [ ] The stop's power-down still blocks the owner for about 7 s (the
      frequency ramp-down). The hashrate is zeroed first, but campaign status
      and live-hardware service stall. Consider an incremental ramp.
- [ ] Chip detection's drain is one ASIC action of up to 5 s (about 1.2 s
      normally); progress before it bounds the gap to that action.
- [ ] Self-test reports its rollback under `effect_prepare_hardware`. The
      Stratum v2 session maps preparation progress inline; share one mapping.
- [ ] A failed preparation's rollback still holds the owner, now fed, for up
      to about 127 s.

### task-ultra205-virtual-board-validation | 2026-09-30 | Full functional virtual Ultra 205 and pre-flash validation

Status: Verified software milestone; full qualification blocked by missing actual
margin/call-closure proof and unsupported mandatory integration coverage. The
corrected-target panic is resolved in emulation (campaign004, below).
Cross-reference | 2026-10-02: archived `task-device-noise-worker-stack` resolved
run017's composed Noise boundary in emulation (campaign004) and passed a real
device Noise handshake (attempt-003). Other mandatory profiles and coverage
remain open. Later virtual-path changes (`654338d0` generation-gate layout,
`24af10be`/`00f84eae` movsp-safe queues, `92f58abb` PSRAM preference) postdate
the recorded results and rotate their source bindings.
Objective: reuse production control logic in deterministic host and Xtensa/QEMU
scenarios, model every Ultra 205 functional peripheral, then qualify one canonical
pre-flash gate. References: `task-str005-v2-accepted-share-probe`, its immutable
status001/recovery006 results, and the retained installed ce8f/453d package.

- [ ] Extract shared host-buildable runtime interfaces and production logic;
  preserve physical policies, existing host regressions and historical readers.
- [ ] Implement seeded virtual BM1366, sensor/fan/voltage/ADC/display/input,
  persistence/reset/core, networking and control-transport models with explicit
  unsupported outcomes and independent model-conformance evidence.
- [ ] Add capability-aware memory/fragmentation and named-phase allocation faults,
  including observed3451-free/2176-largest and8192-internal allocation profiles.
- [ ] Exercise composed startup/Start/status/Stop/recovery with real protocol,
  admission/accounting and local fixture; preserve earliest typed failure and
  separate qualification, historical resources and current safety.
- [ ] Integrate pinned SDK5.5.4 QEMU esp_develop_9.2.2_20250817, managed checksum
  bootstrap/doctor and an unmistakable virtual dual-core S3/16MB/8MB-OPI target.
  Physical flash must reject virtual artifacts; report paired configuration diffs.
- [ ] Provide Bazel-backed virtual-board, qualify-virtual-board and preflash-validate
  commands, versioned source/package/config/model/emulator/validator-bound results.
- [ ] Qualify complete healthy lifecycle, all mandatory faults, target panic/core
  decoding, resource/ownership tests, independent goldens/mutations, repeatability
  and host/QEMU semantic agreement. Unsupported required coverage blocks qualification.
- [ ] After baseline qualification only, enforce centrally selected local profiles
  for affected flash/update/live commands before discovery or sensitive inputs.
  Emergency Stop/Close/failure-only recovery remain independently available.
- [ ] Run affected Bazel suites, ordered Cargo fmt/Clippy/build/tests, production-ELF
  audits, standards/reference/redaction/Markdown/diff checks. Commit and push verified
  milestones with truthful remaining gaps; archive this task only when all pass.

Execution contract: local software/test artifacts and synthetic virtual effects only.
No physical device detection, USB access, real signer/pool credentials, hardware
flashing, physical mining/grants, ROM operations or physical faults. Synthetic
signed frames are permitted only inside isolated virtual scenarios. All existing device-effect
flags remain disabled. Virtual builds are separately identified and cannot become
flashable physical candidates. Synthetic NVS/identities/trust and bounded private
local evidence only; no modification of sealed predecessor inventories or raw dumps.
Pinned upstream reference remains read-only. Functional models are not electrical,
RF, cycle-accurate or physical USB timing proof. Keep calibration gaps and modeled
memory distinct from actual SDK observations; no hardware parity promotion.
This task promotes no parity row (`docs/parity/progress.jsonl` is authoritative);
accepted-share stays active/unresolved. Changed SDK/model/source/
package/validator identities invalidate affected results. Unknown changes select the
full validation suite. Missing required profile, calibration or process-release proof
is a concrete blocker, never success or permission for another hardware attempt.

Local milestone review | 2026-09-30: runtime/model/command foundations implemented.
Initial ordered Cargo checks and 17 affected Bazel targets pass; the production
package builds. Host step-5 command passes; healthy host deliberately returns
unsupported for missing encrypted share integration. No device access occurred.
Full qualification and mandatory rollout remain unresolved; do not archive.

QEMU diagnostic continuation | 2026-09-30: pinned virtual guest component probes
prove boot, 8192-byte internal allocation/release, task join, NVS reset increment
and deliberate panic/core decoding. Healthy composed execution instead faults in
SDK TLSF during possession canonical serialization. Splitting the guest UART
reader reduces the dispatch frame from4800 to832 bytes with the same16384-byte
main stack, but the healthy failure recurs. This is virtual-target evidence;
physical status001's cause remains unproved. Preserve every prior private run.

Bounded next diagnostic contract: one instrumented synthetic healthy QEMU run
only, paired exact virtual ELF/source/config and pinned emulator; allocation-free
heap-integrity and task-stack observations before controller/possession boundaries,
plus an offline native call-frame audit. Allowed: local compilation, virtual flash
copies, GDB on the owned emulator and private logs/core inspection. Prohibited:
physical detection/USB/network access, signer credentials, flashing, mining,
grants, partition clearing, changed stack/safety limits and blind retries. Bound
emulator runtime to30 seconds and debugger output/time; always release/reap the
owned process group and retain partial evidence if core capture is unavailable.
Stop this diagnostic on recurrence after a targeted correction, unproved process
release, source/digest drift or missing meaningful crash state. Broader independent
host/model integration may continue without another target retry.

QEMU diagnostic015 successor | 2026-09-30: diagnostic014 was consumed without
resuming the paused guest because the pinned debugger rejected a command. Its
partial result and released process groups remain retained. Offline preflight now
passes against exact virtual ELF
b76fdd3dd942dbebc3b47143530ae772c54b83fd35f15a7cfbf1aa6a36a7ce36,
including supported debugger syntax, exported checkpoint symbols, 384-byte record
layout and verified Xtensa argument register. Permit exactly one successor
`just virtual-emulator diagnose-heap --manifest <retained-build014>/virtual-package.json --evidence-dir <repo>/scratch/virtual-emulator/diagnose015-health`.
The prior local-only effects, 30-second emulator/25-second debugger bounds,
privacy, no limit increases, cleanup and stop conditions apply unchanged. This
examines the frozen pre-extension guest; it cannot qualify the current broader
source or establish the physical failure cause. No further checkpointed target
retry is authorized by this continuation.

Corrective local qualification continuation | 2026-09-30: diagnostic015 measured
9280 stack bytes available at the pre-controller checkpoint. Its own144-byte
frame and the32-byte observer frame end before construction, giving9456 bytes
available at the constructor callsite. The exact frozen guest's resolved required
trust-key validation path alone needs10752 bytes (before additional curve leaves):
a1296-byte deficit. This is a demonstrated virtual caller-frame insufficiency,
not proof of the physical status001 cause or a complete callgraph upper bound.

Split the shared scenario initializer from the larger Start/status execution frame
and return an owned boxed controller; preserve the16KiB configured stack, SDK
extra512 bytes, all safety/authority/accounting limits and observer order. Verify
host corpus equality and build a separately marked corrected virtual package with
fresh source/ELF/config bindings. Offline audit must show the initialization frame
and constructor path fit with the existing margin before target execution. Then
permit one fresh synthetic `healthy-lifecycle` seed1 QEMU run only, with status and
allocation probes, bounded to30 seconds, in a new private evidence root. No
checkpoint diagnostic, new limit, physical discovery/effect or old grant replay.
Stop on recurrence or missing release/proof; save a precise blocker and continue
only independent host checks. Ordinary full qualification is still not authorized
by a passing component probe; all mandatory profiles and rollout criteria remain.

Evidence review correction | 2026-09-30: require natural host completion separately
from resource release; verify scenario/seed/source/package/executable bindings,
embed compiler inputs rather than attest to a later checkout, and verify copied
frozen manifest/config digests. New host run-v3 records preserve earlier v1/v2
results. Resolve compiler-stamp build setup/API errors with focused offline checks
before broad qualification. No previous result or seal may be rewritten.

Corrected target outcome | 2026-09-30: fresh guest build017 pairs the production
ELF27594c9f92e84e907d1f08acd87ea6a1c0644168a45a83c8dcfbbeeacf282b22;
virtual ELF579fdd34473fe17245f36d7e026fbd68e1041b0a7408a1cb3f958d4b030eb98e.
Further separation retires board construction and trust-validation temporaries
before controller assembly. The ordinary wrapper-inclusive selected native path
needs10752 bytes plus992 of observed leaf/iteration allowance:11744 total, leaving
4640 against the unchanged16384-byte configuration. This is a selected lower-bound
check; the complete call closure remains unproved.

The one authorized `run017-health` target attempt then failed: boot, task join,
internal allocation/release and allocator observations passed, but an unexpected
panic/reset prevented the scenario and actual stack-margin results. Host process
group release passed. The retained core's checksum and full ELF identity verify;
SDK fake frames are absent. The meaningful fault is LoadStorePIFAddrError in SDK
find_containing_heap, reached through heap_caps_free and a Rust Vec drop during
Noise mix_hash/Initiator completion inside the authenticated Start path. The
constructor progressed; the new failure's cause and actual margin remain unproved.
Raw core/debugger/registers remain private under run017-health/decoded-core. Stop further target execution under the recurrence rule;
retain exact ELF/logs/flash and any partial core. Constructor margin alone cannot
qualify target execution or establish the remaining cause. No device access occurred.

Preflash observation outcome | 2026-09-30: `just preflash-validate` saved a sealed
working-tree observation with42 passed/21 unsupported host cases across63 rows;
all63 target corpus cases were withheld because required host integration is
unsupported. Five exact production native audits passed. The development source
was dirty, full qualification was false, no package was frozen and no flash occurred.
The original report remains immutable; unavailable comparison coverage is typed
unsupported by the successor validator, not a demonstrated semantic mismatch.

Noise diagnosis continuation | 2026-10-01 | Approved retained-core and minimal probe

- [x] Independently verify retained run017 checksum/full ELF and pinned SDK task
  ABI; inspect Vec deallocation layout, stack bounds and captured heap data in a
  new private analysis root. Preserve raw dumps and offline001 findings unchanged.
- [x] Implement a host/target Noise-only probe using the actual production
  NoiseInitiator and the existing synthetic responder/transport shape; isolate
  controller/board/work/grants. Verify authentication, frame bounds, MAC/truncation
  rejection and phase ordering locally; retain a specific red-capable signal.
- [ ] Add allocation-free phase callbacks for SDK heap integrity and actual stack
  observations; keep diagnostics local to the marked virtual target. Build and
  audit exact ELF/caller frames and the existing2KiB margin before execution.
- [ ] Publish and push the complete repo-owned command/contract and verification
  receipts before a fresh bounded emulator attempt. Use fresh private evidence,
  exact package/source/config/model/emulator/validator bindings and verified release.
- [ ] Compare the minimized target result with retained full-scenario failure;
  state ranked falsifiable hypotheses before any additional diagnostic variant.
  Apply only an evidence-backed lifetime/ownership correction, add a regression,
  audit and then recheck the original composed boundary through a separate gate.
- [ ] Run affected Bazel and ordered Cargo checks, native audits, reference,
  redaction, standards, Markdown and diff checks. Commit/push truthful outcomes;
  leave this parent task and accepted-share unresolved unless all criteria pass.

Effect policy: offline inspection and synthetic virtual effects only, no physical
USB/detection/network/device effects, no signer or real credentials, no grants,
mining, work/nonce/submission or ASIC activity in the minimal probe. The synthetic
Noise handshake is solely an in-memory byte exchange. Maintain16KiB main stack,
SDK extra512 bytes, original internal/PSRAM routing/reserve and all authority,
accounting/safety limits. No original seal repair or core clearing. Raw memory,
registers/debugger output stay in ignored0700 roots/0600 files under ADR-0030;
share only redacted findings and hashes. Parity remains90/95.

noise-probe-baseline-enabled: false

Baseline execution gate enabled for exactly one clean/pushed Noise-only baseline. Admission requires a fresh exact
marked package, verified offline stack/resource bounds and a tested repo-owned
command. Once published, permit one Noise-only valid-handshake seed1 baseline,
QEMU at most30 seconds and any debugger at most25 seconds, single owned process
family, no automatic unchanged retry. Always collect partial facts/core on panic
and terminate/reap children/listeners before sealing. Missing checksum/ELF, stale
source, unexpected effects, invalid input/phase proof or unproved release stops
this stage. A differing or green minimal result diagnoses context dependence;
it is not full lifecycle or hardware qualification. Subsequent variants require
specific hypotheses and separately published scope, not another broad Start.

Offline revalidation | 2026-10-01: offline001 and independent offline002 both
verify checksum/full ELF. Main SP is10168 bytes below the SDK TCB base; observed
deduplicated native entry frames total25824 bytes. Fixture caller8464 and
constructor7136 overlap Noise completion6080 and outer frames. This proves a
virtual callsite stack contract violation. The96-byte Vec pointer is aligned,
in configured PSRAM and consistent across frames; its payload/header and heap
registry descriptor are absent. The direct registry node is unaligned/outside
configured memory. Earliest overwrite, double-free and physical cause are unproved.
See `docs/testing/20261001-noise-panic-diagnosis.md`; raw findings remain private.

Baseline publication | 2026-10-01: repo-owned command
`just virtual-emulator noise --manifest <fresh-current-clean-marked-package>/virtual-package.json --audit <fresh-exact-ELF-native-audit>.json --evidence-dir <repo>/scratch/virtual-noise-diagnostic/baseline001 --seed 1 --mode valid`.
One-use claim: ignored `scratch/virtual-noise-diagnostic/baseline001.claim.json`,
created exclusively before effect and never reset/reused. Only this command/seed/
mode is enabled. Require clean pushed code, package source equal to current HEAD,
source_dirty false, full ELF/image/config hashes, current compiled-source closure,
fixed16KiB stack/SDK routing, live current auditor+objdump replay and every required
selected native path within14336 bytes before admission. No changing declared
artifact identities to reuse a stale report. Retain all failed/unused prior roots.

Offline build001 observed Noise-only completion path12704, frame path7696 against
14336 available. Twelve native audit and23 runner regressions pass; seven actual
Noise host regressions pass. Broader crypto/indirect-call closure remains unproved,
so baseline must additionally prove all14 phase checkpoints, heap integrity,
configured stack observations and actual SDK low-water at least2048. The dirty
build001 is diagnostic development evidence, not the enabled execution package;
a fresh clean marked build and exact audit are required after this publication.
Result must bind validator/source/package/auditor/emulator and verify process/FD
release before finalization; failures/partial core survive independently. A
collection cutoff never becomes fixture natural completion. No full lifecycle,
physical diagnosis or parity qualification is implied by Noise-only success.

Baseline001 result and re-plan | 2026-10-01: the one-use claim was consumed at
pushed df639c99, clean virtual ELF80875ff9aaef090bc3f9016693c60de2fc99161771e66f9c87c2959b7750a94d,
compiled sourcebfeff883011f1d720af6c4deb018c61f7ce465f086fe8a71711e5b1131d97d62.
The minimal target panicked before a result; process/FD release passed, the virtual
core partition was empty. The first LoadStorePIFAddrError is in TLSF integrity
inspection; the second LoadProhibited is in panic/core-storage preparation, not
another baseline invocation. Preserve both partial roots and the consumed claim.

Correction from exact native caller instructions: optimized DWARF line attribution
suggested BeforeCompletion, but the actual call loads phase111AfterCompletion,
then advances to112. Valid certificate verification had returned before the
failing heap check. Do not infer that completion was never entered. The valid
certificate/strauss/odd-table path requires17264 bytes before additional branches,
exceeding even nominal16896 allocation. The old selected mix_hash fit receipt
omitted this required valid branch and cannot admit another full Noise run. No
current task bounds/checkpoint RAM were retained, so first overwrite is still
unseen; retain the distinction from the earlier run017 proved stack overrun.

Staged emulator bisection | 2026-10-01 | Explicit owner strategy change

- [x] Implement typed runtime-prefix diagnostics with checkpoint, boundary-reached
  and released conclusions. Prefixes101..110 stop before the known oversized
  completion branch and must never credit full authentication/frame/mining success.
- [x] Preserve one compiled campaign image/seed/config for runtime-prefix comparisons;
  record exact native caller frames and their delta from baseline6368. The new
  prefix helper is a separately compiled artifact, not an unchanged baseline
  frame. Compile-time code removal is a separate
  corroboration artifact with changed frame measurements, never automatic fix proof.
- [x] Strengthen native audit to bind the exact selected cutoff and constructor/act-two
  crypto-family paths. Report unresolved external and indirect edges explicitly;
  require independent live bounds and heap facts instead of claiming an upper bound. Full-completion admission must also include
  certificate verification and nested crypto, blocking the known17264-byte path.
- [x] Add a private debugger owner that pauses at reached-prefix, completed release
  or first panic/assert before secondary panic capture. Save bounded phase RAM,
  task stack bounds/SP/high-water and heap-integrity facts without invoking target
  allocation functions from GDB. Verify installed debugger grammar offline first.
- [ ] Publish/push command, tests and one-use per-cutoff inventory before effects;
  binary-search the earliest bad prefix with fresh roots and release every owner.
- [ ] Apply only a measured lifetime/ownership correction and real-boundary
  regression, rebuild/audit, then independently admit the corrected full boundary.
  Never raise stack/heap/safety limits or treat a shortened prefix as qualification.

noise-prefix-bisect-enabled: false

Effect contract: emulator-only prefix code bisection, no physical discovery/USB,
credentials, grants, work/mining/ASIC, original replay, erase or parity promotion.
Use same actual Noise initiator/responder/time100/lifetime3600/seed1 and pinned
SDK5.5.4/QEMU backend. Keep16KiB main/configured2KiB margin, SDK extra512 and
allocation routing/reserve unchanged. Allowed runtime cutoffs101,102,103,105,106,
107,109,110;110 is the last pre-completion cutoff. Known bad111 is historical
feedback only and cannot execute until a separately audited correction exists.
Each cutoff consumes one exclusive claim bound to frozen source/ELF/config and
native cutoff audit; max8 distinct prefix trials, no unchanged repeat. Select
halves adaptively from recorded outcomes; distinguish boundary reached while
owners live from cleanup/release failure. Missing facts remain unknown and halt
binary-search inference rather than becoming a healthy prefix. Per trial bound
QEMU35 seconds, debugger25 seconds, output2MiB, private new0700 root/0600 files.
Always terminate/reap QEMU/GDB descendants and close listeners before finalizing.
A cutoff image/diagnostic result is explicitly partial and physical-ineligible.
Stop on source/identity drift, unexpected effects, unproved ownership release or
any requirement to enter the known over-budget branch. Further trials after a
verified boundary-changing correction use a new explicit successor inventory;
no elapsed human-response timeout or per-trial confirmation is required.

Ranked predictions after the red-capable minimal baseline: (1) overlapping virtual
caller/crypto stack frames are strongest; retiring constructor/caller temporaries
must lower measured native peak and restore phase111 integrity at unchanged
limits. (2) earlier heap damage would already fail a pre-completion prefix or
its initial checkpoint. (3) an ownership/emulator allocator defect would persist
after stack bounds/margin and healthy pre-completion heap are independently
proved; no current evidence proves double-free or a Noise algorithm fault.

Commands: build the clean campaign package with `just virtual-emulator build`
and the pairing manifest, then generate each cutoff receipt through
`bazel run //scripts:virtual_noise_prefix_audit`. Execute only
`just virtual-emulator noise-prefix --manifest scratch/virtual-noise-diagnostic/build005-clean-prefix/virtual-package.json --audit AUDIT --stop CUTOFF --seed 1 --evidence-dir ROOT`.
The exact build/audit invocations and placeholder bindings are published in
`docs/testing/20261001-noise-panic-diagnosis.md`. `AUDIT` and `ROOT` are fresh
private paths for that admitted cutoff; all artifact identities are checked
before the one-use claim and effects. Start at 106 only if its native audit
passes, otherwise select an earlier admitted prefix.

Pre-effect verification | 2026-10-02: ordered Cargo format, Clippy, all-target
build and all-feature tests pass; nine affected Bazel suites pass. Exact native
prefix audits for 101/106/110 pass their selected-path budgets. The campaign
helper frame is 2,704 bytes, 3,664 below the separate baseline helper. This is a
changed diagnostic artifact, not a corrected full-handshake claim. The retained
110 crypto-family ECDH path is 13,600 bytes; unresolved external/indirect paths
remain explicit and live margin/heap proof is mandatory. Offline GDB SDK ABI and
command grammar pass against full ELF
`1ca18eea8596e0f4eb33c5e74e5db8c065138cd71231f2bd06d97cf1468aeaa9`.
Compiler-folded marker aliases now require mutually exclusive flag/argument
conditions. Missing Rust DWARF object types are verified from exact ELF symbol
sizes; no live function calls supply those facts. Standards, reference,
redaction, scoped Markdown and diff checks pass. One debugger subagent stopped
with a provider cybersecurity-content flag; its completed evidence was preserved,
and local artifact validation resolved the independent preflight type failure.
No prefix execution occurred before publication. Rebuild from this clean pushed
source and recheck exact package/audit/debugger identities before claiming 106.

Campaign001 result | 2026-10-02: published at 8817a8a3 and consumed cutoff106.
Clean virtual ELF `0d4d9400391793fa4285cfc2db4edde3d8cf3baaed814216efb4a23a52b3c5d0`
completed all six guest prefix checks with no observed failure, but debugger
collection timed out and supplied no independent snapshots. Prefix and cleanup
snapshot conclusions remain unknown. QEMU/GDB process groups, log descriptors
and the loopback listener were released. The one-use claim and all evidence are
immutable. Native breakpoint placement was at the hook entry before register
window setup; this is a diagnostic collection gap, not proof of a target failure.
Do not repeat or promote campaign001.

Checkpoint-only continuation | 2026-10-02 | Explicit owner constraint

- [x] Replace active prefix execution with ordinary application checkpoint
  collection. No debugger attachment, memory capture, core acquisition,
  extraction, inspection, decoding or examination of panic output.
- [x] Project only complete `VIRTUAL_U205` application records into retained
  stdout; discard other stdout and stderr without interpreting it. Bound all
  raw process bytes and partial-line buffering; fail visibly on overflow.
- [x] Keep source/ELF/config/native-audit bindings, existing stack/heap limits,
  one-use cutoff claims, unconditional host cleanup and subset-only results.
  Separate guest facts from unsupported independent task-bound observations.
- [x] Verify real process projection and release plus affected scenario/CLI
  suites; run ordered Cargo, standards/reference/redaction/Markdown/diff checks.
- [x] Publish and push this successor contract before its first trial; collect
  106 then select the remaining half using measured application records. Missing
  completion/checkpoint output stops diagnosis without reading crash material.

noise-prefix-telemetry-enabled: false

Successor effect contract: campaign002 replaces debugger-driven collection only.
Use `just virtual-emulator noise-prefix --manifest scratch/virtual-noise-diagnostic/build006-clean-telemetry/virtual-package.json --audit AUDIT --stop CUTOFF --seed 1 --evidence-dir ROOT`.
Build/package from the clean pushed source and generate each cutoff audit with
`bazel run //scripts:virtual_noise_prefix_audit`; exact manifest identities and
all selected-path limits must pass before claiming an effect. `AUDIT` and `ROOT`
are fresh ignored private paths for each admitted cutoff. Keep SDK5.5.4, pinned
QEMU, the same synthetic credentials/time/seed, main16KiB, margin2KiB and existing
memory routing/reserves. Only101,102,103,105,106,107,109,110 are allowed; no full
certificate completion, Start, grants, fixture/mining/ASIC, physical discovery,
USB, flashing, restart, real credentials or original replay. Campaign002 permits
one newly instrumented106 and at most eight distinct cutoffs total, each bound
to one fixed image/source/config/validator and an exclusive durable claim.
Campaign001 claims are never cleared or reused. QEMU35seconds, total output2MiB,
partial line64KiB; terminate/reap every descendant and release descriptors before
finalization. Retain only ordinary structured application facts and host release
records. Missing, invalid or unsafe checkpoint facts stop further inference;
unknown is never a healthy prefix. No core partition or debugger material is
read after execution. Independent live task-bound proof remains unsupported and
prevents full qualification; guest checks alone may support this explicitly
partial diagnostic comparison. Seals stay immutable, accepted-share stays
unresolved, physical-effect gates stay disabled and parity remains90/95.

Pre-effect review: real synthetic process projection/release regressions pass
35/35, ordinary telemetry runner tests pass8/8, and six affected Bazel suites
pass. Ordered Cargo format/Clippy/build/tests pass. Standards, pinned-reference,
redaction, scoped Markdown and diff checks pass. The new runner is smaller,
removes the debugger dependency from active collection, and never reads a
post-run partition. Explicit v2 result semantics preserve partial guest evidence
without claiming independent task bounds or full qualification. New record
parsing rejects malformed/non-object output before inference. Current commands
and coverage limitations are in
`docs/testing/20261002-checkpoint-only-bisection.md`. Publish this reviewed source
before the first campaign002 effect and keep earlier outcomes immutable.

Campaign002 measured outcome | 2026-10-02: source17461e37, clean virtual ELF
`6d07737269d24d3749adad65150d8285a2d7ce948469a318965f23236f9e758b`,
SDK configuration`cbe2ab13dc61885a77a0052ebee912ce0494393c41d0c3390cae293dbaabd6e1`.
Both106 and110 pass all six application prefix checks with healthy recorded heap
integrity and post-drop release. Minimum observed main free stack:106=9,672 bytes;
110=2,296 bytes, above the unchanged2,048-byte requirement. Verified host process,
descendant, descriptor and writer release passed for both. Target stderr files
are empty; only projected ordinary records were retained (2,342/3,174 bytes).
No debugger, memory/core acquisition, decoding, post-run partition inspection or
examination of panic data was used. Claims and records remain immutable. Close
campaign002 rather than spending further unchanged cutoffs. These are diagnostic
subset passes with independent task bounds unsupported and full qualification
false. They narrow the next source investigation to completion/live caller-frame
overlap; changed prefix frames prevent claiming an unchanged-baseline bisection
or a demonstrated exact failing statement.

Completion lifetime correction | 2026-10-02 | Source and build artifacts only

- [x] Extract preparation into a non-inlined helper whose native frame ends
  before certificate completion. Keep actual codecs, trust, deterministic inputs,
  clock and existing failure categories/phase ordering unchanged.
- [x] Use fallibly reserved owner storage and the existing production completion
  seam to reduce result-construction scratch; retain unconditional post-owner
  release observation on every error. Add a meaningful reservation failure test.
- [x] Verify real host handshake/frame regressions, independent trust rejection,
  exact target native paths through construction, ECDH/signing, valid certificate
  verification and nested crypto, serialization and cleanup. Report unresolved
  paths explicitly; never increase stack, heap, authority or safety limits.
- [ ] Keep every execution gate disabled while building/auditing the correction.
  Publish a separate checkpoint-only full-boundary diagnostic contract only if
  its native resource admission passes. No core/panic/debugger data is permitted
  in this continuation. Missing normal records stop inference.

Current blocker: complete-boundary resource proof and a newly admitted corrected
run do not exist yet. The known original full helper remains inadmissible. This
source-only correction is a measured hypothesis, not a physical firmware repair
or virtual-board qualification. Accepted-share remains unresolved; do not archive.

Correction source verification | 2026-10-02: ordered Cargo format/Clippy/build
and all-feature tests pass (2,581 passed, 0 failed), including all 11 Noise probe
tests with real owner-reservation failure and unreserved-slot regressions. Eleven
affected Bazel suites, standards, reference, redaction and diff checks pass. No
local Markdown formatter is installed; none was added. Host behavior only: the
corrected target's native resource paths remain unaudited, so items 3 and 4 stay
open and the development build under `build007-lifetime` is not admitted.

Responder frame correction | 2026-10-02 | Dev builds only, never executed

- [x] Measure the `85999542` correction with the longest resolved crypto-family
  descent, calibrated against campaign002 cutoff110 (static ECDH 14,272 bytes vs
  runtime ~14,088 used). Completion with certificate verification fell from
  16,160 to 12,160 bytes, but responder ECDH/sign rose to 15,104 bytes, over the
  unchanged 14,336-byte budget: the first correction moved preparation over margin.
- [x] Split preparation into sibling non-inlined frames: `prepare_initiator`,
  `respond`, `construct_responder` and `step_responder`, consuming the act-two
  result in place. The responder is already boxed upstream; no new allocation.
  Phase order, seeds, trust and fault behavior are unchanged.
- [x] Replace the full-probe native auditor with v2: required paths follow the
  corrected helpers and extend each crypto boundary by its longest resolved
  descent, including a named certificate-verification path, encrypted frame and
  outcome emission. Indirect, outside-family, cycle and drop-glue edges remain
  reported, not bounded; `complete_callgraph_bound` stays false.
- [x] Build a clean package from the pushed source and bind a v2 receipt to it.

Dev measurement (`build011-step-in-place`, dirty source, not admitted): all eight
v2 paths fit. Tightest static headroom: responder ECDH/sign 13,552 bytes (784),
completion and certificate verification 12,288 bytes (2,048). Static descents are
lower bounds; only a separately published checkpoint-only run can measure runtime
margin. Gates stay disabled.

Clean v2 receipt | 2026-10-02: source `7b2fe9e6` (clean), compiled source
`88ae2a06275ec63a50663526ca1092f5423a29763a67d6c11ebebf58a2340cbf`, virtual ELF
`73b3f2ccded8958dc47ecf7325c70389a3bce5546a13231f523b470ef49f3dec`, SDK
configuration `10f5335da267cef5bdcd383671ca20a7d41b13072b96cca00aed04a24ded7bda`,
auditor `ce7fd5da2b31e05cdbad163fedd617e46277bca04bb34bc14ae803a740002385`,
receipt `b01492292e8355218a6b7236fd82a76b1d82945f11b22eeae51c04cf45515383`.
All eight paths fit, with figures identical to the dev measurement. Read-only
`admitNoisePackage` recomputes and accepts the native proof. This satisfies the
static native resource admission only. Drop glue, indirect and outside-family
edges remain unbounded, and runtime margin, heap integrity and release are
unmeasured. No emulator, debugger or hardware effect was run. The receipt lives
under ignored `scratch/virtual-noise-diagnostic/build012-clean-split`.

Next: item 4 may now publish a separate checkpoint-only full-boundary contract
for this exact package. It needs full-probe checkpoint telemetry, a one-use claim,
a fresh protected root and an explicit runtime-margin and heap verdict before any
effect gate is enabled. Until then, full qualification stays false and
accepted-share stays unresolved. Do not archive.

Full-boundary checkpoint contract (campaign003) | 2026-10-02 | Explicit owner approval

- [x] Add `noise-checkpoint`, a full-probe runner that keeps only projected
  `VIRTUAL_U205` application records. It does not use a debugger, panic text,
  core decoding or post-run partition reads. Package admission moves to
  `noise-admission.mjs`, so the runner never loads crash tooling. The validator
  identity binds every visible emulator and core-dump script.
- [x] Publish and push this contract with both gates enabled, then build the
  package from that exact commit and bind a fresh v2 receipt to it.
- [x] Run exactly one claimed execution, record the outcome, disable both gates,
  then commit and push.

noise-full-checkpoint-enabled: false

Objective: one emulated valid seed-1 handshake, certificate completion and
encrypted frame round trip on the corrected image. Judge only application
records: boot identity, authenticated round trip, owner release, 14-phase heap
integrity, and at least 2,048 bytes of free stack at every phase.

Commands, run from a clean tree equal to `origin/main`:

```sh
just virtual-emulator build --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --evidence-dir scratch/virtual-noise-diagnostic/build013-full-checkpoint
node scripts/virtual-emulator/noise-stack-audit.mjs --elf ELF --sdkconfig CONFIG --compiled-source-sha256 SHA --output scratch/virtual-noise-diagnostic/build013-full-checkpoint/full-audit-v2.json
just virtual-emulator noise-checkpoint --manifest scratch/virtual-noise-diagnostic/build013-full-checkpoint/virtual-package.json --audit scratch/virtual-noise-diagnostic/build013-full-checkpoint/full-audit-v2.json --seed 1 --evidence-dir scratch/virtual-noise-diagnostic/full-checkpoint001
```

ELF, CONFIG and SHA must come from the build013 package. The receipt must pass
before any effect. The evidence parent stays mode 0700, and the
`full-checkpoint001` child must be absent immediately before launch. Wrapper
stdout and stderr go to separate mode-0600 sibling files.

Allowed effects: one bounded emulator process with a 60-second collection
window, owned and reaped by `runPrivate`. Prohibited: hardware, USB, network,
debugger, memory or core capture, reading target stderr or panic text, partition
inspection, and raising any stack, heap or safety limit.

Retry bound: one execution. The claim
`scratch/virtual-noise-diagnostic/full-checkpoint001.claim.json` is written
exclusively immediately before the effect. A failure before that claim consumes
nothing and may be repaired and rerun with a fresh root. Once claimed, an
unchanged retry is prohibited, and any successor needs a new contract and a
verified fix.

Stop conditions: a failed check, missing or invalid records, an unproven release,
or a source or validator change. Missing normal records stop inference without
reading crash material. Evidence and privacy: raw artifacts stay under ignored
mode-0700 roots; commits record only digests, check outcomes and numeric
observations. Passing would prove only this emulated diagnostic. Independent
task bounds stay unsupported, and full-board qualification, hardware, parity
and accepted-share remain unchanged.

Campaign003 outcome | 2026-10-02: the contract was published at `a024948a`. The
package built from that commit has compiled source
`88ae2a06275ec63a50663526ca1092f5423a29763a67d6c11ebebf58a2340cbf`, virtual ELF
`1bc1cfd7c3d5828a2670b271134192fef5c97d7f1f2a6b8d4b2cd8d3326886df`, v2 receipt
`6bd3a035bc4f0335084af59f388bdb2cfe41213ad96922118f8b33bc032e8094` and
validator `b5820382ceaa02e6a7ea37066c92f4f7f08f5eeb1aa6da06cc33783d010ec52e`.
The single claimed execution passed all five checks: boot identity, an
authenticated handshake with certificate completion and a 32-byte frame round
trip, owner release, heap integrity at all 14 phases (101–114), and the
unchanged stack margin. Minimum free main stack was 3,016 bytes, first reached
at phase 109 (responder ECDH and sign), or 13,368 bytes used against the
13,552-byte static path. Completion and the frame stayed shallower. Internal
free heap ranged from 340,723 to 346,403 bytes, with a constant 188,416-byte
largest block. The emulator was released at the collection cutoff, no writer
leases remained, and target stderr was empty. The projected application records
were 3,701 bytes. Result digest:
`8447cc4e865de46429d8a0f608d887500764c2a7df1317647dda30deeaa86a61`. The claim is
consumed, so this run cannot be replayed. Both gates are disabled again.

This supports the frame-lifetime hypothesis in emulation only. The earlier
baseline's heap-integrity fault after completion is absent from the corrected
image. That does not show which statement was originally faulty, and it is not
a device repair. Independent task bounds remain unsupported, and full-board
qualification, hardware, parity (90/95) and accepted-share are unchanged.
Remaining parent-task blockers are listed under Verification below; do not
archive.

Verification: ordered Cargo format/Clippy/build/tests,21 affected Bazel targets,
55 model tests,19 encrypted-profile tests,9 scenario tests, compiler-closure
stale-source/build regressions, process/listener and evidence regressions, five
production native audits, reference/redaction/standards/Markdown/diff checks.
Final verification receipts and public summary accompany the committed milestone.
Completion review: not complete; do not archive. Explicit remaining blockers are
missing actual margin/call-closure proof, strict live fixture/header/nonce vector
(the Gate/page/HTTP/framed-I/O integration, lease flows, the close-rejection owner
release and real allocation interception are complete in increments 1-7 below),
consolidated production lifecycle/ownership profiles, full
qualified host/target corpus and mandatory matching-report rollout. The
corrected-target panic is resolved in emulation (campaign004). Missing
calibration remains an explicit fidelity non-claim. Accepted-share stays active,
all physical-effect gates stay disabled and this task promotes no parity row.

Framed-link integration, increment 1 | 2026-10-10 | Owner-approved plan

Objective: run the production inbound `bwg-worker-serial/0.2` link decisions on
the host. Extract them from firmware `bwg_worker_usb/link.rs` into a pure
`bitaxe-worker-control::serial::link` core behind a statically dispatched effects
trait, keep the firmware file as a thin adapter, and drive the simulation through
real envelopes (Hello, sequence, heartbeat liveness, receive credit, Close) instead
of bare controller JSON. Later increments: outbound writer core; a stdio virtual
device driven by the pinned Gate controller; the Gate page plus loopback HTTP
supervisor, which alone may flip `gate-page-http-seams`.

- [x] Refresh stale status, blocker and parity text; derive the validation report's
  parity figure from the ledger instead of a literal.
- [x] Add a host characterization harness that records today's link effect
  transcripts, and keep a pre-refactor ELF as the stack baseline.
  `//firmware/bitaxe:usb_link_characterization_tests` pins 21 ordered effect
  transcripts covering every reachable failure stage. New `just audit-link-stack`
  on the pre-refactor ELF: `link::run` frame 1,264 bytes, deepest normal path
  2,928 of a 6,144-byte budget (3,216 headroom, 96 unresolved targets).
- [x] Extract the pure core with unit, allocation-failure and Gate conformance
  vector tests; reduce firmware `link.rs` to an adapter with identical transcripts.
  `bitaxe_worker_control::serial::link` (`SerialLinkReceiver`, statically
  dispatched `SerialLinkEffects`, pinned `SerialLinkFailure` codes) has 52 unit
  tests; `:serial_link_tests` drives the pinned Gate serial and integrity vectors
  and armed allocation failures. All 21 characterization transcripts reproduce
  byte for byte through the adapter.
- [x] Update source-ownership tests and the native USB ownership verifier; add a
  link-thread stack audit and compare it against the baseline. The compiler
  spills loop call targets to stack slots, so the audit now also counts every
  address-taken callee (conservative). On that basis the baseline ELF measures
  5,600 bytes and the refactored ELF 5,536 of a 6,144-byte budget, with the same
  envelope-parse deepest path; the control-owner audit is unchanged at 10,256 and
  no runtime realignment callers exist.
- [x] Drive simulation scenarios through the core; add a `framed-serial-link`
  coverage row while `gate-page-http-seams` stays unsupported. Every
  controller-backed scenario now opens a Hello, sends sequenced Control records,
  replays the host's one-second heartbeats at their original instants (keeping
  envelope parsing out of the deep preparation stack) and ends with a clean Close
  checked as `serial_link`. `heartbeat-loss` now closes the link at its liveness
  deadline (`link_heartbeat_timeout`) and, like the firmware, delivers no further
  status or restore frames; owner cleanup is still checked.
- [x] Run the affected Bazel and ordered Cargo checks, firmware build/package,
  native audits, redaction/reference/parity/standards checks; commit and push.

Authorization: repository source, tests, fixtures, documentation, build, package,
static ELF audits, host-backend virtual runs, commit and push only. No detector,
USB/device session, flash, monitor, emulator effect gate, credentials, network
discovery, mining or other hardware effect. No parity promotion.

Verification: ordered Cargo fmt/Clippy (`-D warnings`)/build/tests; all 325
Bazel tests (`just test`); the 21 pinned link transcripts; `just build`,
`just package`, the virtual guest Xtensa build (`just virtual-emulator build`,
compile and frame audit only); link, control and realignment native audits;
host-backend `just virtual-board run` rows for healthy-lifecycle, heartbeat-loss
and cancel-preparation; native USB ownership, redaction, reference, parity and
Bright Builds checks.

Completion review: increment 1 is complete. Firmware and simulation share one
inbound link implementation, pinned byte-for-byte to the previous firmware
behavior, and the virtual corpus now exercises it. Residual risks and non-claims:
the refactored firmware link has not met hardware; it does so only in a later,
separately contracted hardware task. QEMU host/target agreement and the target
main-stack margin were not rerun for the changed scenarios. The outbound writer,
the pinned Gate controller, the Gate page and the HTTP supervisor remain
unexercised, so `gate-page-http-seams` stays unsupported and qualification
stays false. Next: increment 2 (outbound writer core).

Framed-link integration, increment 2 | 2026-10-11 | Owner-approved plan

Objective: share the outbound `bwg-worker-serial/0.2` writer decisions. Extract
hello_ack, device heartbeats, receive and terminal credit, control replies,
output admission, sequencing and framed-versus-bootstrap diagnostics from
firmware `bwg_worker_usb/writer.rs` into a pure
`bitaxe-worker-control::serial::writer` core, prove byte-identical output, and
make the simulated device emit real device-to-host records that a Rust host peer
checks against the pinned Gate rules. Firmware-only diagnostics scheduling stays
in the firmware adapter.

- [x] Add a deterministic writer characterization harness with pinned
  transcripts and a writer-thread stack audit; record the pre-refactor baseline.
  `//firmware/bitaxe:usb_writer_characterization_tests` runs the unchanged
  writer loop against a scripted clock, queue and production-shaped transport
  (per-chunk, pre-drain and post-flush admission checks, real `WriteFailure`s,
  partial-output tracking) and pins 22 transcripts. New `just audit-writer-stack`
  on the pre-refactor ELF: deepest normal path 5,536 of 6,144 bytes (608
  headroom) through credit, `next_record`, `emit` and envelope encode.
- [x] Extract the core with unit and pinned Gate device-record vector tests;
  reduce firmware `writer.rs` to an adapter with identical transcripts.
  `bitaxe_worker_control::serial::writer` (`SerialRecordWriter`, a `&self`
  `SerialWriterView` plus a `&mut` `SerialOutputSink`) has 30 unit tests, and
  `:serial_writer_tests` reproduces the pinned Gate device `heartbeat` and
  `receive_credit` wire strings byte for byte. All 22 writer transcripts and the
  21 link transcripts are unchanged; the real-thread startup harness passes.
- [x] Update source-ownership tests, the native USB ownership verifier, the
  bootstrap writer audit and source inventories; compare the stack audit. The
  bootstrap writer audit now selects the core and the adapter's trait impls
  (frame sum 5,424 of 7,680, decode complete after replacing a purpose `match`
  jump table with table loads), and its synthetic corpus fixture stages every
  source. The writer thread measures 5,616 bytes against the 5,536 baseline
  (6,144 budget; an inline hint made it worse and was reverted), the link thread
  5,568, the control owner is unchanged at 10,256, and no realignment callers
  exist.
- [x] Emit simulated device records through the core, check them with a Gate-rule
  host peer (`serial_output`), and require it in `framed-serial-link` coverage.
  The simulated device acknowledges Hello, heartbeats, credits and replies (or
  rejects) through `SerialRecordWriter`; device heartbeats are replayed with host
  heartbeats at their original instants, and queued heartbeats now precede a
  long request's reply. Every controller scenario, including heartbeat-loss,
  reports a passing `serial_output`; a missing or failed one keeps
  `framed-serial-link` unsupported or failed.
- [x] Run the affected Bazel and ordered Cargo checks, firmware build/package,
  guest build, native audits, host-backend runs and repository gates; commit and
  push.

Authorization: repository source, tests, fixtures, documentation, build,
package, static ELF audits, host-backend virtual runs, commit and push only. No
detector, USB/device session, flash, monitor, QEMU run, emulator effect gate,
credentials, network discovery, mining or other hardware effect. No parity
promotion. The spilled-call-target audit fix belongs to a separate session; this
increment does not edit `scripts/virtual-emulator/noise-stack-audit.mjs` or
`scripts/link-stack-audit.mjs`.

Verification: ordered Cargo fmt/Clippy (`-D warnings`)/build/tests; all 328
Bazel tests (`just test`); 22 writer and 21 link characterization transcripts
unchanged; `:serial_writer_tests` Gate device vectors; 53 simulation unit tests
including the Gate peer, and the 35-seed scenario corpus with no failed checks;
`just build`, `just virtual-emulator build` (compile and guest frame audit only:
992/4,208 bytes, unchanged); writer, link, control and realignment native audits
plus the bootstrap writer audit on the new ELF; `just verify-native-usb-ownership`;
host-backend `just virtual-board run` for healthy-lifecycle, heartbeat-loss,
max-status, replay-challenge and cancel-preparation with passing `serial_link`
and `serial_output`; redaction, reference, parity and Bright Builds checks.

Completion review: increment 2 is complete. Firmware and simulation now share
both directions of the serial link, pinned byte for byte to the previous
firmware, and the virtual corpus judges device output against the Gate's rules.
Residual risks and non-claims: the refactored firmware writer has not met
hardware; it does so only in a later, separately contracted hardware task. The
writer thread is 80 bytes deeper (5,616 of 6,144). QEMU host/target agreement
and the target main-stack margin were not rerun. The simulation does not revoke
the link after a rejection reply as the firmware owner does, models no partial
output or diagnostics schedule, and its Rust peer is an interim stand-in for the
pinned Gate. `gate-page-http-seams` stays unsupported and qualification stays
false. Next: increment 3 (stdio virtual device driven by the pinned Gate
controller).

Framed-link integration, increment 3 | 2026-10-11 | Owner-approved plan

Objective: drive a virtual device with the real pinned Gate controller. A
`tools/virtual-board serve-serial` child speaks `bwg-worker-serial/0.2` on
stdin/stdout through the shared link and writer cores and the simulated
`WorkerControl`; the pinned Gate TypeScript controller (Bun, through its
injectable serial runtime) admits it end to end: Hello, signed capability
`discover`, `prove_possession`, heartbeat, baseline `status`, a further
`status()` and a clean `close()`. Lease flows, the Gate page and the loopback
HTTP supervisor stay out of scope; `gate-page-http-seams` stays unsupported.

- [x] Add a real-time `StdioDevice` to `bitaxe-simulation` that serves the
  firmware's signed capability, the real serial manifest digest and no
  simulation status evidence, and mirrors the firmware owner's reply,
  rejection and revocation handling. Eight in-crate tests drive it with real
  host records and the Gate-rule peer: clean admission and Close, accepted
  possession, baseline status without simulation evidence, device heartbeat
  liveness, rejection then revocation, EOF without Close, and a refused second
  Hello.
- [x] Add `serve-serial` to `tools/virtual-board` with a real-process test.
  `serve-serial --seed=N --capability=PATH [--outcome=PATH]` steps the device
  every 10 ms on the wall clock, writes only protocol bytes to stdout, and on
  stdin EOF writes a new mode-0600 outcome (categories, counts, capability
  digest) and exits 0 only for a clean session.
- [x] Add the Bun Gate driver with process-group ownership and tests for
  admission, an untrusted capability and device loss.
  `//scripts:virtual_board_gate_serial_test` runs the pinned Gate's
  `createWebSerialWorkerController` (real-time injected runtime, in-memory
  continuity, firmware deployment-trust update keys) over a `WorkerSerialPort`
  on a detached `serve-serial` child. The Gate admits the device (ready,
  baseline status, clean close; device outcome clean, exit 0, group released),
  refuses the firmware capability under the Gate fixture trust (only `discover`
  answered), and reports a killed device with a transport-loss category.
- [x] Add a required `gate-controller-serial` coverage row, update the docs, run
  the affected Bazel, ordered Cargo and repository checks; commit and push.

Authorization: repository source, tests, documentation, build, host processes
(the virtual device child and Bun running the pinned Gate code), commit and
push only. No hardware, USB, Web Serial, browser, network, QEMU run, credentials
or parity promotion.

Verification: ordered Cargo fmt/Clippy (`-D warnings`)/build/tests including the
`StdioDevice` unit tests and `serve-serial` real-process tests; all 329 Bazel
tests (`just test`) including `//scripts:virtual_board_gate_serial_test` (three
cases against the pinned Gate, each under a second); `just build`,
`just package`, `just virtual-emulator build` (compile and guest frame audit
only: 992/4,208 bytes, unchanged); host-backend `just virtual-board run` for
healthy-lifecycle (unsupported only for its existing strict-share non-claim)
and heartbeat-loss (passed); redaction, reference, parity and Bright Builds
checks.

Completion review: increment 3 is complete. The pinned Gate's own serial
controller admits, queries and cleanly closes a real-time virtual device that
runs the shared firmware link and writer cores, refuses it under a trust that
did not sign its capability, and reports its loss. Residual risks and
non-claims: Bun comes from the host PATH, as in the existing Gate fixtures, so
the test is not hermetic; no lease flow (Start/Renew needs a signing authority)
crosses the Gate; one logical session per device process; the tests run on the
wall clock with generous bounds. The device's possession is self-asserted on
initial admission, as the Gate allows. `gate-page-http-seams` stays unsupported
and qualification stays false. Next: increment 4 (the Gate page and loopback
HTTP supervisor over this device).

Framed-link integration, increment 4 | 2026-10-11 | Owner-approved plan

Objective: exercise the Gate page and loopback HTTP seams. The pinned acceptance
page, built from the pinned Gate sources and served by the unmodified production
no-mining supervisor (`createNoMiningSupervisor`) on an ephemeral loopback port,
runs from the served bytes under Bun with browser globals shimmed and drives the
real-time virtual device: configure, a trusted Connect click (Hello, signed
`discover`, possession, heartbeat, baseline status), a refresh and Close, with
every published state recorded through the supervisor's own client and judge.
Owner decision: this no-lease run flips `gate-page-http-seams` to passed, with
the non-claims recorded below.

- [x] Add the page driver and test: browser shims, the pinned bundle and page
  served by the real supervisor, and cases for admission, an untrusted
  capability, device loss, a foreign origin and privacy.
- [x] Set `gate-page-http-seams` from the test, update docs and the close-rejection
  non-claim, run the affected and full checks; commit and push. Under full-suite
  load the driver's refresh raced the page's own connect-time status (the Gate
  correctly refused it with `operation_active`); the driver now waits on the
  page's published state, and both Gate tests pass 8 of 8 under 8-way parallel
  runs.

Authorization: repository source, tests, documentation, build, host processes
(the device child, the Bun page driver, a loopback supervisor on an ephemeral
port), commit and push only. No browser, Web Serial, hardware, USB, network
beyond loopback, port 48765 or the owner's Gate tab, QEMU run, credentials or
parity promotion.

Verification: ordered Cargo fmt/Clippy (`-D warnings`)/build/tests; all 330
Bazel tests (`just test`) including `//scripts:virtual_board_gate_page_test`
(five cases, built from the pinned archive's sources) and the unchanged
no-mining supervisor tests; both Gate tests 8 of 8 under 8-way parallel runs;
`just build`, `just package`, `just virtual-emulator build` (compile and guest
frame audit only: 992/4,208 bytes, unchanged); redaction, reference, parity and
Bright Builds checks.

Completion review: increment 4 is complete and `gate-page-http-seams` is set
from a real run: the served pinned page and the unmodified no-mining supervisor
admit, refresh and close the virtual device, record every published state
through the supervisor's judge, record an untrusted capability and device loss,
and refuse a foreign origin. Non-claims: no lease flow (Start, Renew, Restore)
crosses the page, supervisor or Gate, so `physical_host_owner_release` stays
unsupported (now detailed as `lease_restore_owner_release_not_integrated`) and
close-rejection rows still block qualification; the browser is shimmed in Bun,
not Chromium; Bun comes from the host PATH; the supervisor context's package
fields are synthetic and its frozen-source check is not applicable to the
virtual device; port 48765 and the owner's Gate tab were never used.
Qualification stays false.

Framed-link integration, increment 5 | 2026-10-11 | Plan recorded at start

Objective: carry lease flows through the Gate. A throwaway lease authority from
the pinned Gate's development-authority script (generated in a private temporary
directory, never the owner's) signs a qualification-attempt Start and its
renewals; the served acceptance page, a loopback lease supervisor built from the
production signing and judging modules, and the Gate controller drive the
virtual device through Start, Renew, Stop and Close, and through Close with an
active lease.

- [x] Device: a fresh revocation generation per Hello on the real-time device;
  a runtime deployment trust (`serve-serial --trust`); renewal; Gate-valid
  `worker-qualification-v1` evidence; link and writer service while the owner
  actuates; lease outcome fields; tests over a scripted port.
- [x] Gate harness: the throwaway authority and virtual deployment trust, the
  lease supervisor, page and controller lease cases.
- [x] Coverage row `gate-lease-flows`, docs and non-claims; affected and full
  checks; commit and push.

Authorization: repository source, tests, documentation, build, host processes
(the device child, Bun page and signer processes, a loopback supervisor on an
ephemeral port), commit and push only. No browser, Web Serial, hardware, USB,
network beyond loopback, port 48765 or the owner's Gate tab, QEMU run, owner
authority keys, credentials or parity promotion.

Verification: ordered Cargo fmt/Clippy (`-D warnings`)/build/tests, including 18
real-time device tests over a scripted port (nine lease cases: Start with the link
live through preparation, a host silent during preparation, running evidence,
renewal, restore, Close with a lease, Close during preparation, an untrusted
authority); all 330 Bazel tests (`just test`) with
`//scripts:virtual_board_gate_serial_test` (six cases, three with leases) and
`//scripts:virtual_board_gate_page_test` (eight cases, three with leases); both
Gate tests four runs each at eight parallel jobs, all passing; the simulation
scenario corpus passing; `just build`, `just package`, `just virtual-emulator
build` (compile and guest frame audit only: 992/4,208 bytes, unchanged);
redaction, reference, parity and Bright Builds checks.

Completion review: lease flows cross the Gate. Under a throwaway authority the
pinned Gate controller starts, renews and restores a signed qualification attempt
and closes with one active; the served pinned page, behind a test-only loopback
lease supervisor that reuses the production `signAttempt`, client and `/record`
judge, prepares, loads, starts, renews on its own cadence, stops and closes, or
closes with the lease active; every published state passes the judge, and the
device restores cleanly each time. `gate-lease-flows` is a required coverage row
set from both Gate tests. Non-claims: the lease supervisor has no campaign
lineage, preflight, pool input or cooling and budget reviews, so the production
supervisors' preflight is not exercised; the device's owner stack headroom is
modeled, its watchdog is reported absent and it dispatches no work;
`physical_host_owner_release` stays unsupported (`rejected_lease_restore_not_integrated`)
and close-rejection still blocks qualification; the browser is shimmed in Bun; Bun
comes from the host PATH; port 48765 and the owner's Gate tab were never used.
Qualification stays false.

Framed-link integration, increment 6 | 2026-10-11 | Owner-approved scope

Objective: the close-rejection owner release, through the Gate. Owner decisions:
cover both close rejections, and move the physical claim from the in-process
close-rejection scenario (which keeps its independent-restoration check) to a
required coverage row set from the Gate tests. Under a lease, (a) the device
rejects the close-time restore (its cooling proof fails once): the Gate still
closes its port and releases ownership, records the failure through the page
and supervisor, and the device restores itself once the link drops; (b) the
native port close is rejected: the Gate correctly retains its port ownership,
the device restores independently, and ownership is released when the page
unloads.

- [x] Device: `serve-serial --fault=reject-restore` fails the first restoration
  of a started lease at the cooling-proof step; outcome counts rejected
  restorations; tests over a scripted port.
- [x] Gate controller and page cases for both rejections, with lock, ownership,
  supervisor-record and device-restoration assertions. The harness port now
  models a serial line: reader cancel and writer close reach the device only
  through the native port close, which the earlier cases still pass under.
- [x] Coverage row `gate-close-rejection-release`; drop the unsupported
  `physical_host_owner_release` scenario check; docs and non-claims; affected
  and full checks; commit and push.

Authorization: as increment 5 (repository source, tests, documentation, build,
host processes and a loopback supervisor, commit and push only).

Verification: ordered Cargo fmt/Clippy (`-D warnings`)/build/tests, including 20
real-time device tests (two new: a rejected restore is answered, and the device
restores without the host afterwards); all 330 Bazel tests (`just test`) with
`//scripts:virtual_board_gate_serial_test` (eight cases) and
`//scripts:virtual_board_gate_page_test` (ten cases); both Gate tests three runs
each at eight parallel jobs, all passing; `just build`, `just package`, `just
virtual-emulator build` (compile and guest frame audit only: 992/4,208 bytes,
unchanged); redaction, reference, parity and Bright Builds checks.

Completion review: the close-rejection owner release crosses the Gate. When the
device rejects the close-time restore under a lease, the controller and the page
still close the port and release ownership and the lock, the page records
`close_failed` through the supervisor, and the device restores itself after the
link is gone (one rejected and one completed restoration, no pending lease). When
the native port close is rejected, the Gate keeps ownership and its lock, the
page records `close_failed` with ownership retained, and the device, restored by
the close-time restore, ends cleanly once the page unloads. The physical claim is
the required `gate-close-rejection-release` coverage row; the in-process scenario
keeps its independent-restoration check. Non-claims: both rejections are
injected (one cooling-proof failure, a harness port refusal), page unload is
modeled by ending the device's input, and the increment 5 non-claims stand.
Qualification stays false for the remaining parent blockers.

Framed-link integration, increment 7 | 2026-10-11 | Owner-approved scope

Objective: real allocation interception. Owner decisions: type the three real
allocation classes at the receive-envelope parse, the V2 record snapshot and the
outbound envelope encode; fix fault provenance now so a recovered allocation
failure never claims the once-per-boot fatal slot; and make the claim a required
coverage row set from host tests with a capability-aware test allocator, with the
in-process scenarios dropping their always-unsupported "actual allocator" checks.

- [x] Seam and provenance: `bitaxe_core::fallible` marks recoverable allocation
  regions by site; firmware installs hooks that open a per-task recoverable region
  in `bitaxe_fault_provenance`, whose failure callback flags such records in the
  history ring and leaves the legacy first-failure slot to fatal failures; every
  production `try_reserve` site in firmware and its shared crates uses the seam.
- [x] Typed sites: the link receiver parses envelopes without an infallible
  payload copy and reports frame-buffer exhaustion as a typed link failure;
  `V2Record` snapshots fallibly; `SerialEnvelope::encode` writes through a bounded,
  fallible buffer. Stack audits and characterization transcripts re-run.
- [x] Interception: a capability-aware host test allocator (firmware routes these
  allocations PSRAM-first) fails real allocations by site through the production
  code and the virtual device; `bounded-real-allocation-paths` is set from it; the
  board model's default routing matches firmware; docs and non-claims; affected
  and full checks; commit and push.

Authorization: repository source, tests, documentation, build and host processes,
commit and push only. No hardware, USB, flashing, QEMU run, credentials or parity
promotion; firmware changes are build- and host-verified only.

Verification: ordered Cargo fmt/Clippy (`-D warnings`)/build/tests; all 331 Bazel
tests (`just test`) including the new `//crates/bitaxe-simulation:real_allocation_tests`
(12 cases: allocator routing and fallback, the frame buffer, V2 snapshot and encode
sites, and the virtual device ending typed on control-copy, frame-copy and encode
exhaustion and surviving PSRAM-only exhaustion), the C provenance model (three new
cases), the core-dump provenance validator (two new cases) and the native binary
audit fixture; `just build`, `just package`; native audits on the built ELF: link
5,248 and writer 5,392 bytes (down from 5,568 and 5,616), control 10,256 unchanged,
no runtime realignment callers, fault provenance passing with the recoverable entry
points in its IRAM closure (192 added bytes, one callback CAS); `just
virtual-emulator build` (compile and guest frame audit only: 992/4,208 bytes,
unchanged); redaction, reference, parity and Bright Builds checks. One Cargo test
run, made while a full Bazel suite ran concurrently, counted failure lines that
three later runs did not reproduce; the recorded runs pass.

Completion review: real allocation interception is in place and the fatal
provenance receipt is protected. Every production reservation runs in a named
recoverable region; on firmware the recorder flags those failures in its history
and leaves the first-failure receipt to unhandled ones. The receive envelope is
parsed without a payload copy, the V2 snapshot and the outbound encode allocate
only recoverably, and receive-buffer exhaustion is the new pinned link failure
`frame_allocation`. A capability-aware test allocator, routing as firmware does,
fails these real sites typed, including inside the running virtual device. The
in-process scenarios dropped their always-unsupported actual-allocator checks and
`bounded-real-allocation-paths` is set from the real tests. Non-claims: host
allocator faults, not ESP-IDF TLSF behaviour; V2 status `to_value`, controller
reply serialization, Hello strings and other unmarked allocations still abort on
exhaustion; firmware recoverable marking is not hardware-verified. The parent's
named-phase allocation profile item stays open. Qualification stays false.

Deferral (2026-10-21): the owner deferred this task. Its increments 1-7 are
complete; the open parent items above remain: the seeded virtual BM1366 and
peripheral models, the named-phase allocation profile, the composed
startup/Start/status/Stop/recovery runs, QEMU integration and the
pre-flash commands. Before resuming, re-check the device model against the
polled cooling proof (`ba22b7fb`) and the progress-reporting preparation
(`bf98d091`).

### task-device-response-time-measurement | 2026-10-21 | Explain the slower HTTP responses since the PSRAM atomics fix

Owner decision 2026-10-21: record and defer. Soak 006 (ADR-0039
firmware) estimated request handling at p50 90 ms and p95 152 ms. Soak
005, before the fix, gave 77 ms and 104 ms, with an unchanged minimum.
The estimate is a host-side proxy: each sample interval minus the 2 s
sleep, so it includes the network round trip and host work. Visible host
load did not explain the rise. The fix, which turns every Rust atomic into
a libcall and takes a shared spinlock for PSRAM atomics, is a plausible
cause, but that is unproven either way. Mining was unaffected: the median
hashrate and power were unchanged.

- [ ] A device-side measurement of HTTP request handling (for example,
      handler time in the retained log or the response headers), on
      firmware with and without the ADR-0039 flag, under its own
      contract.
- [ ] If the fix is the cause, find the hot atomics (for example, in the
      HTTP or telemetry path), and keep the ones that need PSRAM-safe
      handling.

### task-telemetry-descriptive-extensions | 2026-10-15 | Add descriptive, unambiguous telemetry names as post-parity extensions

Owner request (2026-10-15). Start only after the parity work is complete.

Several upstream names are ambiguous or overloaded:
- `bestDiff` (all-time, persisted) versus `bestSessionDiff` (this boot);
- `best_difficulty` on BAP (all-time only);
- `found_block` (subscription) versus `block_found` (request answer);
- `fan_speed` (RPM on BAP subscriptions, but percent in settings);
- unit-less `voltage`, `current` and `power`.

Design them as extensions alongside, never instead of, the upstream fields, so
upstream clients keep working.

- [ ] Inventory the ambiguous or overloaded fields across `/api/system/info`,
      the WebSocket telemetry, BAP and the screen, with their units and scopes
      (lifetime, boot, session, connection).
- [ ] Propose additional, explicitly named fields, for example:
      `bestNonceDifficultyAllTime`, `bestNonceDifficultyBoot`,
      `fanSpeedRpm`, `fanDutyPercent`, `inputVoltageMillivolts`. Split values
      that upstream conflates (per pool connection versus per boot). Record the
      design in an ADR.
- [ ] Keep every upstream field byte-compatible, and document the extensions
      as accepted project behaviour, not parity.

Verification: pending. Completion review: pending.

### task-cross-platform-device-session-adapters | 2026-07-22 | Qualify Linux and Windows ESP device sessions

- [ ] Implement Linux physical/enumeration identity, exclusive ownership,
  receive-only observation, and bounded reacquisition behind the canonical
  device-session contract.
- [ ] Implement the corresponding Windows adapter without weakening
  exclusive ownership, request-once, or private-artifact guarantees.
- [ ] Add platform-native real-process tests.
- [ ] Keep unsupported platforms fail-closed until each exact adapter and its
      task-gated hardware evidence qualify.

Verification: Pending.

Completion review: Pending. macOS remains the only production adapter. Standing
task authorization permits ordinary implementation and task-gated evidence;
credentials, network discovery, direct UART or pin work, and evidence promotion
remain governed by their specific contracts.

### task-str005-noise-cooperative-cancellation | 2026-09-16 | Explore bounded and cancellable Noise cryptography

Status: Deferred by the owner; future backlog outside current parity work.
Do not automatically select this dependency-engineering work while that scope
deferral remains in force. A future scope decision is required to activate it.

Origin: `task-str005-noise-runtime-readiness` and the
[source/API readiness audit](docs/hardware/str005-noise-runtime-readiness.md).
The pinned crypto calls lack supported in-call cancellation and a maximum
ElligatorSwift search length. No five-second device overrun was measured.
The five-second cleanup guarantee is a prospective qualification requirement,
not established upstream parity behavior.

Scope: Assess a supported upstream solution first. If none suffices, evaluate
a narrow, reviewed dependency patch or maintained fork and its maintenance,
cryptographic-review and provenance costs before committing to that approach.
No fork or upstream contribution is commissioned by this backlog entry.

- [ ] Reassess available upstream APIs and the required cancellation guarantee;
      document whether a supported solution avoids a maintained fork.
- [ ] If activated, bound scalar-generation retries and make the native
      ElligatorSwift search cancellable/bounded without biased fallback keys,
      partial-success results or changed wire/authentication semantics.
- [ ] Propagate cancellation through act-one creation and act-two authentication;
      preserve first failure and prove actual worker/socket cleanup and disposal
      of owned secret buffers.
- [ ] Verify real-operation cancellation, deadline boundaries, interoperability,
      invalid inputs and failure cleanup; qualify indivisible native operation
      timing, stacks and memory alongside ordinary runtime owners.
- [ ] Review dependency source/provenance and publish exact tested pins before
      handing software readiness back to `task-str005-noise-runtime-readiness`.

Boundaries: Software investigation only after activation; no implicit dependency
upgrade, fork publication, device access, flashing, mining or parity promotion.
Do not edit downloaded registry sources. The frozen successor contract and all
historical evidence remain unchanged. If a different qualification strategy is
chosen, it requires a separate reviewed contract-amendment task. The linked v2
amendment supplies that decision for current Noise qualification; this backlog
entry itself grants no effect authority.

Verification: Pending future activation. Existing characterization tests and
readiness reports establish the gap, not a completed cancellation implementation.

Scope update | 2026-09-16: The [v2 parity-scope amendment](docs/hardware/str005-noise-parity-scope-amendment.md) removes this work
from the runtime/fixture/live readiness dependencies. Keep it deferred as a
stronger future cancellation/latency guarantee. It does not block v2 Noise or
later parity work; any future adoption needs its own verified contract.

Completion review: Pending future activation. No implementation or fork has
been commissioned.

## Effectful Hardware Task Gate

Standing permission for safe USB interaction remains subject to `AGENTS.md` and
`docs/hardware/hardware-attempt-policy.md`. Before any effectful hardware run,
move or add one task block under `Active` that explicitly records:

- the exact permitted repo-owned command and objective;
- the evidence destination, privacy class, and redaction policy;
- recovery, restoration, and cleanup procedures;
- retry bounds, including the unchanged-boundary stop rule; and
- accepted terminal categories and stop conditions.

If any field is missing, hardware work is not authorized. A task entry never
expands the direct-UART, pin-manipulation, privacy, safety, or archived-lineage
boundaries in `AGENTS.md`.

## Accepted Debt and Constraints

- Milestone v1.2 is administratively closed with gaps and is not a release.
- Phase 36 stopped after 8 of 10 plans. Plans 36-07 and 36-04 did not complete.
- SYS-02, EVD-11, EVD-12, and EVD-14 remain blocked. EVD-15 is satisfied by
  exact preservation, typed demotion, and explicit non-claims.
- The sole final Phase 36 hardware attempt sealed `sealed_non_promotion`,
  produced no candidate, and left device restoration unresolved.
- Do not repeat the unchanged hardware attempt. A future attempt requires new
  diagnostic information, a targeted regression-backed fix or objectively
  verified non-invasive remediation, and a complete task-scoped hardware
  contract under the gate above.
- Administrative closure, software verification, or task completion alone is
  never hardware or parity evidence.
