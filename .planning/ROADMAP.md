# Roadmap: Bitaxe Rust Firmware

## Milestones

- [x] **v1.0 Ultra 205 Parity** - Phases 1-21 shipped 2026-07-04; full archive: `.planning/milestones/v1.0-ROADMAP.md`.
- [ ] **Next milestone** - not yet planned; define fresh requirements with `/gsd-new-milestone`.

## Current Planning State

The v1.0 roadmap has been archived to keep the active roadmap small after shipment. V1 delivered Ultra 205 BM1366 device-user parity in deliberate layers: Rust ESP-IDF foundation, typed configuration, BM1366 protocol behavior, Stratum v1 mining, AxeOS-compatible API/static surfaces, safety controllers, OTA/release flows, recovery/OTAWWW evidence boundaries, and live mining/soak evidence.

The v1.0 milestone audit found no unsatisfied requirements and no requirement, integration, or flow gaps. Remaining work is accepted tech debt and future-scope product debt: partial Nyquist validation for older phases and conservative parity rows that remain below `verified` until their exact evidence criteria are met.

## Backlog

No active backlog is currently promoted into the roadmap. Start the next milestone with `/gsd-new-milestone` to define the next requirement set and phase plan.
