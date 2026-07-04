---
status: completed
created: 2026-07-04
quick_id: 260704-ejk
title: Add Pool Credentials Env Sample And Docs
---

# Summary

Added a committed pool credential sample file and documented the local-only owner pool input workflow.

## Changes

- Added `pool-credentials.env.example` with safe placeholder values.
- Updated `AGENTS.md` so agents use the sample for shape only and keep real `pool-credentials*.env` files local.
- Updated `docs/release/ultra-205.md` with copy, edit, source, and redaction instructions for Phase 21 live mining verification.

## Verification

- `git check-ignore pool-credentials.env`
- `git check-ignore pool-credentials.env.example` returns nonzero
- `rg "pool-credentials.env.example|BITAXE_POOL_URL|BITAXE_POOL_USER" AGENTS.md docs/release/ultra-205.md pool-credentials.env.example`
- Phase 21 wrapper missing/sourced pool env redaction check
- `bash scripts/phase21-live-mining-evidence-test.sh`
- `git diff --check`
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`

## Residual Risk

- The sample intentionally contains placeholders only. Live mining still requires an operator-owned local credential file, fresh target selection, redaction review, and existing hardware gates.
