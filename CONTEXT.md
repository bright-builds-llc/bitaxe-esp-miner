# Bitaxe Rust Firmware

This context names the project concepts for a Rust implementation of Bitaxe ESP-Miner firmware, with upstream ESP-Miner kept as the behavioral reference.

## Language

**Device-User Parity**:
The Rust firmware matches the observable behavior a Bitaxe user, administrator, pool, or flashing tool relies on from upstream ESP-Miner. It does not require preserving C module boundaries, FreeRTOS task layout, or internal implementation quirks unless those details affect observable behavior.
_Avoid_: Full parity, byte-for-byte parity, source parity

**Rust Firmware**:
The new firmware implementation written in Rust and owned by this project. It is the product under development, separate from the upstream reference source.
_Avoid_: Rust fork, C rewrite

**Reference Implementation**:
The upstream ESP-Miner codebase used as the authoritative behavioral comparison for device-user parity. It is read-only project evidence, not source code to modify as part of the Rust firmware.
_Avoid_: Upstream fork, legacy code, vendor code

**Reference Refresh**:
An intentional update of the pinned reference implementation commit to a newer upstream ESP-Miner revision. A reference refresh changes the comparison baseline without modifying upstream files.
_Avoid_: Vendor update, upstream patch, submodule edit

**Parity Checklist**:
A project-owned audit artifact that tracks observable device-user parity against the reference implementation. Each item carries reference breadcrumbs, Rust-owned implementation pointers, status, verification evidence, and any documented parity gap.
_Avoid_: Task list, release checklist, TODO list

**First Hardware Target**:
The Bitaxe Gamma 601 with BM1370 ASIC used for earliest USB flashing, smoke tests, and hardware acceptance. Other upstream-supported boards remain in parity scope, and Bitaxe 205 is an available secondary device, but 601 is the preferred bring-up device.
_Avoid_: Default board, only supported board

**Reference Breadcrumb**:
A concise pointer from Rust-owned code or docs to the reference implementation path, function, or parity checklist anchor that explains the behavior being matched. Breadcrumbs are required at module and behavior boundaries, not as line-by-line translation notes.
_Avoid_: Translation comment, source mirror, porting note

**Verification Evidence**:
Recorded proof that a parity checklist item matches the reference implementation or accepted project behavior. Evidence may come from unit tests, golden outputs, API comparison, hardware smoke checks, hardware regression checks, or an explicit deferred gap.
_Avoid_: Done note, implementation status

**Provenance Record**:
A project-owned record of where behavior, copied material, ported logic, third-party dependencies, and release artifacts come from and what license posture applies to them. It exists to keep the Rust firmware's relationship to GPL-3.0 upstream ESP-Miner explicit while preserving MIT licensing for original project work where possible.
_Avoid_: License note, attribution blob
