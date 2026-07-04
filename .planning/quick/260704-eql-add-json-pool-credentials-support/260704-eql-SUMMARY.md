---
status: completed
created: 2026-07-04
quick_id: 260704-eql
title: Add JSON Pool Credentials Support
---

# Summary

Added JSON-first local pool credential support for Phase 21 live mining verification while keeping `BITAXE_POOL_*` env vars as the internal runtime interface.

## Changes

- Added `pool-credentials.json.example` with safe placeholders and ignored real `pool-credentials*.json` files.
- Added `scripts/phase21-pool-credentials-json.mjs` to validate local JSON credentials and emit redacted-safe env assignments.
- Updated `scripts/phase21-live-mining-evidence.sh` with `--pool-credentials`, JSON loading before live-prerequisite checks, and redaction for JSON/env pool fields.
- Updated Phase 21 wrapper tests for missing credentials, JSON success, malformed and missing JSON fields, redaction, and existing env-var fallback.
- Updated `AGENTS.md`, `docs/release/ultra-205.md`, Bazel script data, and mining allow validation for the `local-owner-supplied` category.

## Verification

- `git check-ignore pool-credentials.json`
- `git check-ignore pool-credentials.json.example` returns nonzero
- `rg "pool-credentials.json.example|poolURL|poolUser|--pool-credentials" AGENTS.md docs/release/ultra-205.md pool-credentials.json.example scripts/phase21-live-mining-evidence.sh`
- `rg "pool-credentials|BITAXE_POOL|DEVICE_URL" AGENTS.md .gitignore docs/release/ultra-205.md pool-credentials.env.example pool-credentials.json.example scripts/phase21-live-mining-evidence.sh`
- `node --check scripts/phase21-pool-credentials-json.mjs`
- `bash scripts/phase21-live-mining-evidence-test.sh`
- `bazel test //scripts:phase21_live_mining_evidence_test`
- `git diff --check`
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`

## Residual Risk

- This does not run live mining by itself. A future live run still needs fresh Ultra 205 detection, an explicit or rule-derived `DEVICE_URL`, local owner credentials, and redacted evidence review.
