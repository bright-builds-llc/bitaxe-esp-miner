# ADR-0042: Pause the Rust firmware and continue in a C fork

## Status

Accepted on 2026-10-30 by the owner. It ends active development under ADR-0002 and
ADR-0003. Those decisions and the evidence earned under them stay as history.

## Context

Each supported board needs its own evidence (ADR-0014), and a port has to re-earn behaviour
upstream already has. Upstream supports far more boards than this repository can test, and every
upstream release has to be re-implemented here. The image has also run short of space. The full
reasoning and measurements, including how much of the size gap is Rust itself, are in
[the handoff](../project/c-fork-handoff.md).

## Decision

- Pause development in this repository. Nothing is deleted, and it stays public as a record.
- Continue in a new fork of `bitaxeorg/ESP-Miner`, carrying over our extensions and Bitcoin Work
  Gate interoperability, and sending suitable improvements upstream.
- [`docs/project/c-fork-handoff.md`](../project/c-fork-handoff.md) is the bootstrap prompt for
  that fork.

## Consequences

- `TASKS.md` stops being a queue for automatic selection. Work here needs the owner's explicit request.
- Rust evidence doesn't transfer: every claim the fork makes needs its own evidence.
- The fork gives up Rust's memory safety and must rebuild host-test discipline in C.
