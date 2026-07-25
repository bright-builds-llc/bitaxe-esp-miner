---
status: completed
created: 2026-07-04
quick_id: 260704-f2t
title: Require JSON-Only Pool Credentials
---

# Summary

Removed the `.env` pool credential workflow and made `--pool-credentials` JSON the only accepted local pool credential input for Phase 21 live mining verification.

## Changes

- Deleted `pool-credentials.env.example`.
- Removed `.env` fallback instructions from `AGENTS.md` and `docs/release/ultra-205.md`.
- Updated `scripts/phase21-live-mining-evidence.sh` to clear inherited `BITAXE_POOL_*` values before live-prerequisite checks and only repopulate them from validated JSON.
- Replaced the env-var success test with an env-only rejection test while keeping JSON success, HTTP/WebSocket probe, malformed JSON, missing field, and redaction coverage.

## Verification

- `git check-ignore pool-credentials.json`
- `git check-ignore pool-credentials.json.example` returns nonzero
- `test ! -e pool-credentials.env.example`
- `rg "pool-credentials.env.example|env fallback|direct BITAXE_POOL" AGENTS.md docs/release/ultra-205.md pool-credentials.json.example scripts/phase21-live-mining-evidence-test.sh` finds no matches
- `node --check scripts/phase21-pool-credentials-json.mjs`
- `bash scripts/phase21-live-mining-evidence-test.sh`
- `bazel test //scripts:phase21_live_mining_evidence_test`
- `git diff --check`
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`

## Residual Risk

- Existing historical artifacts still mention prior env fallback context; they remain historical and are not rewritten.
