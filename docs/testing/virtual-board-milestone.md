# Functional Ultra 205 development milestone

This milestone is **not full virtual-board qualification**. The task stays active,
mandatory pre-flash rollout remains disabled, and physical device effects remain
disabled. Parity stays **90/95**. Accepted-share qualification and all historical
resource-proof gaps remain unresolved; earlier seals are unchanged.

## Implemented and checked

The shared runtime now owns production preparation, shutdown, authority deadlines,
fan freshness, I2C retry and request-queue policy. Physical firmware consumes the
same implementations. The model supplies seeded observations and register/packet
I/O, isolated persistence, capability budgets and reset behavior.

Host scenarios compose real Controller possession and signed V2 admission,
production preparation, Noise transport, ASIC work/nonce parsing, independent
submission/ACK validation and retained accepted-share facts. The public genesis
vector is explicitly distinct from the strict live regtest fixture. Completed I/O
and dispatch accounting have separate observations.

Regressions cover step-5 failure discriminators, queue/I2C failures, cancellation
at every preparation/shutdown step, retained-record absence, reboot, replay,
persistence and Close rejection. Protocol mutations test key trust, ciphertext,
replay, incomplete writes, submission context and ACK accounting. Reset invalidates
old observations, clears volatile resources, rejects stale allocation tickets and
preserves physical inertia and persistent identity/accounting. EMC internal
registers round-trip the existing +5°C production calibration.

Host evidence binds scenario/seed, package/ELF, source/configuration, controller,
fixture, model, validator and actual executable. Compiler inputs are embedded at
build time and rejudged before execution. Failed natural completion cannot become
success after resource release. Frozen manifest/configuration copies are verified.
A durable live-writer marker prevents sealing an ancestor report.

## Target evidence and boundaries

The managed backend verifies Espressif QEMU `esp_develop_9.2.2_20250817` from the
pinned SDK5.5.4 manifest/archive. The separate virtual image uses dual-core S3,
16MiB flash and8MiB octal PSRAM, retains resolved configuration differences and
is rejected by physical flash admission. A guest-only synthetic ADC calibration
adapter avoids an unsupported emulator initialization operation. It makes no
ADC-calibration or electrical-fidelity claim. Emulator watchdog hardware is
explicitly disabled and remains a hardware qualification.

Private target component evidence proves boot, real FreeRTOS task creation/join,
8192-byte internal allocation and release, NVS boot-count persistence across reset,
and deliberate panic/core decoding with verified checksum and full ELF identity.

Composed target execution exposed a constructor stack failure. A frozen diagnostic
found heap checks passing before controller construction; the subsequent heap walk
faulted before possession began. The first native subset needs10752 bytes against
9456 available. Incomplete checkpoint fields are unavailable, not false readings;
the original partial record and a separate corrected interpretation are retained.

Separating the bounded UART reader reduced its dispatch caller from4800 to832 bytes
without changing the configured16384-byte stack or SDK's additional512 bytes.
Boxing the scenario controller alone still failed the next offline margin gate:
the required resolved prefix needed19504 bytes. That gate withheld target execution.
Further caller-lifetime separation passed the ordinary-wrapper-inclusive selected
constructor check:11744 bytes including observed leaf/iteration allowance, leaving
4640 against the unchanged configured stack. The one fresh corrected-image run
still panicked before scenario completion or an actual margin measurement. Target
execution stopped under the recurrence rule; the remaining cause is unresolved. The actual retained core verifies checksum
and exact ELF identity, with no SDK fake frames. Its meaningful trace reaches SDK
`find_containing_heap` through `heap_caps_free`, a Rust vector destructor and Noise
initiator completion during authenticated Start. This proves execution advanced
beyond construction; it does not establish why the allocation metadata/pointer
became invalid. Raw registers, backtraces and core bytes remain private. The complete native graph
has unresolved edges; these are lower bounds, not a global upper bound. None of
this establishes the physical status001 panic cause.

The exact production package built during this milestone has full ELF SHA-256
`27594c9f92e84e907d1f08acd87ea6a1c0644168a45a83c8dcfbbeeacf282b22`.
Its signed Start/Renew stack, panic cutoff, core-storage and original-fault
provenance audits pass. This working-tree development artifact is not a clean
release or an admitted physical candidate.

The preflash observation command saved42 passed and21 unsupported host rows across
three fixed seeds. All63 target corpus rows were withheld because mandatory host
integration remains unsupported. No package was frozen and no flash occurred.
The successor validator labels absent cross-backend comparisons unsupported;
the earlier sealed observation is retained unchanged.

## Remaining qualification requirements

The following remain required, failed or unsupported:

- Corrected Xtensa healthy lifecycle and sufficient native stack margin.
- Strict live fixture/header/nonce qualification; the generic public vector does
  not replace it.
- Capability-specific failure injection into actual parser/snapshot/serializer
  allocation paths, distinct from predicted model budgets.
- Production Gate/page/HTTP and framed-control integration through virtual I/O.
- Full fixture natural completion, historical ownership, listener/serial release
  and live-writer scenarios through one consolidated production lifecycle runner.
- Complete mandatory host/target corpus agreement, qualification and matching-report
  rollout before discovery or sensitive-input loading.

The command interface saves explicit failed/unsupported coverage and cannot freeze
a passing physical package while any required proof is missing. It collects host
facts first and does not spend additional target attempts to compensate for
unsupported host integration. Emergency recovery retains its independent contracts.

Raw dumps, debugger text, live registers, emulator flash copies and private process
logs remain under ignored evidence roots. No physical detection, USB access,
flashing, mining or real grant issuance occurred during this milestone.
