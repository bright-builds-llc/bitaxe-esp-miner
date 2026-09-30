# Virtual Ultra 205 target execution

The virtual firmware is an explicit development target. It executes the shared
runtime and scenarios on Xtensa with ESP-IDF and FreeRTOS while supplying modeled
board observations. Its ELF contains
`BITAXE_EXECUTION_PROFILE=virtual-ultra205`; its package declares
`execution_profile: virtual-ultra205` and is ineligible for physical installation.
A virtual pass does not establish hardware parity.

## Managed backend

The required backend is Espressif QEMU
`esp_develop_9.2.2_20250817`, selected by the existing ESP-IDF **5.5.4**
`tools/tools.json`. The repository lock independently checks the selected release,
platform, archive length and SHA-256. The managed SDK installer performs download
and extraction. Doctor compares every installed file with the checksummed archive
before checking the actual executable version. Installation stays under ignored
`.embuild/espressif`; runs stay in new private evidence roots.

On Apple Silicon the official archive SHA-256 is
`aa92e337461d482f5d9f31cd8efc0bd67b3de8fcfcfb567289cb43a59c184651`.
The Linux and Intel macOS checksums are locked separately. No newest-release
lookup or unpinned installer is used.

```sh
just virtual-emulator bootstrap --evidence-dir scratch/virtual-emulator/bootstrap-new
just virtual-emulator doctor --evidence-dir scratch/virtual-emulator/doctor-new
just virtual-emulator build --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --evidence-dir scratch/virtual-emulator/build-new
just virtual-emulator run --manifest scratch/virtual-emulator/build-new/virtual-package.json --scenario healthy-lifecycle --seed 1 --evidence-dir scratch/virtual-emulator/run-new
```

Each root must be absent before execution. Output logs, flash copies, decoded
cores and debugger text remain private. Commands have bounded execution, owned
process groups and verified group release. Timeout collection is recorded
explicitly; it is not natural fixture completion.

## Guest and package binding

The guest uses two ESP32-S3 cores, a **16 MiB** flash image and **8 MiB octal
PSRAM**, matching the retained Ultra 205 profile. It reads production SDK defaults
and the existing partition CSV without modifying them. Board-specific GPIO
Kconfig declarations are excluded because the guest has no physical GPIO
adapter. Its application version begins `VIRTUAL-ultra205-`.

The guest build retains the full ELF and resolved SDK configuration, compares the
resolved configuration against its paired production package, verifies the paired
production ELF digest, and records the image/configuration digests. The SDK's
`esptool merge_bin` generates the padded image and partition layout. No custom
image merger is used.

The pinned QEMU SAR ADC cannot finish the SDK startup constructor's internal
calibration conversion. A retained private GDB trace identifies
`read_cal_channel` → `adc_hal_self_calibration` before guest `main`. The virtual
guest alone wraps that HAL I/O boundary with an explicitly uncalibrated offset
of 2048. This adapter cannot qualify ADC calibration; runtime voltage readings
still come from the functional board model. The production image and SDK source
are unchanged. The package records this synthetic adapter separately from SDK
configuration differences.

QEMU's `-m 8M` supplies PSRAM; it does not resize the internal heap. The explicit
`ssi_psram.is_octal=true` flag selects the SDK-supported octal model. The SDK
QEMU adapter disables emulated timer-group watchdogs; this difference is retained
in the runner arguments and means watchdog hardware behavior needs separate
qualification. Networking is disabled, and no physical USB serial owner is opened.

Target component probes independently exercise NVS boot counters, real pthread
creation/join, actual capability-aware internal allocation/release and target
status serialization. `scenario` calls the same `bitaxe_simulation::run_scenario`
entrypoint as the host backend. Modeled budgets and actual ESP-IDF allocator
measurements are reported separately.

A reset probe requires a second observed boot with the incremented NVS counter.
A panic probe requires actual retained flash core bytes, verified checksum and
matching full ELF identity through the existing offline decoder; merely observing
`panic_requested` cannot qualify panic capture. Debugger output remains private.

## Coverage policy

`target_component_probe_only` is not full-board qualification. Failed, missing or
unsupported required checks prevent full qualification, even if QEMU booted.
Virtual ASIC, dynamics and transport models do not establish electrical, RF,
wall-clock USB or physical timing fidelity. The optional newer `esp-emulator`
beta is excluded from this required backend and does not fill missing coverage.

Sources: [ESP-IDF 5.5 QEMU documentation](https://docs.espressif.com/projects/esp-idf/en/v5.5/esp32s3/api-guides/tools/qemu.html),
[pinned QEMU source](https://github.com/espressif/qemu/tree/esp-develop-9.2.2-20250817),
and the locally generated pinned SDK manifest and `idf_py_actions/qemu_ext.py`.

## Bounded native diagnosis

`virtual-emulator diagnose-heap` requires a separately recorded one-run task
contract and a frozen virtual package. It preflights the actual managed debugger
options, checkpoint symbol size and argument location before spawning QEMU.
It sends the request after the guest task readiness event, then stops at the
first SDK assertion/panic or completion of the fourth checkpoint. A durable
writer lease prevents finalization until each process group and log writer is
released. The host regression exercises a real descendant TCP listener.

The diagnostic stores eight fixed 48-byte records without allocation or logging.
Each record has an entered/completed marker. An incomplete record's zero-filled
fields are unavailable facts; readers publish `null`, not false measurements.
The raw journal and live debugger output remain private.

The first completed diagnostic localized a native heap fault between controller
construction and possession. Heap integrity passed before controller creation;
the next integrity walk faulted. An independently resolved trust-key validation
call path needs at least 10,752 bytes while the measured constructor entry has
9,456 bytes available, including the released checkpoint frames. The complete
call graph has unresolved edges, and the corrupting write was not captured.
These findings justify a bounded caller/frame correction in virtual orchestration;
they do not establish the physical device's panic cause.

The configured main stack remains 16,384 bytes. ESP-IDF's existing
`ESP_TASK_MAIN_STACK` adds `TASK_EXTRA_STACK_SIZE` of 512 bytes; the actual
16,896-byte allocation is consistent with that SDK policy.
