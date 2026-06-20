# Keep Rust Firmware in a New Monorepo

This project is a new Rust firmware monorepo, not a Rust branch inside a fork of upstream ESP-Miner. Upstream `bitaxeorg/ESP-Miner` will be included as a pinned read-only reference implementation, while the Rust firmware, Bazel workspace, Justfile, project docs, parity checklist, scripts, and tests live outside that reference so the project can evolve its own architecture without modifying the C codebase.
